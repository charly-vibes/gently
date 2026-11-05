//! Purpose: the deterministic grid layout serving the recorded oracle
//! corpus (gently-0of) — chain rows, below-attachments, parallel bends,
//! selfloops, and multi-band routes.
//! Responsibilities: place every node on a `(column-rank, band)` slot in
//! one chain-order pass over the edges (extend east on the source's
//! band, else attach below; attach an unplaced endpoint to a placed one
//! west-then-east; a fresh chain starts its own band; edgeless graphs line
//! up on band 0), insert 3-line gap rows exactly where edges route
//! vertically or bend, and route each edge as an orthogonal cell path that
//! never enters a node cell; consume the graph model through its public
//! index-keyed fields.
//! Rationale: a minimal deterministic subset of upstream Graph::Easy 0.69's
//! rank/order/path machinery (lib/Graph/Easy/Layout.pm and submodules),
//! shaped by the recorded v0.69 @ ededa3d7 companions — ge.layout.c1 (pure
//! function of the model: index-ordered vectors, no set/map iteration) and
//! c2 (no node-cell overlap, grid containment) hold; the full property
//! contracts stay with gently-4nx (ge.layout pN).

mod chains;
mod flow;
mod paths;
mod placement;
mod ports;

use crate::graph::{Graph, ObjectKind, Scope};
use flow::Flow;
use paths::{assemble_rows, required_slots, route_edges};
use placement::arrange;
use ports::node_span;

/// A grid cell: `(column, row)` — column grows eastward, row southward.
pub type Cell = (usize, usize);

/// The deterministic grid layout of a graph.
///
/// Node cells are indexed by node index and edge paths by edge index —
/// plain vectors in model order keep the output a pure function of the
/// input (ge.layout.c1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// Grid cell of each node, by node index.
    pub node_cells: Vec<Cell>,
    /// Routed path of each edge — orthogonal, grid-connected, never
    /// entering a node cell — by edge index.
    pub edge_paths: Vec<Vec<Cell>>,
    /// The label cell of each edge, by edge index — `Some(cell)` when the
    /// edge carries a `label` (the cell the label is drawn on, always on
    /// the edge's routed path), `None` otherwise. (ge.layout.c4.)
    pub label_cells: Vec<Option<Cell>>,
    /// Grid extent: `width` columns.
    pub width: usize,
    /// Grid extent: `height` rows.
    pub height: usize,
    /// Grid-cell span of each node, by node index (gently-awd) — grown by
    /// the indexed ports attached to it (gently-byq: an indexed
    /// `front,N`-style port on a node grows its row span to `N+1`, so the
    /// render must see the grown span, not re-derive it from the attrs).
    pub spans: Vec<(usize, usize)>,
    /// The label cell of each slot-routed edge (gently-byq): the
    /// arrow-end cell — the cell the edge's label rides, when the edge
    /// carries a `label` and an indexed port routes it. `None` otherwise.
    pub slot_labels: Vec<Option<Cell>>,
}

/// How an edge travels — decided at placement time, realized as cell paths
/// once the grid rows are assembled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Route {
    /// Self-loop: one gap-row cell directly above the node.
    Selfloop,
    /// This edge placed (at least) one of its endpoints: straight through
    /// the gap when the endpoints are adjacent on one band, otherwise the
    /// placed-below attachment or a route around via the source band.
    Grew,
    /// Both endpoints were already placed and the pair already carries an
    /// edge: bends through the gap row above the band (odd prior count) or
    /// below it (even), so parallel edges never overlap the straight one.
    Parallel(usize),
    /// Parallel channel between a MULTI-ROW pair (gently-awd): channel
    /// `k` rides the boxes' row span — grid row `band + k` — as a
    /// straight run through the tall boxes (the so-flowchart-size
    /// oracle shape); the pair's first edge keeps the band-row run and
    /// channels beyond the span fall back to `Parallel` bends.
    Stacked(usize),
    /// Both endpoints were already placed and this is the pair's first
    /// edge: straight when adjacent, otherwise east/west along the source
    /// band, then vertically through the gap rows at the target column.
    Around,
    /// A `start:`/`end:` port names a south side (gently-89r): the edge
    /// dips through the gap row below the band — exits the source's south,
    /// runs along the gap, enters the target's south (`^`)
    /// (tests/repro/claims/port-side-minimal.observed; the port-* fixtures).
    PortBelow,
    /// A port names a north side: the edge bends through the gap row above
    /// the band, entering the target's north (`v`).
    PortAbove,
    /// An indexed horizontal port slot (gently-byq, upstream Node.pm +
    /// Path.pm): the edge attaches at grid row `band + N` of the slotted
    /// end's node — rendered as that row band's middle line (`band_top +
    /// 3N + 1`). A START slot rides the source band east at the slot row
    /// to the target's column, then bends north into the target's south
    /// (arrow-end `^`, the label cell). An END slot drops south from the
    /// source at the source column, then runs west at the attachment row
    /// into the target's east (`<`); when the drop cells are taken by
    /// another edge's path, it dodges EAST first — exits the source's
    /// east at the band row through the east-gap channel column, down to
    /// the attachment row, then west.
    Slot { end: bool, n: usize },
}

