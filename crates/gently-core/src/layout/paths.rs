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

use super::{node_span, Cell, Placement, Route, Rows};
use crate::graph::Graph;
use std::collections::HashSet;

/// Shared routing context: the occupied node cells (the walks never
/// enter one) and the eastmost grid column (the dodge side guard,
/// gently-4ga).
struct Ctx<'a> {
    occupied: &'a HashSet<Cell>,
    max_x: usize,
}

/// Both endpoints of one edge as the routing pass sees them: the
/// placement slots, the grid cells, and the grid-cell spans (gently-awd).
struct Endpoints {
    su: Cell,
    sv: Cell,
    gu: Cell,
    gv: Cell,
    span_u: (usize, usize),
    span_v: (usize, usize),
}

/// How many gap rows each boundary needs (boundary `k` is the gap above
/// band `k`; `k == bands` trails the last band). Only edges that route
/// vertically or bend need one; a plain chain row stays gap-free (oracle:
/// mixed_isolated). Parallel multi-edges take one gap per extra channel.
pub(super) fn required_slots(graph: &Graph, placement: &Placement) -> Vec<usize> {
    let mut slot = vec![0usize; placement.bands + 1];
    for (ei, e) in graph.edges.iter().enumerate() {
        let pu = placement.cell[e.from].unwrap();
        let pv = placement.cell[e.to].unwrap();
        mark_slot(&mut slot, placement.route[ei], pu, pv);
    }
    slot
}

/// Raise a boundary's gap count to `count` when lower.
fn raise(slot: &mut [usize], boundary: usize, count: usize) {
    if slot[boundary] < count {
        slot[boundary] = count;
    }
}

/// Mark the gap slots one edge needs.
fn mark_slot(slot: &mut [usize], route: Route, pu: Cell, pv: Cell) {
    match route {
        Route::Selfloop => raise(slot, pu.1, 1),
        // A south-port dip takes the first gap row below the band; a
        // north-port bend the first gap row above it.
        Route::PortBelow => raise(slot, pu.1 + 1, 1),
        Route::PortAbove => raise(slot, pu.1, 1),
        Route::Parallel(prior) => {
            let (boundary, k) = parallel_channel(prior, pu.1);
            raise(slot, boundary, k + 1);
        }
        Route::Grew | Route::Around => mark_travel(slot, pu, pv),
    }
}

/// The boundary and 0-based gap index a parallel edge with `prior` prior
/// edges on its pair travels: odd priors bend above the band (first,
/// second, ... gap above), even priors below it — channels 1/2 keep the
/// recorded two-parallel oracle shape (above, then below).
fn parallel_channel(prior: usize, band: usize) -> (usize, usize) {
    if prior % 2 == 1 {
        (band, (prior - 1) / 2)
    } else {
        (band + 1, prior / 2 - 1)
    }
}

/// Mark the gap slots one vertical or bending route needs.
fn mark_travel(slot: &mut [usize], pu: Cell, pv: Cell) {
    let (b1, b2) = (pu.1.min(pv.1), pu.1.max(pv.1));
    if b1 != b2 {
        for boundary in b1 + 1..=b2 {
            raise(slot, boundary, 1);
        }
    } else if pu.0.abs_diff(pv.0) != 1 {
        // Same band but not adjacent ranks: bend above the band.
        raise(slot, pu.1, 1);
    }
}

/// Assemble the grid rows: `count` gap rows at each required boundary,
/// node bands everywhere else, in band order, plus each band's row-span
/// continuation rows (gently-awd): a multi-row node's `rows` span
/// occupies the grid rows south of its band, so the bands below shift
/// south by the band's tallest row span minus one (upstream places a
/// node's span over the rows south of its band row; empty continuation
/// rows collapse to height 0 in `_prepare_layout`, so they render
/// invisibly — but they must EXIST so a tall node's box does not
/// swallow the following bands' rows). One-celled bands reserve
/// nothing, so plain graphs keep the recorded adjacency.
pub(super) fn assemble_rows(graph: &Graph, placement: &Placement, slots: &[usize]) -> Rows {
    let bands = placement.bands;
    let mut row_of_band = vec![0usize; bands];
    let mut gap_rows: Vec<Vec<usize>> = vec![Vec::new(); bands + 1];
    let mut row = 0usize;
    for b in 0..bands {
        push_gaps(&mut gap_rows[b], slots[b], &mut row);
        row_of_band[b] = row;
        row += 1;
        // Row-span continuation rows (gently-awd): the band's tallest
        // multi-row node spans this many extra grid rows.
        let cy = (0..graph.nodes.len())
            .filter(|&i| placement.cell[i].is_some_and(|c| c.1 == b))
            .map(|i| node_span(graph, i).1)
            .max()
            .unwrap_or(1);
        row += cy - 1;
    }
    push_gaps(&mut gap_rows[bands], slots[bands], &mut row);
    Rows {
        row_of_band,
        gap_rows,
        height: row,
    }
}

