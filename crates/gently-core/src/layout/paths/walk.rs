//! Purpose: vertical walks — the column walk every multi-band route
//! uses, jogging east around occupied rows.
//! Responsibilities: walk one column between two rows, pick free jog
//! lanes, and return to the walked column once the wall ends.
//! Rationale: split from paths.rs past the tidy file limit (gently-cbk).

use super::super::Cell;
use super::Ctx;

/// The first row a south-going vertical walk may occupy: below the
/// source's own row span (the band row is the span's TOP row, so the
/// span's cells sit between the band row and the walk's far row).
/// Walking from the band row would jog around the source's own cells —
/// and at the grid's west edge there is no lane to jog to, so the walk
/// breaks empty (gently-0is: edge 68, a same-column south edge from a
/// 3-row source at column 0). Only a walk whose far row lies strictly
/// below the span is adjusted; a rider inside the span keeps the band
/// row (the pinned tier-1 shapes).
pub(super) fn walk_from(from: usize, span_cy: usize, to: usize) -> usize {
    if to > from && to >= from + span_cy.max(1) {
        from + span_cy.max(1) - 1
    } else {
        from
    }
}

/// Push the horizontal corridor between two columns at one row: every
/// column strictly between the current column and the target, plus the
/// target itself. A WIDE jog lane (k > 1) validated by `walk_column`'s
/// `lanes` skips intermediate columns — without this run the path
/// stitched non-adjacent pairs at every wide jog (gently-0is: edges
/// 0/1/7 of the whole-corpus projection). The lane's validation already
/// checked the whole corridor free at that row; callers bridging onto a
/// row without that validation must check the corridor themselves.
pub(super) fn push_corridor(path: &mut Vec<Cell>, from_col: usize, to_col: usize, row: usize) {
    if to_col > from_col {
        path.extend((from_col + 1..=to_col).map(|c| (c, row)));
    } else if to_col < from_col {
        path.extend((to_col..from_col).rev().map(|c| (c, row)));
    }
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
pub(super) fn walk_column(col: usize, from: usize, to: usize, ctx: &Ctx) -> Vec<Cell> {
    let rows = walk_rows(from, to);
    let (mut path, broke, dodging) = walk_rows_all(col, from, &rows, ctx);
    terminal_close(&mut path, to, dodging, broke, ctx);
    path
}

/// The interior rows the walk passes through (exclusive of both ends).
fn walk_rows(from: usize, to: usize) -> Vec<usize> {
    if to > from {
        (from + 1..to).collect()
    } else {
        ((to + 1)..from).rev().collect()
    }
}

/// Walk every interior row, jogging around occupied ones. Returns the
/// path, whether the walk gave up mid-column, and the open dodge
/// column. The jog lane: a free column beside the walked one — the east
/// gap column is the oracle's lane (odd columns never hold BASE cells),
/// but a grown span can cover it (gently-0is), so the lane is validated
/// per stretch and falls back west when east is taken.
fn walk_rows_all(
    col: usize,
    from: usize,
    rows: &[usize],
    ctx: &Ctx,
) -> (Vec<Cell>, bool, Option<usize>) {
    let mut path: Vec<Cell> = Vec::new();
    let mut dodging: Option<usize> = None;
    for (i, &r) in rows.iter().enumerate() {
        if !walk_step(
            &mut path,
            col,
            from,
            rows.get(i + 1).copied(),
            r,
            ctx,
            &mut dodging,
        ) {
            return (path, true, dodging);
        }
    }
    (path, false, dodging)
}

/// One row of the walk, by occupancy and dodge state. Returns false
/// when no free lane exists (the walk stops rather than pushing
/// through a node).
#[allow(clippy::too_many_arguments)]
fn walk_step(
    path: &mut Vec<Cell>,
    col: usize,
    from: usize,
    next_row: Option<usize>,
    r: usize,
    ctx: &Ctx,
    dodging: &mut Option<usize>,
) -> bool {
    match (ctx.occupied.contains(&(col, r)), *dodging) {
        (true, None) => start_jog(path, col, from, r, ctx, dodging),
        (true, Some(d)) => continue_jog(path, col, d, from, r, ctx, dodging),
        (false, Some(d)) => {
            dodge_step(path, col, d, next_row, r, ctx, dodging);
            true
        }
        (false, None) => {
            path.push((col, r));
            true
        }
    }
}

/// Start a jog: the walked column's cell is taken on row `r`. Jog east
/// on the previous (free) row, then step past the blocked row in the
/// gap column. Returns false when no free lane exists (the walk stops
/// rather than pushing through a node — the renderer reports the
/// truncated path as a typed error, gently-0is).
fn start_jog(
    path: &mut Vec<Cell>,
    col: usize,
    from: usize,
    r: usize,
    ctx: &Ctx,
    dodging: &mut Option<usize>,
) -> bool {
    let prev = path.last().map_or(from, |&(_, y)| y);
    match free_lane(col, prev, r, ctx) {
        Some(d) => {
            push_corridor(path, col, d, prev);
            path.push((d, r));
            *dodging = Some(d);
            true
        }
        None => false,
    }
}

/// The dodge column itself is taken further down: switch to another
/// free column beside the lane at this row. Returns false when no lane
/// exists (give up, gently-0is).
fn continue_jog(
    path: &mut Vec<Cell>,
    col: usize,
    d: usize,
    from: usize,
    r: usize,
    ctx: &Ctx,
    dodging: &mut Option<usize>,
) -> bool {
    if !ctx.occupied.contains(&(d, r)) {
        path.push((d, r));
        return true;
    }
    let prev = path.last().map_or(from, |&(_, y)| y);
    match free_lane(d, prev, r, ctx) {
        Some(d2) if d2 != col || !ctx.occupied.contains(&(col, r)) => {
            push_corridor(path, d, d2, prev);
            path.push((d2, r));
            *dodging = Some(d2);
            true
        }
        _ => false,
    }
}

/// A free row in the dodge column: push it, and return to the walked
/// column once the NEXT row is free too. Staying in the dodge column
/// while the next row is blocked keeps the fan-in star's below-hub
/// stack from stepping back east and reversing the path (gently-206) —
/// the renderer rejects a 180° turn as 'turns without changing axis'.
fn dodge_step(
    path: &mut Vec<Cell>,
    col: usize,
    d: usize,
    next_row: Option<usize>,
    r: usize,
    ctx: &Ctx,
    dodging: &mut Option<usize>,
) {
    path.push((d, r));
    let next_blocked = next_row.is_some_and(|nr| ctx.occupied.contains(&(col, nr)));
    if next_blocked {
        return;
    }
    // The return run crosses back through the corridor
    // between the dodge column and the walk column. A
    // WIDE lane (k > 1) has intermediate columns — they
    // must be pushed, and they must be free AT THIS ROW
    // (the lane's validation covered only the jog-start
    // row; a grown span can wall the corridor off down
    // the walk — gently-0is: the (1,63)->(4,63) stitch).
    // A blocked corridor keeps the walk in the dodge
    // column; the return is retried on a later free row.
    let lo = d.min(col);
    let hi = d.max(col);
    if (lo..=hi).all(|c| !ctx.occupied.contains(&(c, r))) {
        push_corridor(path, d, col, r);
        *dodging = None;
    }
}

/// Close the walk's terminal row when the walk ended in the dodge
/// column (a stacked multi-row node fills the walked column down to
/// the face — cli.t6 under cli.c5, the corpus's e19 shape): step into
/// the face row itself so the endpoint stays grid-adjacent to the face
/// cell — the edge enters through the dodge-column side (east face),
/// never through the blocked face (gently-206). Only close when the
/// walk actually reached the terminal row — a give-up break leaves the
/// path mid-column, and a close here would jump (gently-0is).
fn terminal_close(path: &mut Vec<Cell>, to: usize, dodging: Option<usize>, broke: bool, ctx: &Ctx) {
    if !broke {
        if let Some(d) = dodging {
            if !ctx.occupied.contains(&(d, to)) {
                path.push((d, to));
            }
        }
    }
}

/// The free jog lane beside `from_col` for a walk passing from row
/// `prev` onto blocked row `r`. The oracle's lane (one east) first,
/// then west, then wider — grown spans can wall off several columns
/// (gently-0is). A wider lane is valid only when the whole horizontal
/// corridor at BOTH rows is free: the path walks from (from_col, prev)
/// to (d, prev) through every column between. The scan is unbounded
/// (to the grid edges): a corpus-scale wall can exceed any fixed lane
/// count, and the corridor check keeps every wider candidate honest
/// (gently-0is: the whole-corpus projection's row 99).
fn free_lane(from_col: usize, prev: usize, r: usize, ctx: &Ctx) -> Option<usize> {
    (1..=ctx.max_x + 1).find_map(|k| {
        let east = from_col.checked_add(k).filter(|&d| d <= ctx.max_x);
        let west = from_col.checked_sub(k);
        [east, west].into_iter().flatten().find(|&d| {
            let lo = d.min(from_col);
            let hi = d.max(from_col);
            (lo..=hi).all(|c| !ctx.occupied.contains(&(c, prev))) && !ctx.occupied.contains(&(d, r))
        })
    })
}
