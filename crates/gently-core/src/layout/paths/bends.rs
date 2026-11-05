//! Purpose: same-band route realizations — straight gap columns, bend
//! paths through gap rows, selfloops, and the west-going run bridge.
//! Responsibilities: find the single gap column of a column-adjacent
//! pair, stitch orthogonal bends through gap rows, and bridge a
//! `walk_column` result onto a westward run row.
//! Rationale: split from paths.rs past the tidy file limit (gently-cbk).

use super::super::{Cell, Route, Rows};
use super::slots::parallel_channel;
use super::walk::{push_corridor, walk_column, walk_from};
use super::Ctx;

/// The single gap column between two same-band neighbours: east of the
/// source's column span, or west of the target's (`None` when the pair
/// is not adjacent).
pub(super) fn straight_gap(
    gu: Cell,
    gv: Cell,
    span_u: (usize, usize),
    span_v: (usize, usize),
) -> Option<usize> {
    if gu.0 < gv.0 {
        (gv.0 == gu.0 + span_u.0 + 1).then_some(gu.0 + span_u.0)
    } else {
        (gu.0 == gv.0 + span_v.0 + 1).then_some(gv.0 + span_v.0)
    }
}

/// The first gap-row cell directly above the selfloop node.
pub(super) fn selfloop_path(su: Cell, gu: Cell, rows: &Rows) -> Vec<Cell> {
    match rows.gap_rows[su.1].first().copied() {
        Some(g) => vec![(gu.0, g)],
        None => Vec::new(),
    }
}

/// Band of a same-band bend's gap row: above the band for odd parallel
/// channels (or any non-parallel bend), below it for even channels —
/// each channel takes its own gap row.
pub(super) fn bend_gap_row(route: Route, band: usize, rows: &Rows, fallback: usize) -> usize {
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
pub(super) fn bend_path(
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

/// Complete a west-going multiband path: bridge the `walk_column`
/// result onto the run row and append the westward run.
///
/// The walk can end three ways. (a) Un-dodged, one row short of the run
/// row at the walk column — the run's first cell `(gu0, g)` is adjacent;
/// append the run (or close the empty run at the source column, the
/// target's east face lying east of the source, gently-hpd edge 33).
/// (b) Dodged, the terminal close landed ON the run row at the dodge
/// column — walk the corridor across to the run's start column (gap
/// rows hold no node cells, so this is free; validated anyway, gently-0is)
/// and append the run skipping its duplicated first cell. (c) The
/// give-up break stopped the walk mid-column — appending the far-away
/// run would stitch a non-adjacent jump, so return `None`: the caller
/// tries the next candidate, and a last resort truncates (the renderer
/// reports the shape as a typed error).
pub(super) fn west_run_path(
    gu: Cell,
    span_cy: usize,
    run: &[Cell],
    g: usize,
    ctx: &Ctx,
) -> Option<Vec<Cell>> {
    let gu0 = gu.0;
    let path = walk_column(gu0, walk_from(gu.1, span_cy, g), g, ctx);
    let Some(&(dc, dr)) = path.last() else {
        return Some(close_empty_walk(path, run, gu0, g, ctx));
    };
    if dr == g {
        return close_on_run_row(path, dc, run, gu0, g, ctx);
    }
    if dr.abs_diff(g) == 1 && dc == gu0 {
        return Some(append_run(path, run, gu0, g, ctx));
    }
    // The give-up break stopped the walk short of the run row.
    None
}

/// The walk range was empty: the run row is adjacent to the source
/// band, so the run's first cell is adjacent to the source's
/// attachment (gently-0is: edge 68 rendered fine before the helper —
/// replicate that shape).
fn close_empty_walk(
    mut path: Vec<Cell>,
    run: &[Cell],
    gu0: usize,
    g: usize,
    ctx: &Ctx,
) -> Vec<Cell> {
    if run.is_empty() {
        if !ctx.occupied.contains(&(gu0, g)) {
            path.push((gu0, g));
        }
        return path;
    }
    path.extend(run.iter().copied());
    path
}

/// Dodge-column close on the run row. Bridging east to the
/// source column and then running back west would 180°-turn on
/// the spot (gently-0is: edge 1603) — instead continue ALONG
/// the run row toward the run's west end (the target's east-face
/// entry cell). The gap row holds no node cells, so the travel
/// is free whatever the dodge column; cells east of the close
/// are simply not drawn (any valid route is fine at this tier).
fn close_on_run_row(
    mut path: Vec<Cell>,
    dc: usize,
    run: &[Cell],
    gu0: usize,
    g: usize,
    ctx: &Ctx,
) -> Option<Vec<Cell>> {
    match run.last() {
        Some(&(west_end, _)) => {
            bridge_along(&mut path, dc, west_end, g);
            Some(path)
        }
        None => {
            // Empty run (the target's east face lies east of the
            // source, gently-hpd edge 33): bridge the corridor to
            // the source column — the path ends there, so no
            // 180° can follow.
            if dc != gu0 {
                let lo = dc.min(gu0);
                let hi = dc.max(gu0);
                if !(lo..=hi).all(|c| !ctx.occupied.contains(&(c, g))) {
                    return None;
                }
                push_corridor(&mut path, dc, gu0, g);
            }
            Some(path)
        }
    }
}

/// Continue along one row toward the run's west end, from the dodge
/// column `dc` to `west_end` — either direction, never turning back.
fn bridge_along(path: &mut Vec<Cell>, dc: usize, west_end: usize, g: usize) {
    if dc > west_end {
        path.extend((west_end..dc).rev().map(|c| (c, g)));
    } else if dc < west_end {
        path.extend((dc + 1..=west_end).map(|c| (c, g)));
    }
}

/// Un-dodged end one row off the run row at the walk column: append the
/// run (or close the empty run at the source column).
fn append_run(mut path: Vec<Cell>, run: &[Cell], gu0: usize, g: usize, ctx: &Ctx) -> Vec<Cell> {
    if run.is_empty() {
        if !ctx.occupied.contains(&(gu0, g)) {
            path.push((gu0, g));
        }
        return path;
    }
    path.extend(run.iter().copied());
    path
}
