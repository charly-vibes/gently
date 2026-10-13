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

use super::canvas::Canvas;
use super::edges::EdgeNodeEnds;
use super::RenderError;
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
/// path cells. `start_arrow` requests the source-side arrowhead (the edge
/// starts with an arrow — the `<` of `<-->`): the glyph points INTO the
/// source box, mirroring the entry rule at the target (gently-7qz, pinned
/// by tests/repro/claims/pod-bidir-parallel.observed — the bent
/// bidirectional edge draws `v` at the source's north attachment).
/// `src_edge`/`dst_edge` flag `shape: edge` endpoint nodes (gently-cbk,
/// upstream Scout.pm zeroes the START/END flags of a multi-cell path at
/// such an endpoint): no arrowhead rides that end, and a final run that
/// slides UNDER or OVER the target's cell extends flush through the
/// target column's far edge instead of turning into it (the oracle's
/// merged full-width dash cell — bg-edge-node's Ulm->car line ends at
/// car's west edge with no stub and no corner).
pub(super) fn draw(
    canvas: &mut Canvas,
    layout: &Layout,
    edge_index: usize,
    (sc, sr): Cell,
    (tc, tr): Cell,
    start_arrow: bool,
    ends: EdgeNodeEnds,
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
    let (entry, _) = entry_dir(&ctx, last)?;
    let a0 = exit_attachment(canvas, exit, sc, sr, edge_index)?;
    let a_end = entry_attachment(canvas, entry, tc, tr, edge_index)?;
    let (waypoints, arrow) = final_waypoints(canvas, &ctx, exit, entry, a0, a_end, ends)?;
    // The start arrowhead points into the source box — opposite the exit
    // travel direction, the same glyph an end arrowhead arriving through
    // that side would carry. A shape:edge source carries none (upstream
    // strips its START flags outright).
    let start = if start_arrow && !ends.src {
        Some(start_glyph(exit))
    } else {
        None
    };
    draw_runs(canvas, &waypoints, a0, a_end, arrow, start);
    Ok(())
}

/// The waypoint chain and the (possibly suppressed) end arrowhead: a
/// `shape: edge` target under (or over) a horizontal final run slides the
/// run flush through the target column's far edge — no corner, no stub,
/// no arrowhead (gently-cbk); any other `shape: edge` target keeps the
/// normal chain but carries no head.
fn final_waypoints(
    canvas: &Canvas,
    ctx: &Ctx,
    exit: Dir,
    entry: Dir,
    a0: Pt,
    a_end: Pt,
    ends: EdgeNodeEnds,
) -> Result<(Vec<Pt>, Option<char>), RenderError> {
    if !ends.dst {
        return Ok((waypoints(canvas, ctx, exit, entry, a0, a_end)?, Some(arrow_of(ctx)?)));
    }
    let last = *ctx.path.last().expect("path has a first cell");
    // The direction of the final run: the last path hop's direction (a
    // one-cell path travels straight from the exit).
    let last_dir = match ctx.path.len() {
        0..=1 => exit,
        n => dir_between(ctx.path[n - 2], ctx.path[n - 1], ctx.edge_index)?,
    };
    let sliding_past = matches!(entry, Dir::North | Dir::South)
        && matches!(last_dir, Dir::East | Dir::West);
    if !sliding_past {
        return Ok((waypoints(canvas, ctx, exit, entry, a0, a_end)?, None));
    }
    let far = if last_dir == Dir::West {
        canvas.offsets[ctx.tc]
    } else {
        canvas.offsets[ctx.tc + 1]
    };
    let mut wps = waypoints_before_tail(canvas, ctx, exit, a0)?;
    wps.push((far, canvas.mid(last.1)));
    Ok((wps, None))
}

/// The entry arrowhead glyph for the path (the normal target-side rule).
fn arrow_of(ctx: &Ctx) -> Result<char, RenderError> {
    let last = *ctx.path.last().expect("path has a first cell");
    Ok(entry_dir(ctx, last)?.1)
}

/// The waypoints up to (excluding) the final run's tail: the source
/// attachment plus a corner wherever the run direction turns, walking all
/// but the last path cell.
fn waypoints_before_tail(canvas: &Canvas, ctx: &Ctx, exit: Dir, a0: Pt) -> Result<Vec<Pt>, RenderError> {
    let mut wps = vec![a0];
    let mut dir = exit;
    for i in 0..ctx.path.len() - 1 {
        let next_dir = dir_between(ctx.path[i], ctx.path[i + 1], ctx.edge_index)?;
        if next_dir != dir {
            push_corner(canvas, ctx, &mut wps, dir, next_dir, Some(ctx.path[i + 1]))?;
        }
        dir = next_dir;
    }
    Ok(wps)
}

/// The start arrowhead glyph for an edge leaving its source box through
/// `dir`: the arrow points INTO the box (gently-7qz, pinned by
/// tests/repro/claims/pod-bidir-parallel.observed — the bent bidirectional
/// edge leaves north and draws `v` at the source's north attachment).
fn start_glyph(exit: Dir) -> char {
    match exit {
        Dir::East => '<',
        Dir::West => '>',
        Dir::North => 'v',
        Dir::South => '^',
    }
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
        Dir::North => (Some(canvas.vert_x(c)), canvas.band_top(r).checked_sub(1)),
        Dir::South => (Some(canvas.vert_x(c)), Some(canvas.band_below(r))),
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
        Dir::South => (Some(canvas.vert_x(c)), canvas.band_top(r).checked_sub(1)),
        Dir::North => (Some(canvas.vert_x(c)), Some(canvas.band_below(r))),
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
            (canvas.vert_x(col), wps.last().expect("seeded").1)
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

/// Draw the runs between consecutive waypoints, then the corners, then
/// the arrowhead at the final approach — and the start arrowhead at the
/// source attachment when the edge carries one (gently-7qz). A
/// `shape: edge` target end carries no arrowhead (gently-cbk).
fn draw_runs(
    canvas: &mut Canvas,
    wps: &[Pt],
    a0: Pt,
    a_end: Pt,
    arrow: Option<char>,
    start: Option<char>,
) {
    for w in wps.windows(2) {
        draw_run(canvas, w[0], w[1], a0, a_end);
    }
    for &corner in &wps[1..wps.len() - 1] {
        canvas.grid[corner.1][corner.0] = b'+';
    }
    if let Some(arrow) = arrow {
        canvas.grid[a_end.1][a_end.0] = arrow as u8;
    }
    if let Some(s) = start {
        canvas.grid[a0.1][a0.0] = s as u8;
    }
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
            canvas.grid[p.1][x] = b'-';
        }
    } else {
        let (y1, y2) = ordered(p.1, q.1);
        for y in y1..=y2 {
            canvas.grid[y][p.0] = b'|';
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