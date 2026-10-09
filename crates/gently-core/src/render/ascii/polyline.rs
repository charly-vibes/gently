//! Purpose: polyline drawing for bent and multi-band edges of the ascii
//! renderer (gently-0of).
//! Responsibilities: turn an edge's routed cell path into a char polyline —
//! attachment points one char/line off the source/target box borders, `+`
//! corners where the run direction turns, dashes with a one-char space pad
//! at box-attached horizontal ends, pipes on vertical runs, and the
//! arrowhead (`v`/`^`/`>`/`<`) at the final approach to the target box.
//! Rationale: extracted from the monolithic renderer so each phase stays
//! under the tidy gates; geometry shaped by the recorded v0.69 @ ededa3d7
//! companions (the parallel elbow, the diamond's verticals and c->d route).

use super::{Canvas, RenderError, BAND_HEIGHT};
use crate::layout::{Cell, Layout};

/// Direction of travel along an edge polyline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    East,
    West,
    North,
    South,
}

/// A point in char space: `(x, line)`.
type Pt = (usize, usize);

/// The polyline-drawing context for one edge: its routed cell path, the
/// target cell it must reach, and its model index (for typed errors).
struct Ctx<'a> {
    path: &'a [Cell],
    tc: usize,
    tr: usize,
    edge_index: usize,
}

/// Draw one bent or multi-band edge as a char polyline through its routed
/// path cells.
pub(super) fn draw(
    canvas: &mut Canvas,
    layout: &Layout,
    edge_index: usize,
    sc: usize,
    sr: usize,
    tc: usize,
    tr: usize,
) -> Result<(), RenderError> {
    let ctx = Ctx {
        path: &layout.edge_paths[edge_index],
        tc,
        tr,
        edge_index,
    };
    let Some(&first) = ctx.path.first() else {
        return Err(RenderError::unsupported(format!(
            "edge {edge_index} has an empty path"
        )));
    };
    let last = *ctx.path.last().expect("path has a first cell");
    let exit = exit_dir(&ctx, first, sc, sr)?;
    let (entry, arrow) = entry_dir(&ctx, last)?;
    let a0 = exit_attachment(canvas, exit, sc, sr, edge_index)?;
    let a_end = entry_attachment(canvas, entry, tc, tr, edge_index)?;
    let waypoints = waypoints(canvas, &ctx, exit, entry, a0, a_end)?;
    draw_runs(canvas, &waypoints, a0, a_end, arrow);
    Ok(())
}

/// How the path leaves the source box, judged from the first path cell's
/// position relative to the source cell.
fn exit_dir(ctx: &Ctx, first: Cell, sc: usize, sr: usize) -> Result<Dir, RenderError> {
    if first.1 == sr {
        match first.0.cmp(&sc) {
            std::cmp::Ordering::Greater => Ok(Dir::East),
            std::cmp::Ordering::Less => Ok(Dir::West),
            std::cmp::Ordering::Equal => Err(unsupported_side(ctx, "leaves")),
        }
    } else if first.0 == sc {
        Ok(if first.1 < sr { Dir::North } else { Dir::South })
    } else {
        Err(unsupported_side(ctx, "leaves"))
    }
}

/// How the path arrives at the target box, and the matching arrowhead
/// char, judged from the last path cell's position.
fn entry_dir(ctx: &Ctx, last: Cell) -> Result<(Dir, char), RenderError> {
    let (tc, tr) = (ctx.tc, ctx.tr);
    if last.1 == tr {
        match last.0.cmp(&tc) {
            std::cmp::Ordering::Less => Ok((Dir::East, '>')),
            std::cmp::Ordering::Greater => Ok((Dir::West, '<')),
            std::cmp::Ordering::Equal => Err(unsupported_side(ctx, "enters")),
        }
    } else if last.0 == tc {
        if last.1 < tr {
            // Moving south arrives through the target's north side.
            Ok((Dir::South, 'v'))
        } else {
            // Moving north arrives through its south side.
            Ok((Dir::North, '^'))
        }
    } else {
        Err(unsupported_side(ctx, "enters"))
    }
}

/// The attachment point where the path leaves a box through `dir`: one
/// char off the border (horizontal exits) or one line off (vertical).
/// Rejected as a typed error when it falls outside the grid — valid
/// layouts never produce that, since exits and entries always have a gap
/// row on their far side.
fn exit_attachment(
    canvas: &Canvas,
    dir: Dir,
    c: usize,
    r: usize,
    ei: usize,
) -> Result<Pt, RenderError> {
    let pt = match dir {
        Dir::East => (Some(canvas.offsets[c + 1]), Some(canvas.mid(r))),
        Dir::West => (canvas.offsets[c].checked_sub(1), Some(canvas.mid(r))),
        Dir::North => (Some(canvas.center(c)), (r * BAND_HEIGHT).checked_sub(1)),
        Dir::South => (Some(canvas.center(c)), Some(r * BAND_HEIGHT + 2 + 1)),
    };
    attachment(canvas, pt, ei, "leaves")
}

