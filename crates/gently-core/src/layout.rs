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
        };
    };
    let slots = required_slots(graph, &placement);
    let rows = assemble_rows(placement.bands, &slots);
    let node_cells: Vec<Cell> = placement
        .cell
        .iter()
        .map(|c| {
            let (col, band) = c.unwrap();
            (2 * col, rows.row_of_band[band])
        })
        .collect();
    let max_col = placement
        .cell
        .iter()
        .filter_map(|c| c.map(|p| p.0))
        .max()
        .unwrap_or(0);
    let max_x = 2 * max_col;
    let width = max_x + 1;
    let edge_paths = route_edges(graph, &placement, &node_cells, &rows, max_x);
    let labels = label_cells(graph, &edge_paths);
    let grid = Layout {
        node_cells,
        label_cells: labels,
        edge_paths,
        width,
        height: rows.height,
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