/// Append `count` consecutive gap rows to one boundary's list.
fn push_gaps(gaps: &mut Vec<usize>, count: usize, row: &mut usize) {
    for _ in 0..count {
        gaps.push(*row);
        *row += 1;
    }
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
    let mut occupied: HashSet<Cell> = HashSet::new();
    for (&(c, r), &(cx, cy)) in node_cells.iter().zip(spans) {
        for dc in 0..cx {
            for dr in 0..cy {
                occupied.insert((c + dc, r + dr));
            }
        }
    }
    let ctx = Ctx {
        occupied: &occupied,
        max_x,
    };
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(ei, e)| {
            let su = placement.cell[e.from].unwrap();
            let sv = placement.cell[e.to].unwrap();
            edge_path(
                placement.route[ei],
                &Endpoints {
                    su,
                    sv,
                    gu: node_cells[e.from],
                    gv: node_cells[e.to],
                    span_u: spans[e.from],
                    span_v: spans[e.to],
                },
                rows,
                &ctx,
            )
        })
        .collect()
}

/// The cell path of one edge, by travel kind. Straight: the single gap
/// column between adjacent neighbours on one band. Bend: through the gap
/// row above (or below) the band, entering/exiting through the node
/// columns. Multi-band: east-going along the source band to the target
/// column, then vertically through the gap rows (oracle: the diamond's
/// c->d); west-going bends out of the source's north/south first and runs
/// along the gap row adjacent to the target band (oracle: bg-edge-node's
/// Ulm->car — the source-band walk would drive through the nodes between).
fn edge_path(route: Route, ep: &Endpoints, rows: &Rows, ctx: &Ctx) -> Vec<Cell> {
    let (su, gu) = (ep.su, ep.gu);
    let (gv, span_u, span_v) = (ep.gv, ep.span_u, ep.span_v);
    match route {
        Route::Selfloop => selfloop_path(su, gu, rows),
        Route::Parallel(_) => bend_path(
            gu,
            gv,
            span_u,
            span_v,
            bend_gap_row(route, su.1, rows, gu.1),
        ),
        // The port dips run one gap row, all the way between the two node
        // columns — straight through under (or over) everything between.
        Route::PortBelow | Route::PortAbove => {
            let boundary = if route == Route::PortBelow {
                su.1 + 1
            } else {
                su.1
            };
            let g = rows
                .gap_rows
                .get(boundary)
                .and_then(|v| v.first())
                .copied()
                .unwrap_or(gu.1);
            bend_path(gu, gv, span_u, span_v, g)
        }
        Route::Grew | Route::Around => {
            // Straight through the single gap column between the two
            // nodes — east of the source's column span or west of the
            // target's (gently-awd: a multi-celled node's east border
            // sits at `col + cx - 1`, so the gap column shifts east).
            let gap = if gu.0 < gv.0 {
                (gv.0 == gu.0 + span_u.0 + 1).then_some(gu.0 + span_u.0)
            } else {
                (gu.0 == gv.0 + span_v.0 + 1).then_some(gv.0 + span_v.0)
            };
            if gu.1 == gv.1 {
                match gap {
                    Some(c) => vec![(c, gu.1)],
                    None => bend_path(
                        gu,
                        gv,
                        span_u,
                        span_v,
                        bend_gap_row(route, su.1, rows, gu.1),
                    ),
                }
            } else {
                multiband_path(ep, rows, ctx)
            }
        }
    }
}

/// The first gap-row cell directly above the selfloop node.
fn selfloop_path(su: Cell, gu: Cell, rows: &Rows) -> Vec<Cell> {
    match rows.gap_rows[su.1].first().copied() {
        Some(g) => vec![(gu.0, g)],
        None => Vec::new(),
    }
}

/// Band of a same-band bend's gap row: above the band for odd parallel
/// channels (or any non-parallel bend), below it for even channels —
/// each channel takes its own gap row.
fn bend_gap_row(route: Route, band: usize, rows: &Rows, fallback: usize) -> usize {
    let (boundary, k) = match route {
        Route::Parallel(prior) => parallel_channel(prior, band),
        _ => (band, 0),
    };
    rows.gap_rows
        .get(boundary)
        .and_then(|v| v.get(k))
        .copied()
        .unwrap_or(fallback)
}