/// The attachment point where the path arrives at a box moving `dir`:
/// into the west side from the west, the east side from the east, the
/// north side from above, the south side from below.
fn entry_attachment(
    canvas: &Canvas,
    dir: Dir,
    c: usize,
    r: usize,
    ei: usize,
) -> Result<Pt, RenderError> {
    let pt = match dir {
        Dir::East => (canvas.offsets[c].checked_sub(1), Some(canvas.mid(r))),
        Dir::West => (Some(canvas.offsets[c + 1]), Some(canvas.mid(r))),
        Dir::South => (Some(canvas.center(c)), (r * BAND_HEIGHT).checked_sub(1)),
        Dir::North => (Some(canvas.center(c)), Some(r * BAND_HEIGHT + 2 + 1)),
    };
    attachment(canvas, pt, ei, "enters")
}

/// Validate an attachment point and unwrap its coordinates.
fn attachment(
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

/// Walk the path, collecting waypoints: the source attachment, a corner
/// wherever the run direction turns, and the target attachment.
fn waypoints(
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
            push_corner(canvas, ctx, &mut wps, dir, next_dir, ctx.path.get(i + 1).copied())?;
        }
        dir = next_dir;
    }
    wps.push(a_end);
    Ok(wps)
}

/// Push the corner where a horizontal run on one line meets a vertical run
/// on one column (or vice versa): their intersection point.
fn push_corner(
    canvas: &Canvas,
    ctx: &Ctx,
    wps: &mut Vec<Pt>,
    dir: Dir,
    next_dir: Dir,
    next_cell: Option<Cell>,
) -> Result<(), RenderError> {
    let corner = match (dir, next_dir) {
        (Dir::East | Dir::West, Dir::North | Dir::South) => {
            // Vertical run's column: the next cell's, or the target's when
            // the turn happens at the last cell.
            let col = next_cell.map_or(ctx.tc, |next| next.0);
            (canvas.center(col), wps.last().expect("seeded").1)
        }
        (Dir::North | Dir::South, Dir::East | Dir::West) => {
            let row = next_cell.map_or(ctx.tr, |next| next.1);
            (wps.last().expect("seeded").0, canvas.mid(row))
        }
        _ => {
            return Err(RenderError::unsupported(format!(
                "edge {} turns without changing axis",
                ctx.edge_index
            )))
        }
    };
    wps.push(corner);
    Ok(())
}

/// Draw the runs between consecutive waypoints, then the corners, then the
/// arrowhead at the final approach.
fn draw_runs(canvas: &mut Canvas, wps: &[Pt], a0: Pt, a_end: Pt, arrow: char) {
    for w in wps.windows(2) {
        draw_run(canvas, w[0], w[1], a0, a_end);
    }
    for &corner in &wps[1..wps.len() - 1] {
        canvas.grid[corner.1][corner.0] = '+';
    }
    canvas.grid[a_end.1][a_end.0] = arrow;
}

/// One run between two waypoints: dashes on a shared line (padded where it
/// touches a box attachment) or pipes on a shared column.
fn draw_run(canvas: &mut Canvas, p: Pt, q: Pt, a0: Pt, a_end: Pt) {
    if p.1 == q.1 {
        let (x1, x2) = ordered(p.0, q.0);
        let mut lo = x1;
        let mut hi = x2;
        if p == a0 || p == a_end {
            lo += 1;
        }
        if q == a0 || q == a_end {
            hi = hi.saturating_sub(1);
        }
        for x in lo..=hi {
            canvas.grid[p.1][x] = '-';
        }
    } else {
        let (y1, y2) = ordered(p.1, q.1);
        for y in y1..=y2 {
            canvas.grid[y][p.0] = '|';
        }
    }
}

/// `(min, max)` of two usize coordinates.
fn ordered(a: usize, b: usize) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Direction between two orthogonally adjacent path cells.
fn dir_between(p: Cell, q: Cell, ei: usize) -> Result<Dir, RenderError> {
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

fn unsupported_side(ctx: &Ctx, verb: &str) -> RenderError {
    RenderError::unsupported(format!(
        "edge {} {verb} its box through an unsupported side",
        ctx.edge_index
    ))
}