/// Where every node sits and how every edge travels.
pub(super) struct Placement {
    /// `(column-rank, band)` per node, all `Some` after placement.
    pub(super) cell: Vec<Option<(usize, usize)>>,
    /// Travel kind per edge, in model order.
    pub(super) route: Vec<Route>,
    /// Number of bands (== highest band index + 1).
    pub(super) bands: usize,
}

/// Assembled grid rows: which grid row each band occupies, the gap rows
/// at each boundary (boundary `b` is the space above band `b`; boundary
/// `bands` trails the last band — parallel bends and multi-channel
/// routes take more than one), and the total grid height.
pub(super) struct Rows {
    pub(super) row_of_band: Vec<usize>,
    pub(super) gap_rows: Vec<Vec<usize>>,
    pub(super) height: usize,
}

/// Grid columns of every node: upstream places multi-celled nodes at
/// absolute grid coordinates, so the rank→column map must accumulate
/// each same-band predecessor's column span (gently-awd:
/// `col(r) = col(r-1) + cx(r-1) + 1`, which generalizes the even-column
/// `2r` mapping — every 1×1 node contributes no extra). Per-band ranks
/// are already east-ordered by placement, so the extra east shift of a
/// node is the summed `cx - 1` of the band's westward nodes:
/// `col(rank, band) = 2·rank + Σ(cx(n) - 1)` over same-band nodes with
/// smaller rank.
fn node_columns(graph: &Graph, placement: &Placement) -> Vec<usize> {
    let spans: Vec<(usize, usize)> = (0..graph.nodes.len())
        .map(|i| node_span(graph, i))
        .collect();
    let mut cols = vec![0usize; graph.nodes.len()];
    for band in 0..placement.bands {
        // Same-band nodes in east (rank) order — index breaks rank ties
        // deterministically (ge.layout.c1: pure function of the model).
        let mut band_nodes: Vec<(usize, usize)> = (0..graph.nodes.len())
            .filter(|&i| placement.cell[i].is_some_and(|c| c.1 == band))
            .map(|i| (placement.cell[i].unwrap().0, i))
            .collect();
        band_nodes.sort_unstable();
        let mut extra = 0usize;
        for &(rank, i) in &band_nodes {
            cols[i] = 2 * rank + extra;
            extra += spans[i].0.saturating_sub(1);
        }
    }
    cols
}

/// Lay out `graph` deterministically.
///
/// Column ranks grow eastward with one gap column between neighbours;
/// bands grow southward and host one row of nodes each. Gap rows (one
/// 3-line band tall) exist only where an edge routes vertically or bends,
/// matching the recorded oracle: a plain chain row has no leading or
/// inter-band blank lines, a parallel bend or selfloop reserves the gap
/// row above band 0, and a vertical edge reserves the gap row between its
/// endpoint bands.
pub fn layout(graph: &Graph) -> Layout {
    let flow = Flow::of(graph);
    let Some(placement) = arrange(graph, flow == Flow::East) else {
        return empty_layout();
    };
    let grid = assemble_grid(graph, &placement);
    let labels = label_cells(graph, &grid.edge_paths);
    let slot_labels = slot_label_cells(graph, &placement.route, &grid.edge_paths);
    flow::realize(
        Layout {
            label_cells: labels,
            slot_labels,
            ..grid
        },
        flow,
    )
}

/// The assembled grid: spans (one span pass serves the gap slots, the
/// row assembly, the column map, and the router — recomputing it per
/// phase would be an O(n·e) multiplier, ge.layout.c6), gap slots, rows,
/// columns, node cells, and routed edge paths.
fn assemble_grid(graph: &Graph, placement: &Placement) -> Layout {
    let spans: Vec<(usize, usize)> = (0..graph.nodes.len())
        .map(|i| node_span(graph, i))
        .collect();
    let slots = required_slots(graph, placement, &spans);
    let rows = assemble_rows(placement, &spans, &slots);
    let cols = node_columns(graph, placement);
    let node_cells = anchor_cells(placement, &cols, &rows);
    // The eastmost occupied column: a multi-celled node sticks out
    // east over its whole column span (gently-awd). A dodge channel
    // (gently-byq) can lie east of every node — the routed edge cells
    // must be covered too, so the paths are realized first.
    let max_x = eastmost_column(&cols, &spans);
    let edge_paths = route_edges(graph, placement, &node_cells, &rows, max_x, &spans);
    let width = grid_width(&edge_paths, max_x);
    trace_pass(graph, &node_cells, &edge_paths);
    Layout {
        node_cells,
        edge_paths,
        width,
        height: rows.height,
        spans,
        label_cells: Vec::new(),
        slot_labels: Vec::new(),
    }
}

/// The empty layout of an unplaceable graph.
fn empty_layout() -> Layout {
    Layout {
        node_cells: Vec::new(),
        edge_paths: Vec::new(),
        label_cells: Vec::new(),
        width: 0,
        height: 0,
        spans: Vec::new(),
        slot_labels: Vec::new(),
    }
}