/// Cell path of a same-band bend through the gap row `g`: out of the
/// source's border column, across, and down/up into the target's
/// border column — the east/west border of a multi-celled node sits at
/// `col + cx - 1` (gently-awd; the anchor column is its west border).
fn bend_path(
    gu: Cell,
    gv: Cell,
    span_u: (usize, usize),
    span_v: (usize, usize),
    g: usize,
) -> Vec<Cell> {
    if gu.0 < gv.0 {
        let out = gu.0 + span_u.0 - 1;
        let mut path = vec![(out, g)];
        path.extend((out + 1..gv.0).map(|c| (c, g)));
        path.push((gv.0, g));
        path
    } else {
        let into = gv.0 + span_v.0 - 1;
        let mut path = vec![(gu.0, g)];
        path.extend((into + 1..gu.0).rev().map(|c| (c, g)));
        path.push((into, g));
        path
    }
}

/// Cell path of a multi-band route. East-going: along the source band to
/// the target column, then the full-row walk down (or up) the target
/// column. West-going: the recorded oracle A* shape (bg-edge-node's
/// Ulm->car) bends out of the source's north/south first — the full-row
/// walk at the source column toward the gap row adjacent to the target
/// band — then runs along that gap row, ending at the target column (the
/// source-band walk would drive through the intervening node cells).
/// Both vertical walks jog east around band rows occupied by a stacked
/// node (gently-4ga) — the gap-only walk jumped the occupied band row
/// between two reserved gap rows, rendering non-adjacent paths.
fn multiband_path(ep: &Endpoints, rows: &Rows, ctx: &Ctx) -> Vec<Cell> {
    let (su, sv) = (ep.su, ep.sv);
    let (gu, gv) = (ep.gu, ep.gv);
    let (span_u, span_v) = (ep.span_u, ep.span_v);
    if gu.0 > gv.0 {
        let north = sv.1 < su.1;
        // The gap row adjacent to the target band hosts the horizontal
        // run; the vertical walk covers every row between the source and
        // that gap row at the source column.
        let home = if north { sv.1 + 1 } else { sv.1 };
        return match rows.gap_rows.get(home).and_then(|v| v.first()) {
            Some(&g) => {
                let mut path = walk_column(gu.0, gu.1, g, ctx);
                path.extend((gv.0 + span_v.0 - 1..=gu.0).rev().map(|c| (c, g)));
                path
            }
            None => Vec::new(),
        };
    }
    let mut path: Vec<Cell> = if gu.0 < gv.0 {
        (gu.0 + span_u.0..=gv.0).map(|c| (c, gu.1)).collect()
    } else {
        Vec::new()
    };
    path.extend(walk_column(gv.0, gu.1, gv.1, ctx));
    path
}

/// Walk one column between two grid rows (exclusive of both), jogging
/// east around rows occupied by a node (gently-4ga): when the walked
/// column's cell is taken, the path shifts to the free gap column east
/// of it — the jog happens on the free row before the blocked one and
/// shifts back on the first free row after, so every step stays
/// grid-adjacent and no node cell is entered. `to > from` walks south,
/// otherwise north. The east gap column always exists for a walked
/// column (a stacked column always has a node one column east —
/// extend_or_below / attach_next_to only stack when the east slot is
/// taken), so `max_x` only guards the eastmost stack, which dodges west
/// instead (odd columns never hold nodes, whatever the side).
fn walk_column(col: usize, from: usize, to: usize, ctx: &Ctx) -> Vec<Cell> {
    let rows: Vec<usize> = if to > from {
        (from + 1..to).collect()
    } else {
        ((to + 1)..from).rev().collect()
    };
    let dodge = if col < ctx.max_x { col + 1 } else { col - 1 };
    let mut path: Vec<Cell> = Vec::new();
    let mut dodging = false;
    for r in rows {
        match (ctx.occupied.contains(&(col, r)), dodging) {
            (true, false) => {
                // Jog east on the previous (free) row, then step past the
                // blocked row in the gap column.
                let prev = path.last().map_or(from, |&(_, y)| y);
                path.push((dodge, prev));
                path.push((dodge, r));
                dodging = true;
            }
            (true, true) => path.push((dodge, r)),
            (false, true) => {
                path.push((dodge, r));
                path.push((col, r));
                dodging = false;
            }
            (false, false) => path.push((col, r)),
        }
    }
    path
}
