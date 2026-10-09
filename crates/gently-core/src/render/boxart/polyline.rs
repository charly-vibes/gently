//! Purpose: polyline drawing for bent and multi-band edges of the boxart
//! renderer (gently-css) — the geometry twin of the ascii renderer's
//! polyline with Unicode glyph selection.
//! Responsibilities: turn an edge's routed cell path into a glyph polyline
//! — attachment points one char/line off the source/target box borders,
//! the edge style's corner glyphs where the run direction turns, horizontal
//! runs whose character at column x is `pattern[x mod len]` (the upstream
//! per-cell phase rule: every non-SHORT cell restarts its pattern at its
//! own char offset, which coincides with a global column phase), vertical
//! runs of the style's ver glyph, and the open-triangle arrowhead at the
//! final approach to the target box.
//! Rationale: kept parallel to `render::ascii::polyline` rather than
//! unified: the ascii renderer is a closed, corpus-pinned capability whose
//! simplified cell model differs from the boxart one (phase anchoring,
//! attachment columns, column widths) — unifying them risks byte drift in
//! pinned output for no behavioral gain.

use super::canvas::{BAND_HEIGHT, Canvas};
use super::styles;
use crate::render::ascii::RenderError;
use crate::layout::{Cell, Layout};

/// Direction of travel along an edge polyline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Dir {
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
    style: String,
}

/// Draw one bent or multi-band edge as a glyph polyline through its routed
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
        style: canvas.edge_style(edge_index).to_string(),
    };
    let Some(&first) = ctx.path.first() else {
        return Err(RenderError {
            message: format!("edge {edge_index} has an empty path"),
        });
    };
    let last = *ctx.path.last().expect("path has a first cell");
    let exit = exit_dir(&ctx, first, sc, sr)?;
    let (entry, arrow) = entry_dir(&ctx, last)?;
    let a0 = exit_attachment(canvas, exit, sc, sr, edge_index)?;
    let a_end = entry_attachment(canvas, entry, tc, tr, edge_index)?;
    let waypoints = waypoints(canvas, &ctx, exit, entry, a0, a_end)?;
    draw_runs(canvas, &waypoints, a0, a_end, &ctx.style, arrow);
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
/// direction, judged from the last path cell's position.
fn entry_dir(ctx: &Ctx, last: Cell) -> Result<(Dir, Dir), RenderError> {
    let (tc, tr) = (ctx.tc, ctx.tr);
    if last.1 == tr {
        match last.0.cmp(&tc) {
            std::cmp::Ordering::Less => Ok((Dir::East, Dir::East)),
            std::cmp::Ordering::Greater => Ok((Dir::West, Dir::West)),
            std::cmp::Ordering::Equal => Err(unsupported_side(ctx, "enters")),
        }
    } else if last.0 == tc {
        if last.1 < tr {
            // Moving south arrives through the target's north side.
            Ok((Dir::South, Dir::South))
        } else {
            // Moving north arrives through its south side.
            Ok((Dir::North, Dir::North))
        }
    } else {
        Err(unsupported_side(ctx, "enters"))
    }
}

/// The attachment point where the path leaves a box through `dir`: one
/// char off the border (horizontal exits) or one line off (vertical).
/// Vertical attachments sit two chars into the node's column — upstream
/// draws corner cells at fixed cell-x 2 (probed: a 3-wide none-border box
/// still attaches at column start + 2). Rejected as a typed error when it
/// falls outside the grid — valid layouts never produce that.
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
        Dir::North => (Some(canvas.attach_col(c)), (r * BAND_HEIGHT).checked_sub(1)),
        Dir::South => (Some(canvas.attach_col(c)), Some(r * BAND_HEIGHT + 3)),
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
        Dir::South => (Some(canvas.attach_col(c)), (r * BAND_HEIGHT).checked_sub(1)),
        Dir::North => (Some(canvas.attach_col(c)), Some(r * BAND_HEIGHT + 3)),
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
        return Err(RenderError {
            message: format!("edge {ei} {verb} its box through an unreachable side"),
        });
    };
    if y >= canvas.grid.len() || x >= canvas.grid[0].len() {
        return Err(RenderError {
            message: format!("edge {ei} {verb} its box outside the grid at char {x},{y}"),
        });
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
            (canvas.attach_col(col), wps.last().expect("seeded").1)
        }
        (Dir::North | Dir::South, Dir::East | Dir::West) => {
            let row = next_cell.map_or(ctx.tr, |next| next.1);
            (wps.last().expect("seeded").0, canvas.mid(row))
        }
        _ => {
            return Err(RenderError {
                message: format!("edge {} turns without changing axis", ctx.edge_index),
            })
        }
    };
    wps.push(corner);
    Ok(())
}

