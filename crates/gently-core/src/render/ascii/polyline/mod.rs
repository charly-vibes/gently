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

mod attach;

use super::canvas::Canvas;
use super::edges::EdgeNodeEnds;
use super::RenderError;
use crate::layout::{Cell, Layout};
use attach::{
    dir_between, entry_attachment, entry_dir, exit_attachment, exit_dir, start_glyph, waypoints,
    EntrySide,
};

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
/// target cell it must reach, its model index (for typed errors), and
/// both endpoints' grid-cell spans (gently-awd — attachments and side
/// detection judge against the whole span, not just the anchor cell).
struct Ctx<'a> {
    path: &'a [Cell],
    tc: usize,
    tr: usize,
    edge_index: usize,
    span_s: (usize, usize),
    span_t: (usize, usize),
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
    from: usize,
    to: usize,
    start_arrow: bool,
    ends: EdgeNodeEnds,
) -> Result<(), RenderError> {
    let ctx = path_ctx(canvas, layout, edge_index, from, to);
    let (sc, sr) = layout.node_cells[from];
    let (tc, tr) = layout.node_cells[to];
    let Some(&first) = ctx.path.first() else {
        return Err(RenderError::unsupported(format!(
            "edge {edge_index} has an empty path"
        )));
    };
    let last = *ctx.path.last().expect("path has a first cell");
    let exit = exit_dir(&ctx, first, sc, sr)?;
    let (entry, _) = entry_dir(&ctx, last)?;
    let a0 = exit_attachment(canvas, exit, sc, sr, ctx.span_s, edge_index, first.1)?;
    let side = entry_side(&ctx, tc, tr, last);
    let a_end = entry_attachment(canvas, entry, &side, edge_index)?;
    let (waypoints, arrow) = final_waypoints(canvas, &ctx, exit, entry, a0, a_end, ends)?;
    let start = start_head(start_arrow, ends, exit);
    draw_runs(canvas, &waypoints, a0, a_end, arrow, start);
    Ok(())
}

/// The polyline context of edge `edge_index`: its routed path and both
/// endpoints' spans.
fn path_ctx<'a>(
    canvas: &Canvas,
    layout: &'a Layout,
    edge_index: usize,
    from: usize,
    to: usize,
) -> Ctx<'a> {
    let (tc, tr) = layout.node_cells[to];
    Ctx {
        path: &layout.edge_paths[edge_index],
        tc,
        tr,
        edge_index,
        span_s: canvas.spans[from],
        span_t: canvas.spans[to],
    }
}

/// The target-side entry geometry: an indexed-port slot entry arrives one
/// char east of the target's east border — inside the adjacent gap
/// column, mirroring the west-side rule's one-char offset (gently-byq:
/// the SO fixture's `<` at the gap column's second char). Ordinary
/// east-side entries stay flush (the recorded bent westward shapes).
fn entry_side(ctx: &Ctx, tc: usize, tr: usize, last: Cell) -> EntrySide {
    EntrySide {
        c: tc,
        r: tr,
        span: ctx.span_t,
        row: last.1,
        shift: usize::from(last.0 >= tc + ctx.span_t.0 && last.1 > tr),
    }
}

/// The start arrowhead points into the source box — opposite the exit
/// travel direction, the same glyph an end arrowhead arriving through
/// that side would carry. A shape:edge source carries none (upstream
/// strips its START flags outright).
fn start_head(start_arrow: bool, ends: EdgeNodeEnds, exit: Dir) -> Option<char> {
    if start_arrow && !ends.src {
        Some(start_glyph(exit))
    } else {
        None
    }
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
        return Ok((
            waypoints(canvas, ctx, exit, entry, a0, a_end)?,
            Some(arrow_of(ctx)?),
        ));
    }
    let last_dir = last_run_dir(ctx, exit)?;
    let sliding_past =
        matches!(entry, Dir::North | Dir::South) && matches!(last_dir, Dir::East | Dir::West);
    if !sliding_past {
        return Ok((waypoints(canvas, ctx, exit, entry, a0, a_end)?, None));
    }
    Ok((slide_past(canvas, ctx, exit, a0, last_dir)?, None))
}

/// The direction of the final run: the last path hop's direction (a
/// one-cell path travels straight from the exit).
fn last_run_dir(ctx: &Ctx, exit: Dir) -> Result<Dir, RenderError> {
    match ctx.path.len() {
        0..=1 => Ok(exit),
        n => dir_between(ctx.path[n - 2], ctx.path[n - 1], ctx.edge_index),
    }
}

/// The sliding shape: the run extends flush through the target column's
/// far edge (West: through the target's west offset; East: the column
/// after its span).
fn slide_past(
    canvas: &Canvas,
    ctx: &Ctx,
    exit: Dir,
    a0: Pt,
    last_dir: Dir,
) -> Result<Vec<Pt>, RenderError> {
    let far = if last_dir == Dir::West {
        canvas.offsets[ctx.tc]
    } else {
        canvas.offsets[ctx.tc + 1]
    };
    let mut wps = waypoints_before_tail(canvas, ctx, exit, a0)?;
    let last = ctx.path.last().expect("path has a first cell").1;
    wps.push((far, canvas.mid(last)));
    Ok(wps)
}

/// The entry arrowhead glyph for the path (the normal target-side rule).
fn arrow_of(ctx: &Ctx) -> Result<char, RenderError> {
    let last = *ctx.path.last().expect("path has a first cell");
    Ok(entry_dir(ctx, last)?.1)
}

/// The waypoints up to (excluding) the final run's tail: the source
/// attachment plus a corner wherever the run direction turns, walking all
/// but the last path cell.
fn waypoints_before_tail(
    canvas: &Canvas,
    ctx: &Ctx,
    exit: Dir,
    a0: Pt,
) -> Result<Vec<Pt>, RenderError> {
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
        for cell in &mut canvas.grid[p.1][lo..=hi] {
            *cell = b'-';
        }
    } else {
        let (y1, y2) = ordered(p.1, q.1);
        for r in &mut canvas.grid[y1..=y2] {
            r[p.0] = b'|';
        }
    }
}

/// `(min, max)` of two usize coordinates.
fn ordered(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}
