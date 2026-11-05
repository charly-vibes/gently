//! Purpose: gap-slot accounting — how many gap rows each boundary needs
//! before the grid rows are assembled.
//! Responsibilities: per-edge slot marking by travel kind, the parallel
//! channel arithmetic, and the span-aware corridor-clear skip.
//! Rationale: split from paths.rs past the tidy file limit (gently-cbk).

use super::super::{Cell, Placement, Route};

/// How many gap rows each boundary needs (boundary `k` is the gap above
/// band `k`; `k == bands` trails the last band). Only edges that route
/// vertically or bend need one; a plain chain row stays gap-free (oracle:
/// mixed_isolated). Parallel multi-edges take one gap per extra channel.
/// Span-aware (gently-a6g): a vertical travel whose endpoints' row spans
/// overlap needs no gap at all (the fan-in star's east-row attachment —
/// the hub's grown span hosts the attachment row), and one whose
/// intermediate band rows are node-free at the walked column routes
/// through those rows (the star's below-band stem rides the empty
/// band between hub and leaf).
pub(in crate::layout) fn required_slots(
    graph: &crate::graph::Graph,
    placement: &Placement,
    spans: &[(usize, usize)],
) -> Vec<usize> {
    let mut slot = vec![0usize; placement.bands + 1];
    for (ei, e) in graph.edges.iter().enumerate() {
        let pu = placement.cell[e.from].unwrap();
        let pv = placement.cell[e.to].unwrap();
        mark_slot(
            &mut slot,
            placement.route[ei],
            pu,
            pv,
            placement,
            spans,
            spans[e.from],
            spans[e.to],
        );
    }
    slot
}

/// A vertical travel needs no gap rows when its endpoints' band-row
/// spans leave a clear corridor: either the spans overlap (the attachment
/// row is hosted by the other node's grown span — the fan-in star's east
/// ring), or the band rows strictly between the spans are node-free at
/// BOTH endpoints' columns (the star's below-band stem rides the empty
/// band between hub and leaf). Blocked corridors keep the packed
/// stack's gap rows (the fan-out's below-attachments).
fn corridor_clear(
    placement: &Placement,
    spans: &[(usize, usize)],
    pu: Cell,
    span_u: (usize, usize),
    pv: Cell,
    span_v: (usize, usize),
) -> bool {
    // Overlapping spans host each other's attachment rows outright.
    let (u0, u1) = (pu.1, pu.1 + span_u.1 - 1);
    let (v0, v1) = (pv.1, pv.1 + span_v.1 - 1);
    if u0 <= v1 && v0 <= u1 {
        return true;
    }
    let upper_end = u1.min(v1);
    let lower_start = u0.max(v0);
    // Adjacent extents (empty corridor) keep the packed stack's gap row.
    if upper_end + 1 >= lower_start {
        return false;
    }
    (upper_end + 1..lower_start)
        .all(|b| cell_free(placement, spans, pu.0, b) && cell_free(placement, spans, pv.0, b))
}

/// Is the grid cell at (`rank`, `band`) covered by any node's span?
fn cell_free(placement: &Placement, spans: &[(usize, usize)], rank: usize, band: usize) -> bool {
    !placement
        .cell
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.map(|c| (i, c)))
        .any(|(i, (r, b))| r <= rank && rank < r + spans[i].0 && b <= band && band < b + spans[i].1)
}

/// Raise a boundary's gap count to `count` when lower.
fn raise(slot: &mut [usize], boundary: usize, count: usize) {
    if slot[boundary] < count {
        slot[boundary] = count;
    }
}

/// Mark the gap slots one edge needs.
#[allow(clippy::too_many_arguments)]
fn mark_slot(
    slot: &mut [usize],
    route: Route,
    pu: Cell,
    pv: Cell,
    placement: &Placement,
    spans: &[(usize, usize)],
    span_u: (usize, usize),
    span_v: (usize, usize),
) {
    match route {
        Route::Selfloop => raise(slot, pu.1, 1),
        // A slot route rides the slotted node's own continuation rows
        // (grown to `N+1` by the port, gently-byq) — no gap rows.
        Route::Slot { .. } => {}
        // A south-port dip takes the first gap row below the band; a
        // north-port bend the first gap row above it.
        Route::PortBelow => raise(slot, pu.1 + 1, 1),
        Route::PortAbove => raise(slot, pu.1, 1),
        Route::Parallel(prior) => {
            let (boundary, k) = parallel_channel(prior, pu.1);
            raise(slot, boundary, k + 1);
        }
        // A stacked channel rides the boxes' row-span continuation rows
        // (gently-awd) — no gap rows; the span reserves them.
        Route::Stacked(_) => {}
        Route::Grew | Route::Around => mark_travel(slot, pu, pv, placement, spans, span_u, span_v),
    }
}

/// The boundary and 0-based gap index a parallel edge with `prior` prior
/// edges on its pair travels: odd priors bend above the band (first,
/// second, ... gap above), even priors below it — channels 1/2 keep the
/// recorded two-parallel oracle shape (above, then below).
pub(super) fn parallel_channel(prior: usize, band: usize) -> (usize, usize) {
    if prior % 2 == 1 {
        (band, (prior - 1) / 2)
    } else {
        (band + 1, prior / 2 - 1)
    }
}

/// Mark the gap slots one vertical or bending route needs.
fn mark_travel(
    slot: &mut [usize],
    pu: Cell,
    pv: Cell,
    placement: &Placement,
    spans: &[(usize, usize)],
    span_u: (usize, usize),
    span_v: (usize, usize),
) {
    let (b1, b2) = (pu.1.min(pv.1), pu.1.max(pv.1));
    if b1 != b2 {
        // Span-aware skip (gently-a6g): a clear corridor needs no gap
        // rows — the walk rides the existing band rows.
        if corridor_clear(placement, spans, pu, span_u, pv, span_v) {
            return;
        }
        for boundary in b1 + 1..=b2 {
            raise(slot, boundary, 1);
        }
    } else if pu.0.abs_diff(pv.0) != 1 {
        // Same band but not adjacent ranks: bend above the band.
        raise(slot, pu.1, 1);
    }
}
