//! Purpose: box side detection and attachment points for polylines —
//! exit/entry directions, one-char-off border attachment points, and the
//! typed unsupported-side error.
//! Rationale: split from polyline.rs past the tidy file limit (gently-0of).

use super::super::canvas::Canvas;
use super::super::RenderError;
use super::{Ctx, Dir, Pt};
use crate::layout::Cell;

/// The start arrowhead glyph for an edge leaving its source box through
/// `dir`: the arrow points INTO the box (gently-7qz, pinned by
/// tests/repro/claims/pod-bidir-parallel.observed — the bent bidirectional
/// edge leaves north and draws `v` at the source's north attachment).
pub(super) fn start_glyph(exit: Dir) -> char {
    match exit {
        Dir::East => '<',
        Dir::West => '>',
        Dir::North => 'v',
        Dir::South => '^',
    }
}

pub(super) fn exit_dir(ctx: &Ctx, first: Cell, sc: usize, sr: usize) -> Result<Dir, RenderError> {
    // A multi-celled source is left through any column of its span
    // (gently-awd): east of the anchor leaves east, west leaves west,
    // same-column rows leave north/south. An INDEXED port slot leaves
    // through the span's east side on the slot's continuation row
    // (gently-byq): the first path cell sits strictly east of the span,
    // on a row inside the source's row span below the band.
    let span_c = ctx.span_s.0;
    if first.1 == sr {
        match first.0.cmp(&sc) {
            std::cmp::Ordering::Greater => Ok(Dir::East),
            std::cmp::Ordering::Less => Ok(Dir::West),
            std::cmp::Ordering::Equal => Err(unsupported_side(ctx, "leaves")),
        }
    } else if first.0 >= sc + span_c && first.1 > sr && first.1 < sr + ctx.span_s.1.max(1) {
        Ok(Dir::East)
    } else if first.0 < sc && first.1 > sr && first.1 < sr + ctx.span_s.1.max(1) {
        // The span's WEST side on a continuation row (gently-0is, the
        // exit mirror of the entry's west-side rule, gently-206): the
        // corpus's straight gap cell rides the source's span rows west
        // of the node — moving east into the source's west face.
        Ok(Dir::West)
    } else if first.0 >= sc && first.0 < sc + span_c {
        Ok(if first.1 < sr { Dir::North } else { Dir::South })
    } else {
        Err(unsupported_side(ctx, "leaves"))
    }
}

pub(super) fn entry_dir(ctx: &Ctx, last: Cell) -> Result<(Dir, char), RenderError> {
    let (tc, tr) = (ctx.tc, ctx.tr);
    let span_c = ctx.span_t.0;
    if last.1 == tr {
        match last.0.cmp(&tc) {
            std::cmp::Ordering::Less => Ok((Dir::East, '>')),
            std::cmp::Ordering::Greater => Ok((Dir::West, '<')),
            std::cmp::Ordering::Equal => Err(unsupported_side(ctx, "enters")),
        }
    } else if last.0 >= tc + span_c && last.1 > tr && last.1 < tr + ctx.span_t.1.max(1) {
        Ok((Dir::West, '<'))
    } else if last.0 < tc && last.1 > tr && last.1 < tr + ctx.span_t.1.max(1) {
        Ok((Dir::East, '>'))
    } else if last.0 >= tc && last.0 < tc + span_c {
        Ok(if last.1 < tr {
            (Dir::South, 'v')
        } else {
            (Dir::North, '^')
        })
    } else {
        Err(unsupported_side(ctx, "enters"))
    }
}

