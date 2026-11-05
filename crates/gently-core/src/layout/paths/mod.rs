//! Purpose: the edge-path realization half of the layout — gap-slot
//! accounting, grid-row assembly, and each edge's routed cell path by
//! travel kind (split from layout.rs past the tidy file limit,
//! gently-cbk).
//! Responsibilities: count the gap rows each boundary needs, assemble the
//! band/gap row sequence, and realize every route as an orthogonal cell
//! path — straight, bend, port dip, parallel channel, selfloop, and the
//! multi-band walks (east-going along the source band; west-going bending
//! out of the source's north/south first, gently-cbk), both vertical
//! walks jogging east around band rows occupied by stacked nodes
//! (gently-4ga).
//! Rationale: shaped by the recorded v0.69 @ ededa3d7 companions —
//! ge.layout.c1 (pure function) and c4 (orthogonal routes).

mod bends;
mod multiband;
mod port;
mod rows;
mod slots;
mod walk;

pub(in crate::layout) use rows::assemble_rows;
pub(in crate::layout) use slots::required_slots;

use super::{Cell, Placement, Route, Rows};
use crate::graph::Graph;
use std::collections::HashSet;

/// Shared routing context: the occupied node cells (the walks never
/// enter one) and the eastmost grid column (the dodge side guard,
/// gently-4ga).
pub(super) struct Ctx<'a> {
    pub(super) occupied: &'a HashSet<Cell>,
    pub(super) max_x: usize,
}

/// Both endpoints of one edge as the routing pass sees them: the
/// placement slots, the grid cells, and the grid-cell spans (gently-awd).
pub(super) struct Endpoints {
    pub(super) su: Cell,
    pub(super) sv: Cell,
    pub(super) gu: Cell,
    pub(super) gv: Cell,
    pub(super) span_u: (usize, usize),
    pub(super) span_v: (usize, usize),
}

/// Realize each edge's travel as a cell path over the assembled rows.
/// `spans` carries every node's (column, row) grid-cell span (gently-awd):
/// the occupied set covers a multi-celled node's WHOLE span (the walks
/// never enter any of its cells), and the straight-gap detection uses
/// the span to find the single gap column east/west of a node.
pub(super) fn route_edges(
    graph: &Graph,
    placement: &Placement,
    node_cells: &[Cell],
    rows: &Rows,
    max_x: usize,
    spans: &[(usize, usize)],
) -> Vec<Vec<Cell>> {
    let occupied = span_cells(node_cells, spans);
    let ctx = Ctx {
        occupied: &occupied,
        max_x,
    };
    let mut used: HashSet<Cell> = HashSet::new();
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(ei, e)| edge_at(ei, e, placement, node_cells, spans, rows, &ctx, &mut used))
        .collect()
}

/// The occupied set covers a multi-celled node's WHOLE span — the walks
/// never enter any of its cells.
fn span_cells(node_cells: &[Cell], spans: &[(usize, usize)]) -> HashSet<Cell> {
    let mut occupied: HashSet<Cell> = HashSet::new();
    for (&(c, r), &(cx, cy)) in node_cells.iter().zip(spans) {
        for dc in 0..cx {
            for dr in 0..cy {
                occupied.insert((c + dc, r + dr));
            }
        }
    }
    occupied
}

/// The path of edge `ei`: its endpoints, the route's cell path, and the
/// `used` ledger. The cells earlier edges' paths took (model order) let
/// an indexed end-slot edge dodge EAST when its drop column is taken by
/// one of them (gently-byq).
#[allow(clippy::too_many_arguments)]
fn edge_at(
    ei: usize,
    e: &crate::graph::Edge,
    placement: &Placement,
    node_cells: &[Cell],
    spans: &[(usize, usize)],
    rows: &Rows,
    ctx: &Ctx,
    used: &mut HashSet<Cell>,
) -> Vec<Cell> {
    let ep = Endpoints {
        su: placement.cell[e.from].unwrap(),
        sv: placement.cell[e.to].unwrap(),
        gu: node_cells[e.from],
        gv: node_cells[e.to],
        span_u: spans[e.from],
        span_v: spans[e.to],
    };
    let path = edge_path(placement.route[ei], &ep, rows, ctx, used);
    used.extend(path.iter().copied());
    path
}

