//! Purpose: grid-row assembly — the band→row mapping and the per-
//! boundary gap-row lists the routes walk through.
//! Responsibilities: index the bands' anchored/covering nodes, resolve
//! each band's grid row (cursor or span-ride), and reserve gap rows.
//! Rationale: split from paths.rs past the tidy file limit (gently-cbk).

use super::super::{Placement, Rows};

/// Assemble the grid rows: `count` gap rows at each required boundary,
/// node bands everywhere else, in band order. Bands map onto grid rows
/// in sequence (upstream places nodes at absolute grid rows — a band's
/// row IS its index plus the gap rows inserted before it); a multi-row
/// node's span then covers the grid rows south of its band WITHOUT
/// shifting the following bands (gently-a6g: the fan-in star's east-ring
/// attachments sit inside the grown hub's span rows, so the span must
/// not push them south — upstream's `_prepare_layout` collapses empty
/// continuation rows to height 0, and nodes cleared at Chebyshev
/// distance one never overlap a span). One-celled bands reserve nothing,
/// so plain graphs keep the recorded adjacency; the grid height covers
/// every node's full span extent.
pub(in crate::layout) fn assemble_rows(
    placement: &Placement,
    spans: &[(usize, usize)],
    slots: &[usize],
) -> Rows {
    let bands = placement.bands;
    let (anchored, covering) = band_index(placement, spans, bands);
    let mut row = 0usize;
    let mut ledger = Ledger::new(&anchored, bands);
    for (b, covering) in covering.iter().enumerate() {
        row = assemble_band(placement, spans, covering, slots, b, row, &mut ledger);
    }
    let Ledger {
        anchored: _,
        row_of_band,
        mut gap_rows,
        occupied: _,
    } = ledger;
    gap_rows[bands].extend(gap_ids(row, slots[bands]));
    row += slots[bands];
    let height = grid_height(placement, spans, &row_of_band, row);
    Rows {
        row_of_band,
        gap_rows,
        height,
    }
}

/// The mutable assembly state shared across bands: the anchored index,
/// the resolved rows, the gap rows, and the ride-collision ledger.
struct Ledger<'a> {
    anchored: &'a [Vec<usize>],
    row_of_band: Vec<usize>,
    gap_rows: Vec<Vec<usize>>,
    occupied: Vec<(usize, usize, usize, usize)>,
}

impl<'a> Ledger<'a> {
    /// A fresh ledger over `anchored`: no rows resolved yet.
    fn new(anchored: &'a [Vec<usize>], bands: usize) -> Self {
        Ledger {
            anchored,
            row_of_band: vec![0usize; bands],
            gap_rows: vec![Vec::new(); bands + 1],
            occupied: Vec::new(),
        }
    }
}

/// Resolve one band's grid row and advance the cursor south of every
/// span anchored on it (the cursor never shares a row with a span
/// anchored above it), recording the band's occupied rectangles.
fn assemble_band(
    placement: &Placement,
    spans: &[(usize, usize)],
    covering: &[usize],
    slots: &[usize],
    b: usize,
    row: usize,
    ledger: &mut Ledger<'_>,
) -> usize {
    let mut cursor = row;
    ledger.gap_rows[b].extend(gap_ids(cursor, slots[b]));
    cursor += slots[b];
    let band_row = resolve_band(placement, spans, ledger, covering, slots, b, cursor);
    ledger.row_of_band[b] = band_row;
    let cy = ledger.anchored[b]
        .iter()
        .map(|&i| spans[i].1)
        .max()
        .unwrap_or(1);
    cursor = cursor.max(band_row + cy);
    for &i in &ledger.anchored[b] {
        ledger
            .occupied
            .push(occupied_rect(placement, spans, i, band_row));
    }
    cursor
}

/// One resolved node's occupied rectangle (rank r0, rank r1, row y0,
/// row y1) — the ride-collision ledger entry (gently-0is).
fn occupied_rect(
    placement: &Placement,
    spans: &[(usize, usize)],
    i: usize,
    row: usize,
) -> (usize, usize, usize, usize) {
    let (r, _) = placement.cell[i].unwrap();
    let (cx, cy) = spans[i];
    (r, r + cx.saturating_sub(1), row, row + cy - 1)
}

/// Per-band anchored nodes, and per-band covering nodes (earlier bands'
/// row spans that reach over b) — precomputed once so the walk stays
/// linear (ge.layout.c6).
fn band_index(
    placement: &Placement,
    spans: &[(usize, usize)],
    bands: usize,
) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let anchored: Vec<Vec<usize>> = (0..bands)
        .map(|b| {
            placement
                .cell
                .iter()
                .enumerate()
                .filter_map(|(i, c)| c.filter(|c| c.1 == b).map(|_| i))
                .collect()
        })
        .collect();
    let mut covering: Vec<Vec<usize>> = vec![Vec::new(); bands];
    for (i, c) in placement.cell.iter().enumerate() {
        if let Some((_, bi)) = *c {
            for cover in covering[bi + 1..(bi + spans[i].1).min(bands)].iter_mut() {
                cover.push(i);
            }
        }
    }
    for list in covering.iter_mut() {
        list.sort_by_key(|&i| placement.cell[i].map(|c| c.1).unwrap_or(0));
    }
    (anchored, covering)
}

/// The grid height: south of every node's full span extent (the 10-row
/// so-flowchart-size boxes; the star's 3-row hub), and at least past the
/// row cursor.
fn grid_height(
    placement: &Placement,
    spans: &[(usize, usize)],
    row_of_band: &[usize],
    row: usize,
) -> usize {
    placement
        .cell
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.map(|c| (i, c)))
        .map(|(i, (_, b))| row_of_band[b] + spans[i].1)
        .chain(std::iter::once(row))
        .max()
        .unwrap_or(row)
}

