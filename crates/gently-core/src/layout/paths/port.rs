//! Purpose: indexed-port slot edge paths (gently-byq, the decoded
//! upstream routes) — start-slot rides east then north, end-slot drops
//! south then west, with the east channel dodge.
//! Rationale: split from multiband.rs past the tidy file limit.

use super::super::Cell;
use super::{Ctx, Endpoints};
use std::collections::HashSet;

/// The cell path of an indexed-port slot edge (gently-byq, the decoded
/// upstream routes). `end == false` (a start slot `N`): the horizontal
/// ride at grid row `band + N` from the first gap column east of the
/// source's span to the target's column, then the vertical north at the
/// target column down-to-up into the target's south — the arrow-end `^`
/// cell sits on the first grid row south of the target box (its span's
/// bottom row + 1), `band + cy_v`. `end == true` (an end slot `N`): the
/// attachment row is `target_band + N`; the route drops south from the
/// source at the source column (rows `src_band + cy_s ..= att_row`),
/// then runs west at the attachment row into the target's east — WHEN
/// the drop cells are free; if another edge's path already took one, the
/// route dodges EAST instead: out of the source's east side at the band
/// row through the east-gap channel column, down to the attachment row,
/// then west (the SO fixture's shape, where e2's vertical occupies the
/// drop column).
pub(super) fn slot_path(
    ep: &Endpoints,
    ctx: &Ctx,
    used: &HashSet<Cell>,
    end: bool,
    n: usize,
) -> Vec<Cell> {
    if !end {
        start_slot_path(ep, n)
    } else {
        end_slot_path(ep, ctx, used, n)
    }
}

/// (B) start slot: east at the slot row, then north into the
/// target's south. The first cell is the gap column east of the
/// source's span (the node's own span covers the slot row).
fn start_slot_path(ep: &Endpoints, n: usize) -> Vec<Cell> {
    let (gu, gv) = (ep.gu, ep.gv);
    let att_row = gu.1 + n;
    let out = gu.0 + ep.span_u.0;
    let low = gv.1 + ep.span_v.1;
    (out..=gv.0)
        .map(|c| (c, att_row))
        .chain((low..att_row).rev().map(|r| (gv.0, r)))
        .collect()
}

/// (C) end slot: drop south at the source column, then west — dodging
/// EAST through the channel column when another edge's path already
/// took a drop cell (the SO fixture's shape, where e2's vertical
/// occupies the drop column).
fn end_slot_path(ep: &Endpoints, ctx: &Ctx, used: &HashSet<Cell>, n: usize) -> Vec<Cell> {
    let (gu, gv) = (ep.gu, ep.gv);
    let att_row = gv.1 + n;
    let top = gu.1 + ep.span_u.1;
    let drop: Vec<Cell> = if top <= att_row {
        (top..=att_row).map(|r| (gu.0, r)).collect()
    } else {
        vec![(gu.0, att_row)]
    };
    let free = drop
        .iter()
        .all(|c| !ctx.occupied.contains(c) && !used.contains(c));
    if free {
        free_end_slot(gu.0, att_row, gv, ep.span_v.0, drop)
    } else {
        dodged_end_slot(gu, att_row, gv, ep.span_u.0, ep.span_v.0)
    }
}

/// The free end-slot run: south drop at the source column, then west
/// along the attachment row into the target's east.
fn free_end_slot(
    gu_x: usize,
    att_row: usize,
    gv: Cell,
    span_cx: usize,
    drop: Vec<Cell>,
) -> Vec<Cell> {
    let east = gv.0 + span_cx;
    drop.into_iter()
        .chain((east..gu_x).rev().map(|c| (c, att_row)))
        .collect()
}

/// The dodged end-slot route: out of the source's east at the band row
/// through the east-gap channel column, down to the attachment row,
/// then west.
fn dodged_end_slot(
    gu: Cell,
    att_row: usize,
    gv: Cell,
    span_cx: usize,
    span_vx: usize,
) -> Vec<Cell> {
    let channel = gu.0 + span_cx;
    let east = gv.0 + span_vx;
    ((gu.1)..=att_row)
        .map(|r| (channel, r))
        .chain((east..channel).rev().map(|c| (c, att_row)))
        .collect()
}
