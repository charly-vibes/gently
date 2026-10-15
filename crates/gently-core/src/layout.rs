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

use crate::graph::{Graph, ObjectKind, Scope};
use flow::Flow;
use paths::{assemble_rows, required_slots, route_edges};
use placement::arrange;

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

/// Which way a port-constrained edge must dip (gently-89r). Upstream
/// Attributes.pm `_flow_as_side` maps the flow-relative spellings onto
/// sides: right = inflow+90 = south, left = north, front/forward = the
/// flow direction itself (the default attachment), back = the opposite;
/// absolute spellings (south/north/east/west, down/up, degrees) map
/// through `_direction_as_side`. Only the east pipeline honors ports (the
/// non-east flows realize a transformed canonical layout — no tier-1
/// fixture pins ports there), so the sides are read in flow space and
/// matched against the canonical east axes. Indexed forms (`front,N`)
/// select a numbered slot on the side — variable node heights, out of
/// scope (gently-byq). Conflicting sides on the two ends fall back to the
/// unported route (no recorded shape pins one).
fn port_dip(graph: &Graph, ei: usize) -> Option<Route> {
    let side = |v: Option<&str>| match v?.split(',').next()? {
        "right" | "south" | "down" | "180" => Some(Route::PortBelow),
        "left" | "north" | "up" | "0" => Some(Route::PortAbove),
        // front/forward/east and back/west keep the default attachment;
        // anything else (unknown spelling) is ignored.
        _ => None,
    };
    let start = side(graph.get_attr(Scope::Edge(ei), "start"));
    let end = side(graph.get_attr(Scope::Edge(ei), "end"));
    match (start, end) {
        (Some(s), None) | (None, Some(s)) => Some(s),
        (Some(s), Some(e)) if s == e => Some(s),
        _ => None,
    }
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

/// Grid-cell span of node `idx` in columns and rows (gently-awd):
/// the `columns`/`rows` attributes — derived from `size: W,H` in the
/// attribute store — parsed as grid cells. Upstream `_calc_size` reads
/// `abs($attr || 1)`, so an absent, empty, or `0` value is one cell.
/// An INDEXED port attached to the node grows the span (gently-byq,
/// upstream Node.pm ~1170 + Path.pm): an indexed `front,N`-style slot on
/// one of the node's horizontal sides (start port on an edge leaving it,
/// end port on an edge entering it) needs row `N` of the node, so the
/// row span grows to `max(cy, N+1)` — the max-side rule `min_y =
/// max(need.west, need.east)`. Vertical sides (`right,left,south,north`)
/// would grow the column span — no recorded fixture pins them, so they
/// contribute nothing here.
fn node_span(graph: &Graph, idx: usize) -> (usize, usize) {
    let side = |key: &str| {
        graph
            .get_attr(Scope::Node(idx), key)
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(1)
    };
    let (cx, cy) = (side("columns"), side("rows"));
    grow_free_ports(graph, idx, cx, cy)
}

/// The free-ports growth loop (gently-a6g, upstream Node.pm `_grow`):
/// a node carrying FOUR OR MORE unspecified edge-ends grows until every
/// end has a free port. Free ports per side: `1 + (size - cnt - portnr)
/// / 2`, summed over north/south (the column span) and east/west (the
/// row span) — a pure sink skips its front side (east for the east
/// flow), so the fan-in hub grows on its row span (four sources: 1+1+1
/// = 3 < 4, then 1+1+2 = 4 → a 3-row span, the fan-layout.observed
/// hub). Each grow step bumps the row span (then the column span) by
/// two — upstream's grow_what order for the east flow. Fewer than four
/// unspecified ends keep the base span (upstream's shortcut), so the
/// two-parallel corpus pair stays 1×1.
fn grow_free_ports(graph: &Graph, idx: usize, cx: usize, cy: usize) -> (usize, usize) {
    // Per-side port counts over the ends attached to this node: side-
    // constrained without a number (cnt), indexed (portnr + max slot).
    // Horizontal spellings land on east/west (si 2), vertical on
    // north/south (si 3); the scan mirrors node_span's single sweep —
    // one branch pair per edge, port classification only at the ends
    // that touch `idx` (a String per visit would dominate the layout,
    // ge.layout.c6).
    let mut cnt = [0usize; 4];
    let mut portnr = [0usize; 4];
    let mut max = [0usize; 4];
    let mut unspecified = 0usize;
    let mut outgoing = 0usize;
    let mut slot_floor = 0usize;
    for (ei, e) in graph.edges.iter().enumerate() {
        if e.from == idx {
            outgoing += 1;
            classify_port(
                graph,
                ei,
                false,
                &mut cnt,
                &mut portnr,
                &mut max,
                &mut unspecified,
                &mut slot_floor,
            );
        }
        if e.to == idx {
            classify_port(
                graph,
                ei,
                true,
                &mut cnt,
                &mut portnr,
                &mut max,
                &mut unspecified,
                &mut slot_floor,
            );
        }
    }
    if unspecified < 4 {
        // The port-growth floor still applies (gently-byq: an indexed
        // slot needs row N+1).
        return (cx, cy.max(slot_floor));
    }
    // Minimum spans demanded by the constrained ports (upstream min_x
    // from north/south, min_y from east/west).
    let mut cx = cx.max(max[0]).max(max[1]);
    let mut cy = cy.max(max[2]).max(max[3]);
    // Free ports: north/south spend the column span, east/west the row
    // span; a pure sink skips its front side (east, east flow).
    let free = |cx: usize, cy: usize| -> usize {
        let mut free = 0;
        for (si, size) in [(0, cx), (1, cx), (2, cy), (3, cy)] {
            if outgoing == 0 && si == 2 {
                continue;
            }
            free += 1 + (size.saturating_sub(cnt[si] + portnr[si])) / 2;
        }
        free
    };
    let mut grow_cy = true;
    while free(cx, cy) < unspecified {
        if grow_cy {
            cy += 2;
        } else {
            cx += 2;
        }
        grow_cy = !grow_cy;
    }
    (cx, cy)
}

/// Classify one edge end attached to the growing node (start port when
/// `end == false`): indexed horizontal slots count as east/west ports
/// (plus the row floor), unindexed side spellings count on their side,
/// anything else is an unspecified end.
#[allow(clippy::too_many_arguments)]
fn classify_port(
    graph: &Graph,
    ei: usize,
    end: bool,
    cnt: &mut [usize; 4],
    portnr: &mut [usize; 4],
    max: &mut [usize; 4],
    unspecified: &mut usize,
    slot_floor: &mut usize,
) {
    if let Some(n) = indexed_slot(graph, ei, end) {
        portnr[2] += 1;
        max[2] = max[2].max(n + 1);
        *slot_floor = (*slot_floor).max(n + 1);
        return;
    }
    let head = graph
        .get_attr(Scope::Edge(ei), if end { "end" } else { "start" })
        .map(|v| v.split(',').next().unwrap_or(""));
    match head {
        Some("front" | "forward" | "east" | "back" | "west") => cnt[2] += 1,
        Some("right" | "south" | "down" | "left" | "north" | "up") => cnt[3] += 1,
        _ => *unspecified += 1,
    }
}

/// The indexed port slot one edge carries: `start: <side>,N` (on the
/// edge's start, `end: false`) or `end: <side>,N` (on its end, `end:
/// true`) — the side must be a horizontal spelling (flow front/back,
/// east/west; upstream Attributes.pm `_flow_as_side`) and `N` a number.
/// Unindexed spellings and vertical sides give `None`.
pub(crate) fn indexed_slot(graph: &Graph, ei: usize, end: bool) -> Option<usize> {
    let raw = graph.get_attr(Scope::Edge(ei), if end { "end" } else { "start" })?;
    let (side, n) = raw.split_once(',')?;
    let horizontal = matches!(side, "front" | "forward" | "east" | "back" | "west");
    let n = n.trim().parse::<usize>().ok()?;
    horizontal.then_some(n)
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
        return Layout {
            node_cells: Vec::new(),
            edge_paths: Vec::new(),
            label_cells: Vec::new(),
            width: 0,
            height: 0,
            spans: Vec::new(),
            slot_labels: Vec::new(),
        };
    };
    // One span pass serves the gap slots, the row assembly, the column
    // map, and the router (node_span scans the edges — recomputing it
    // per phase would be an O(n·e) multiplier, ge.layout.c6).
    let spans: Vec<(usize, usize)> = (0..graph.nodes.len())
        .map(|i| node_span(graph, i))
        .collect();
    let slots = required_slots(graph, &placement, &spans);
    let rows = assemble_rows(&placement, &spans, &slots);
    let cols = node_columns(graph, &placement);
    let node_cells: Vec<Cell> = placement
        .cell
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let (_, band) = c.unwrap();
            (cols[i], rows.row_of_band[band])
        })
        .collect();
    // The eastmost occupied column: a multi-celled node sticks out
    // east over its whole column span (gently-awd). A dodge channel
    // (gently-byq) can lie east of every node — the routed edge cells
    // must be covered too, so the paths are realized first.
    let max_x = cols
        .iter()
        .zip(&spans)
        .map(|(&c, &(cx, _))| c + cx - 1)
        .max()
        .unwrap_or(0);
    let edge_paths = route_edges(graph, &placement, &node_cells, &rows, max_x, &spans);
    let width = edge_paths
        .iter()
        .flatten()
        .map(|&(c, _)| c + 1)
        .max()
        .unwrap_or(0)
        .max(max_x + 1);
    // Env-gated decision-trace prototype (gently-asc): `layout.place` —
    // the grid cell and span each node occupies; `render.route` — the
    // per-edge path cell walk with its bend count. No effect when
    // GENTLY_TRACE is unset. Anchored here (before the label pass) so
    // the E1 driver can inject the identical block onto pre-fix
    // revisions, where `spans` does not exist (crate::trace::node_span_of
    // re-reads the attrs instead).
    if crate::trace::enabled("layout.place") {
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
    if crate::trace::enabled("render.route") {
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
    let labels = label_cells(graph, &edge_paths);
    let slot_labels = slot_label_cells(graph, &placement.route, &edge_paths);
    let grid = Layout {
        node_cells,
        label_cells: labels,
        edge_paths,
        width,
        height: rows.height,
        spans: spans.clone(),
        slot_labels,
    };
    flow::realize(grid, flow)
}

/// The label cell of each edge: the middle cell of its routed path when
/// the edge carries a `label` (per-edge attribute, falling back to the
/// base edge class), `None` otherwise (ge.layout.c4). Paths are
/// non-empty for every routed edge, so the middle cell always exists.
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
                Some(path[(path.len() - 1) / 2])
            } else {
                None
            }
        })
        .collect()
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