/// The cell path of one edge, by travel kind. Straight: the single gap
/// column between adjacent neighbours on one band. Bend: through the gap
/// row above (or below) the band, entering/exiting through the node
/// columns. Multi-band: east-going along the source band to the target
/// column, then vertically through the gap rows (oracle: the diamond's
/// c->d); west-going bends out of the source's north/south first and runs
/// along the gap row adjacent to the target band (oracle: bg-edge-node's
/// Ulm->car — the source-band walk would drive through the nodes between).
fn edge_path(
    route: Route,
    ep: &Endpoints,
    rows: &Rows,
    ctx: &Ctx,
    used: &HashSet<Cell>,
) -> Vec<Cell> {
    match route {
        Route::Selfloop => bends::selfloop_path(ep.su, ep.gu, rows),
        Route::Parallel(_) => bends::bend_path(
            ep.gu,
            ep.gv,
            ep.span_u,
            ep.span_v,
            bends::bend_gap_row(route, ep.su.1, rows, ep.gu.1),
        ),
        // The port dips run one gap row, all the way between the two node
        // columns — straight through under (or over) everything between.
        Route::PortBelow | Route::PortAbove => port_path(route, ep, rows),
        Route::Grew | Route::Around => grown_path(ep, rows, ctx),
        // An indexed port slot (gently-byq): the decoded upstream routes.
        Route::Slot { end, n } => port::slot_path(ep, ctx, used, end, n),
        // A stacked channel (gently-awd): straight through the gap
        // column at grid row `band + k` — the pair's boxes' row span
        // hosts the channel rows (the so-flowchart-size oracle shape);
        // a non-adjacent pair bends through the stacked row.
        Route::Stacked(k) => stacked_path(k, ep),
    }
}

/// A port-dip edge: the first gap row of the boundary below (PortBelow)
/// or above (PortAbove) the source band, straight between the two node
/// columns.
fn port_path(route: Route, ep: &Endpoints, rows: &Rows) -> Vec<Cell> {
    let boundary = if route == Route::PortBelow {
        ep.su.1 + 1
    } else {
        ep.su.1
    };
    let g = rows
        .gap_rows
        .get(boundary)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(ep.gu.1);
    bends::bend_path(ep.gu, ep.gv, ep.span_u, ep.span_v, g)
}

/// A stacked channel: the gap-column cell at grid row `band + k`, or the
/// bend through that row when the pair is not column-adjacent.
fn stacked_path(k: usize, ep: &Endpoints) -> Vec<Cell> {
    let row = ep.gu.1 + k;
    match bends::straight_gap(ep.gu, ep.gv, ep.span_u, ep.span_v) {
        Some(c) => vec![(c, row)],
        None => bends::bend_path(ep.gu, ep.gv, ep.span_u, ep.span_v, row),
    }
}

/// A Grew/Around route. Straight through the single gap column between
/// the two nodes — east of the source's column span or west of the
/// target's (gently-awd: a multi-celled node's east border sits at
/// `col + cx - 1`, so the gap column shifts east).
fn grown_path(ep: &Endpoints, rows: &Rows, ctx: &Ctx) -> Vec<Cell> {
    let (gu, gv) = (ep.gu, ep.gv);
    let (span_u, span_v) = (ep.span_u, ep.span_v);
    match bends::straight_gap(gu, gv, span_u, span_v) {
        // Straight through the gap column when the pair shares a
        // row — literally, or because one node's grown span hosts
        // the other's row (gently-a6g: the fan-in star's east-ring
        // attachment — n2's row IS the hub's bottom span row, the
        // edge runs straight west into the hub's east face).
        Some(c) if shared_row(gu, gv, span_u, span_v) => vec![overlap_cell(c, gu, gv, span_v)],
        _ if gu.1 == gv.1 => bends::bend_path(
            gu,
            gv,
            span_u,
            span_v,
            bends::bend_gap_row(Route::Grew, ep.su.1, rows, gu.1),
        ),
        _ => multiband::multiband_path(ep, rows, ctx),
    }
}

/// Do the two endpoints' rows coincide, or does one node's grown span
/// host the other's band row?
fn shared_row(gu: Cell, gv: Cell, span_u: (usize, usize), span_v: (usize, usize)) -> bool {
    gu.1 == gv.1
        || (gu.1 >= gv.1 && gu.1 < gv.1 + span_v.1)
        || (gv.1 >= gu.1 && gv.1 < gu.1 + span_u.1)
}

/// The straight run rides the row the two spans OVERLAP on: gu's band
/// row when gu is the rider inside gv's span (the fan-in star's east
/// ring), gv's band row when gv rides inside gu's span — the fan-OUT
/// stack (gently-206): the source grew a row span hosting the targets
/// below it, and the straight cell must sit on the TARGET's row to abut
/// its west face, not on the source's top row (which is diagonal to
/// every rider below the first).
fn overlap_cell(c: usize, gu: Cell, gv: Cell, span_v: (usize, usize)) -> Cell {
    let row = if gu.1 >= gv.1 && gu.1 < gv.1 + span_v.1 {
        gu.1
    } else {
        gv.1
    };
    (c, row)
}