/// The grid cell of each node: its accumulated column and its band's row.
fn anchor_cells(placement: &Placement, cols: &[usize], rows: &Rows) -> Vec<Cell> {
    placement
        .cell
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let (_, band) = c.unwrap();
            (cols[i], rows.row_of_band[band])
        })
        .collect()
}

/// The eastmost node cell over every node's column span.
fn eastmost_column(cols: &[usize], spans: &[(usize, usize)]) -> usize {
    cols.iter()
        .zip(spans)
        .map(|(&c, &(cx, _))| c + cx - 1)
        .max()
        .unwrap_or(0)
}

/// The grid width: the routed cells' east extent over the nodes'.
fn grid_width(edge_paths: &[Vec<Cell>], max_x: usize) -> usize {
    edge_paths
        .iter()
        .flatten()
        .map(|&(c, _)| c + 1)
        .max()
        .unwrap_or(0)
        .max(max_x + 1)
}

/// Env-gated decision-trace prototype (gently-asc): `layout.place` —
/// the grid cell and span each node occupies; `render.route` — the
/// per-edge path cell walk with its bend count. No effect when
/// GENTLY_TRACE is unset. Anchored here (before the label pass) so
/// the E1 driver can inject the identical block onto pre-fix
/// revisions, where `spans` does not exist (crate::trace::node_span_of
/// re-reads the attrs instead).
fn trace_pass(graph: &Graph, node_cells: &[Cell], edge_paths: &[Vec<Cell>]) {
    if crate::trace::enabled("layout.place") {
        trace_place(graph, node_cells);
    }
    if crate::trace::enabled("render.route") {
        trace_route(graph, edge_paths);
    }
}

/// Emit one `layout.place` line per node.
fn trace_place(graph: &Graph, node_cells: &[Cell]) {
    for (i, &(c, r)) in node_cells.iter().enumerate() {
        let (cx, cy) = crate::trace::node_span_of(graph, i);
        crate::trace::emit(
            "layout.place",
            &format!(
                "place {} x={} y={} cx={} cy={}",
                graph.nodes[i].name, c, r, cx, cy
            ),
        );
    }
}

/// Emit one `render.route` line per edge: path cell count and bends.
fn trace_route(graph: &Graph, edge_paths: &[Vec<Cell>]) {
    for (ei, path) in edge_paths.iter().enumerate() {
        let bends = path
            .windows(3)
            .filter(|w| (w[1].0 != w[0].0) != (w[2].0 != w[1].0))
            .count();
        crate::trace::emit(
            "render.route",
            &format!(
                "route e{} {}->{} cells={} bends={}",
                ei,
                graph.nodes[graph.edges[ei].from].name,
                graph.nodes[graph.edges[ei].to].name,
                path.len(),
                bends
            ),
        );
    }
}

/// The label cell of each edge: the middle cell of its routed path when
/// the edge carries a `label` (per-edge attribute, falling back to the
/// base edge class), `None` otherwise (ge.layout.c4). A routed edge's
/// path is EXPECTED to be non-empty (c4 routes every edge), but the
/// fan-star shapes at larger fan counts historically realized zero-cell
/// paths (gently-n0d) — a labelled edge with an empty path contributes
/// no label cell instead of underflowing on `path[(path.len()-1)/2]`;
/// the renderer's own empty-path guard then reports the shape as a
/// typed render error, never a panic.
fn label_cells(graph: &Graph, edge_paths: &[Vec<Cell>]) -> Vec<Option<Cell>> {
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(ei, _)| {
            let labelled = graph.get_attr(Scope::Edge(ei), "label").is_some()
                || graph
                    .get_attr(Scope::Class(ObjectKind::Edge, String::new()), "label")
                    .is_some();
            if labelled {
                let path = &edge_paths[ei];
                non_empty_mid(path)
            } else {
                None
            }
        })
        .collect()
}

/// The middle cell of a non-empty path; `None` when the path is empty
/// (the c4 route contract should make this unreachable — it is the
/// gentle degradation floor, not a designed state).
fn non_empty_mid(path: &[Cell]) -> Option<Cell> {
    if path.is_empty() {
        None
    } else {
        Some(path[(path.len() - 1) / 2])
    }
}

/// The label cell of each slot-routed edge (gently-byq): the ARROW-END
/// cell — the decoded upstream rule for indexed-port edges (route B's
/// `^` cell, route C's `<` cell). `None` for unlabelled edges and every
/// non-slot route.
fn slot_label_cells(graph: &Graph, route: &[Route], edge_paths: &[Vec<Cell>]) -> Vec<Option<Cell>> {
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(ei, _)| {
            let labelled = graph.get_attr(Scope::Edge(ei), "label").is_some()
                || graph
                    .get_attr(Scope::Class(ObjectKind::Edge, String::new()), "label")
                    .is_some();
            if labelled && matches!(route[ei], Route::Slot { .. }) {
                Some(*edge_paths[ei].last().expect("slot paths are non-empty"))
            } else {
                None
            }
        })
        .collect()
}