/// A band's grid row: the running cursor (south of every span so far),
/// UNLESS the band's slot ring-rides inside an earlier band's grown span
/// at disjoint columns (gently-a6g) — then it takes the offset row
/// `row_of_band[b'] + (b - b')` (the fan-in star's east-ring attachment
/// rides the hub's bottom span row). Same-column stacking keeps the
/// cursor (the size-attr column stacks south of each tall box).
/// The ride must ALSO be valid (gently-0is): at corpus scale
/// several bands ride earlier spans and their offset rows landed on
/// each other's and on their hosts' cells (two nodes shared the hub's
/// cell, ge.layout c2 violated), and — after collision fallbacks
/// re-packed the cursor — on the boundaries' reserved gap rows or south
/// of the host span itself. The ride is therefore taken only when its
/// whole row extent (a) overlaps no already-resolved node rectangle,
/// (b) touches no reserved gap row, and (c) stays inside the host
/// span's rows; otherwise the band falls back to the cursor — south of
/// every resolved span, always free.
fn resolve_band(
    placement: &Placement,
    spans: &[(usize, usize)],
    ledger: &Ledger<'_>,
    covering: &[usize],
    slots: &[usize],
    b: usize,
    cursor: usize,
) -> usize {
    let ride = covering
        .first()
        .and_then(|&i| ride_candidate(placement, spans, ledger, slots, b, i));
    // (b) No reserved gap row may fall inside the ride's row extent —
    // gap rows are the routing corridors; a band row on one leaves its
    // vertical walks no cell to route through.
    if let Some(ride) = ride {
        if gap_hit(&ledger.gap_rows, ledger.anchored, spans, b, ride) {
            return cursor;
        }
    }
    if let Some(ride) = ride {
        if ride_free(placement, spans, ledger.anchored, b, ride, &ledger.occupied) {
            return ride;
        }
    }
    cursor
}

/// The offset-row ride candidate for band `b` inside host `i`'s grown
/// span: the host must be column-disjoint from every node anchored on
/// `b`, the offset must honour the gap rows reserved on the boundaries
/// it crosses (gently-n0d), and it must stay inside the host span
/// (gently-0is, condition (c)).
#[allow(clippy::too_many_arguments)]
fn ride_candidate(
    placement: &Placement,
    spans: &[(usize, usize)],
    ledger: &Ledger<'_>,
    slots: &[usize],
    b: usize,
    i: usize,
) -> Option<usize> {
    let (_, bi) = placement.cell[i].unwrap();
    // The span hosts band b: column-disjoint from every node
    // anchored on b? (rank space: one rank is two grid columns.)
    if !host_disjoint(placement, spans, ledger.anchored, b, i) {
        return None;
    }
    // The ride offset honours the gap rows reserved on the boundaries
    // it crosses (gently-n0d): a riding band whose vertical travel
    // reserved a gap row above it may not sit grid-adjacent to the band
    // above it, or the walk has no cell to route through.
    let reserved: usize = (bi + 1..=b).map(|k| slots[k]).sum();
    let candidate = ledger.row_of_band[bi] + (b - bi) + reserved;
    // (c) The ride lives INSIDE the host span — a cursor fallback
    // (collision below) re-packs real rows past the formula's
    // one-row-per-band assumption, and an out-of-span candidate is its
    // signature.
    let host_bottom = ledger.row_of_band[bi] + spans[i].1;
    (candidate < host_bottom).then_some(candidate)
}

/// Is host `i` column-disjoint from every node anchored on band `b`?
fn host_disjoint(
    placement: &Placement,
    spans: &[(usize, usize)],
    anchored: &[Vec<usize>],
    b: usize,
    i: usize,
) -> bool {
    let (ri, _) = placement.cell[i].unwrap();
    anchored[b].iter().all(|&j| {
        let (rj, _) = placement.cell[j].unwrap();
        rj + spans[j].0 <= ri || ri + spans[i].0 <= rj
    })
}

/// Does any reserved gap row fall inside the ride's row extent?
fn gap_hit(
    gap_rows: &[Vec<usize>],
    anchored: &[Vec<usize>],
    spans: &[(usize, usize)],
    b: usize,
    ride: usize,
) -> bool {
    let max_cy = anchored[b].iter().map(|&i| spans[i].1).max().unwrap_or(1);
    gap_rows[..(b + 1).min(gap_rows.len())]
        .iter()
        .flatten()
        .any(|&g| g >= ride && g < ride + max_cy)
}

/// Does the ride's whole row extent overlap no already-resolved node
/// rectangle? (gently-0is, condition (a))
#[allow(clippy::too_many_arguments)]
fn ride_free(
    placement: &Placement,
    spans: &[(usize, usize)],
    anchored: &[Vec<usize>],
    b: usize,
    ride: usize,
    occupied: &[(usize, usize, usize, usize)],
) -> bool {
    anchored[b].iter().all(|&j| {
        let (rj, _) = placement.cell[j].unwrap();
        let (cx, cy) = spans[j];
        occupied.iter().all(|&(r0, r1, y0, y1)| {
            rj > r1 || r0 > rj + cx.saturating_sub(1) || ride > y1 || y0 > ride + cy - 1
        })
    })
}

/// The `count` consecutive gap-row ids starting at `start`.
fn gap_ids(start: usize, count: usize) -> impl Iterator<Item = usize> {
    start..start + count
}