/// Draw the runs between consecutive waypoints, then the corners, then the
/// arrowhead at the final approach.
fn draw_runs(canvas: &mut Canvas, wps: &[Pt], a0: Pt, a_end: Pt, style: &str, arrow: Dir) {
    for w in wps.windows(2) {
        draw_run(canvas, w[0], w[1], a0, a_end, style);
    }
    for k in 1..wps.len() - 1 {
        // Arms of the corner: back along the incoming segment (opposite of
        // the travel direction into the corner), forward along the
        // outgoing one.
        let d_in = dir_of(wps[k - 1], wps[k]);
        let d_out = dir_of(wps[k], wps[k + 1]);
        let back = opposite(d_in);
        let glyph = corner_glyph(style, back, d_out);
        canvas.grid[wps[k].1][wps[k].0] = glyph;
    }
    canvas.grid[a_end.1][a_end.0] = styles::arrow(arrow);
}

/// Travel direction from `p` to `q` (axis-aligned waypoints).
fn dir_of(p: Pt, q: Pt) -> Dir {
    if p.1 == q.1 {
        if q.0 > p.0 { Dir::East } else { Dir::West }
    } else if q.1 > p.1 {
        Dir::South
    } else {
        Dir::North
    }
}

/// The opposite travel direction.
fn opposite(d: Dir) -> Dir {
    match d {
        Dir::East => Dir::West,
        Dir::West => Dir::East,
        Dir::North => Dir::South,
        Dir::South => Dir::North,
    }
}

/// The corner glyph whose arms match the neighbourhood: the corner slot
/// is determined by the arm set — {east,south} = SE, {west,south} = SW,
/// {east,north} = NE, {west,north} = NW (probed: an over-bend entering a
/// box from the north turns with the SE glyph `┌`/`╔`, and leaves the
/// other box with the SW glyph `┐`/`╗`).
fn corner_glyph(style: &str, back: Dir, fwd: Dir) -> char {
    let e = styles::edge(style);
    match (back, fwd) {
        (Dir::East, Dir::South) | (Dir::South, Dir::East) => e.corners[0],
        (Dir::West, Dir::South) | (Dir::South, Dir::West) => e.corners[1],
        (Dir::East, Dir::North) | (Dir::North, Dir::East) => e.corners[2],
        (Dir::West, Dir::North) | (Dir::North, Dir::West) => e.corners[3],
        _ => e.corners[0],
    }
}

/// One run between two waypoints: the style's horizontal pattern phased
/// to the absolute columns (char at x = `pattern[x mod len]`) on a shared
/// line, or the ver glyph on a shared column. The char at a box
/// attachment stays blank — upstream chops the pattern one short of an
/// attached box side (the `_draw_hor` START_* chop).
fn draw_run(canvas: &mut Canvas, p: Pt, q: Pt, a0: Pt, a_end: Pt, style: &str) {
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
        let pat: Vec<char> = styles::edge(style).hor.chars().collect();
        let len = pat.len();
        for x in lo..=hi {
            canvas.grid[p.1][x] = pat[x % len];
        }
    } else {
        let (y1, y2) = ordered(p.1, q.1);
        let ver = styles::edge(style).ver;
        for y in y1..=y2 {
            canvas.grid[y][p.0] = ver;
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
        _ => Err(RenderError {
            message: format!("edge {ei} path cells {p:?} -> {q:?} are not orthogonally adjacent"),
        }),
    }
}

fn unsupported_side(ctx: &Ctx, verb: &str) -> RenderError {
    RenderError {
        message: format!("edge {} {verb} its box through an unsupported side", ctx.edge_index),
    }
}