pub(super) fn exit_attachment(
    canvas: &Canvas,
    dir: Dir,
    c: usize,
    r: usize,
    span: (usize, usize),
    ei: usize,
    exit_row: usize,
) -> Result<Pt, RenderError> {
    let pt = match dir {
        // The east border of a multi-column node sits one char west of
        // the column after its span (gently-awd); the attachment line is
        // the exit cell's row band's middle line — the band's own for a
        // band-row exit, the slot's continuation row for an indexed-port
        // slot exit (gently-byq: line `band_top + 3N + 1`).
        Dir::East => (Some(canvas.offsets[c + span.0]), Some(canvas.mid(exit_row))),
        Dir::West => (canvas.offsets[c].checked_sub(1), Some(canvas.mid(exit_row))),
        Dir::North => (Some(canvas.vert_x(c)), canvas.band_top(r).checked_sub(1)),
        Dir::South => (Some(canvas.vert_x(c)), Some(canvas.box_below(r, span.1))),
    };
    attachment(canvas, pt, ei, "leaves")
}

pub(super) struct EntrySide {
    pub(super) c: usize,
    pub(super) r: usize,
    pub(super) span: (usize, usize),
    pub(super) row: usize,
    pub(super) shift: usize,
}

pub(super) fn entry_attachment(
    canvas: &Canvas,
    dir: Dir,
    s: &EntrySide,
    ei: usize,
) -> Result<Pt, RenderError> {
    let (c, r, span) = (s.c, s.r, s.span);
    let pt = match dir {
        // The attachment line is the entry cell's row band's middle line
        // (gently-byq: an indexed-port slot entry lands on the slot's
        // continuation row, line `band_top + 3N + 1`).
        Dir::East => (canvas.offsets[c].checked_sub(1), Some(canvas.mid(s.row))),
        Dir::West => (
            Some(canvas.offsets[c + span.0] + s.shift),
            Some(canvas.mid(s.row)),
        ),
        Dir::South => (Some(canvas.vert_x(c)), canvas.band_top(r).checked_sub(1)),
        Dir::North => (Some(canvas.vert_x(c)), Some(canvas.box_below(r, span.1))),
    };
    attachment(canvas, pt, ei, "enters")
}

pub(super) fn attachment(
    canvas: &Canvas,
    pt: (Option<usize>, Option<usize>),
    ei: usize,
    verb: &str,
) -> Result<Pt, RenderError> {
    let (Some(x), Some(y)) = pt else {
        return Err(RenderError::unsupported(format!(
            "edge {ei} {verb} its box through an unreachable side"
        )));
    };
    if y >= canvas.grid.len() || x >= canvas.grid[0].len() {
        return Err(RenderError::unsupported(format!(
            "edge {ei} {verb} its box outside the grid at char {x},{y}"
        )));
    }
    Ok((x, y))
}

pub(super) fn waypoints(
    canvas: &Canvas,
    ctx: &Ctx,
    exit: Dir,
    entry: Dir,
    a0: Pt,
    a_end: Pt,
) -> Result<Vec<Pt>, RenderError> {
    let mut wps = vec![a0];
    let mut dir = exit;
    for (i, &cell) in ctx.path.iter().enumerate() {
        let next_dir = match ctx.path.get(i + 1) {
            Some(&next) => dir_between(cell, next, ctx.edge_index)?,
            None => entry,
        };
        if next_dir != dir {
            super::push_corner(
                canvas,
                ctx,
                &mut wps,
                dir,
                next_dir,
                ctx.path.get(i + 1).copied(),
            )?;
        }
        dir = next_dir;
    }
    wps.push(a_end);
    Ok(wps)
}

pub(super) fn dir_between(p: Cell, q: Cell, ei: usize) -> Result<Dir, RenderError> {
    match (q.0 as isize - p.0 as isize, q.1 as isize - p.1 as isize) {
        (1, 0) => Ok(Dir::East),
        (-1, 0) => Ok(Dir::West),
        (0, 1) => Ok(Dir::South),
        (0, -1) => Ok(Dir::North),
        _ => Err(RenderError::unsupported(format!(
            "edge {ei} path cells {p:?} -> {q:?} are not orthogonally adjacent"
        ))),
    }
}

pub(super) fn unsupported_side(ctx: &Ctx, verb: &str) -> RenderError {
    RenderError::unsupported(format!(
        "edge {} {verb} its box through an unsupported side",
        ctx.edge_index
    ))
}
