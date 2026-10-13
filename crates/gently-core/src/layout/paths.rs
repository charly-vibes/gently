//! Purpose: the edge-path realization half of the layout — gap-slot
//! accounting, grid-row assembly, and each edge's routed cell path by
//! travel kind (split from layout.rs past the tidy file limit,
//! gently-cbk).
//! Responsibilities: count the gap rows each boundary needs, assemble the
//! band/gap row sequence, and realize every route as an orthogonal cell
//! path — straight, bend, port dip, parallel channel, selfloop, and the
//! multi-band walks (east-going along the source band; west-going bending
//! out of the source's north/south first, gently-cbk).
//! Rationale: shaped by the recorded v0.69 @ ededa3d7 companions —
//! ge.layout.c1 (pure function) and c4 (orthogonal routes).

use super::{Cell, Placement, Route, Rows};
use crate::graph::Graph;

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
/// node bands everywhere else, in band order.
pub(super) fn assemble_rows(bands: usize, slots: &[usize]) -> Rows {
    let mut row_of_band = vec![0usize; bands];
    let mut gap_rows: Vec<Vec<usize>> = vec![Vec::new(); bands + 1];
    let mut row = 0usize;
    for b in 0..bands {
        push_gaps(&mut gap_rows[b], slots[b], &mut row);
        row_of_band[b] = row;
        row += 1;
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
pub(super) fn route_edges(
    graph: &Graph,
    placement: &Placement,
    node_cells: &[Cell],
    rows: &Rows,
) -> Vec<Vec<Cell>> {
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(ei, e)| {
            let su = placement.cell[e.from].unwrap();
            let sv = placement.cell[e.to].unwrap();
            edge_path(
                placement.route[ei],
                su,
                sv,
                node_cells[e.from],
                node_cells[e.to],
                rows,
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
fn edge_path(route: Route, su: Cell, sv: Cell, gu: Cell, gv: Cell, rows: &Rows) -> Vec<Cell> {
    match route {
        Route::Selfloop => selfloop_path(su, gu, rows),
        Route::Parallel(_) => bend_path(gu, gv, bend_gap_row(route, su.1, rows, gu.1)),
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
            bend_path(gu, gv, g)
        }
        Route::Grew | Route::Around => {
            if gu.1 == gv.1 && gu.0.abs_diff(gv.0) == 2 {
                vec![(gu.0.min(gv.0) + 1, gu.1)]
            } else if gu.1 == gv.1 {
                bend_path(gu, gv, bend_gap_row(route, su.1, rows, gu.1))
            } else {
                multiband_path(su, sv, gu, gv, rows)
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
/// source's column, across, and down/up into the target's column.
fn bend_path(gu: Cell, gv: Cell, g: usize) -> Vec<Cell> {
    let mut path = vec![(gu.0, g)];
    if gu.0 < gv.0 {
        path.extend((gu.0 + 1..gv.0).map(|c| (c, g)));
    } else {
        path.extend((gv.0 + 1..gu.0).rev().map(|c| (c, g)));
    }
    path.push((gv.0, g));
    path
}

/// Cell path of a multi-band route. East-going: along the source band to
/// the target column, then vertically through the gap rows between the
/// bands. West-going: the recorded oracle A* shape (bg-edge-node's
/// Ulm->car) bends out of the source's north/south first — vertically
/// through the gap rows at the source column — then runs along the gap
/// row adjacent to the target band, ending at the target column (the
/// source-band walk would drive through the intervening node cells).
fn multiband_path(su: Cell, sv: Cell, gu: Cell, gv: Cell, rows: &Rows) -> Vec<Cell> {
    if gu.0 > gv.0 {
        let north = sv.1 < su.1;
        // The gap rows between the bands, walked from the source side; the
        // one adjacent to the target band hosts the horizontal run.
        let walked: Vec<usize> = if north {
            (sv.1 + 2..=su.1).rev().collect()
        } else {
            (su.1 + 1..sv.1).collect()
        };
        let home = if north { sv.1 + 1 } else { sv.1 };
        let mut path: Vec<Cell> = walked
            .iter()
            .filter_map(|&k| rows.gap_rows.get(k)?.first().map(|&g| (gu.0, g)))
            .collect();
        if let Some(&g) = rows.gap_rows.get(home).and_then(|v| v.first()) {
            path.extend((gv.0..=gu.0).rev().map(|c| (c, g)));
        }
        return path;
    }
    let mut path: Vec<Cell> = if gu.0 < gv.0 {
        (gu.0 + 1..=gv.0).map(|c| (c, gu.1)).collect()
    } else {
        Vec::new()
    };
    let (b1, b2) = (su.1.min(sv.1), su.1.max(sv.1));
    let mut slots: Vec<usize> = (b1 + 1..=b2).collect();
    if su.1 > sv.1 {
        slots.reverse(); // walk the gaps from the source side
    }
    path.extend(
        slots
            .into_iter()
            .filter_map(|k| rows.gap_rows[k].first().map(|&g| (gv.0, g))),
    );
    path
}
