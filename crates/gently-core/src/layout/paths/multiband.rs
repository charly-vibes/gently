//! Purpose: the multi-band routes — east-going along the source band,
//! west-going bending out of the source's north/south first, and the
//! indexed-port slot paths (gently-byq).
//! Responsibilities: pick the run row candidates, validate the run
//! against the occupied set, and stitch the vertical walk, the approach
//! run, and the span-hosted ride fallbacks.
//! Rationale: split from paths.rs past the tidy file limit (gently-cbk).

use super::super::{Cell, Rows};
use super::bends::west_run_path;

use super::walk::{walk_column, walk_from};
use super::{Ctx, Endpoints};

/// Cell path of a multi-band route. East-going: along the source band to
/// the target column, then the full-row walk down (or up) the target
/// column (oracle: the diamond's c->d). West-going: the recorded oracle
/// A* shape (bg-edge-node's Ulm->car) bends out of the source's
/// north/south first — the full-row walk at the source column toward the
/// gap row adjacent to the target band — then runs along that gap row,
/// ending at the target column (the source-band walk would drive through
/// the intervening node cells). Both vertical walks jog east around band
/// rows occupied by a stacked node (gently-4ga) — the gap-only walk
/// jumped the occupied band row between two reserved gap rows, rendering
/// non-adjacent paths.
pub(super) fn multiband_path(ep: &Endpoints, rows: &Rows, ctx: &Ctx) -> Vec<Cell> {
    let path = if ep.gu.0 > ep.gv.0 {
        west_path(ep, rows, ctx)
    } else {
        east_path(ep, ctx)
    };
    if !path.is_empty() {
        return path;
    }
    // Every structured family missed (gently-bb5's walled far-apart
    // sweep shapes): last resort, a free-cell breadth-first run between
    // the two face rings. `None` keeps the typed render error.
    face_bfs_path(ep, ctx).unwrap_or_default()
}

/// A cell on the face ring of a span: outside the rectangle, adjacent
/// (4-neighbour) to one of its cells.
fn face_ring((anchor, span): (Cell, (usize, usize)), free: &impl Fn(Cell) -> bool) -> Vec<Cell> {
    let (w, h) = span;
    let inside =
        |c: Cell| c.0 >= anchor.0 && c.0 < anchor.0 + w && c.1 >= anchor.1 && c.1 < anchor.1 + h;
    let mut ring = Vec::new();
    for r in anchor.1.saturating_sub(1)..anchor.1 + h + 1 {
        for c in anchor.0.saturating_sub(1)..anchor.0 + w + 1 {
            let cell = (c, r);
            if !inside(cell)
                && free(cell)
                && [
                    (c, r.wrapping_sub(1)),
                    (c, r + 1),
                    (c.wrapping_sub(1), r),
                    (c + 1, r),
                ]
                .iter()
                .any(|&n| inside(n))
            {
                ring.push(cell);
            }
        }
    }
    ring
}

/// The breadth-first free-cell run from the source's face ring to the
/// target's, bounded to the endpoints' bounding box plus a margin of
/// two — the same class of orthogonal free path the structured routes
/// produce, so the renderer's exit/entry attachment still applies.
fn face_bfs_path(ep: &Endpoints, ctx: &Ctx) -> Option<Vec<Cell>> {
    let free = |c: Cell| !ctx.occupied.contains(&c);
    let sources = face_ring((ep.gu, ep.span_u), &free);
    let goals = face_ring((ep.gv, ep.span_v), &free);
    if sources.is_empty() || goals.is_empty() {
        return None;
    }
    let margin = 2;
    let min_r = ep.gu.1.min(ep.gv.1).saturating_sub(margin);
    let max_r = (ep.gu.1 + ep.span_u.1).max(ep.gv.1 + ep.span_v.1) + margin;
    let min_c = ep.gu.0.min(ep.gv.0).saturating_sub(margin);
    let max_c = (ep.gu.0 + ep.span_u.0).max(ep.gv.0 + ep.span_v.0) + margin;
    let in_bounds = |c: Cell| c.0 >= min_c && c.0 <= max_c && c.1 >= min_r && c.1 <= max_r;
    let mut parent: std::collections::HashMap<Cell, Cell> = std::collections::HashMap::new();
    let mut queue = std::collections::VecDeque::new();
    for &s in &sources {
        parent.insert(s, s);
        queue.push_back(s);
    }
    let goal = 'bfs: loop {
        let cur = queue.pop_front()?;
        if goals.contains(&cur) {
            break 'bfs cur;
        }
        for next in [
            (cur.0, cur.1.wrapping_sub(1)),
            (cur.0, cur.1 + 1),
            (cur.0.wrapping_sub(1), cur.1),
            (cur.0 + 1, cur.1),
        ] {
            if in_bounds(next)
                && free(next)
                && !parent.contains_key(&next)
                && parent.insert(next, cur).is_none()
            {
                queue.push_back(next);
            }
        }
    };
    let mut path = vec![goal];
    let mut cur = goal;
    while let Some(&p) = parent.get(&cur) {
        if p == cur {
            break;
        }
        path.push(p);
        cur = p;
    }
    path.reverse();
    Some(path)
}

/// The west-going route: try the run-row candidates beside the target
/// band; on failure fall back to the primary (first) candidate
/// unvalidated, or — when no gap row exists at all (span-hosted
/// attachment, gently-n0d) — ride the target's own span rows.
fn west_path(ep: &Endpoints, rows: &Rows, ctx: &Ctx) -> Vec<Cell> {
    let candidates = west_candidates(ep);
    west_run_first(ep, rows, ctx, &candidates)
        .unwrap_or_else(|| west_fallback_or_hosted(ep, rows, ctx, &candidates))
}

/// The run-row candidates beside the target band. The gap row adjacent
/// to the target band hosts the horizontal run; the vertical walk
/// covers every row between the source and that gap row at the source
/// column. Primary candidate (north): one band south of the target —
/// the recorded Ulm->car oracle bend (the row just south of the TARGET
/// band can sit inside a stacked node's span, as with Hof's box). But
/// once earlier bands reserved gap rows, that boundary's first row can
/// fall INSIDE a later stacked node (gently-hpd: edge 25's primary run
/// rode row 8 straight through nodes 11 and 29) — so the run is
/// validated against the occupied set, and a dirty primary falls back
/// to the target band's own boundary gap row (gap_rows[sv.1]: the rows
/// between the target band and the next, the geometrically adjacent
/// corridor).
fn west_candidates(ep: &Endpoints) -> Vec<usize> {
    let north = ep.sv.1 < ep.su.1;
    if north {
        vec![ep.sv.1 + 1, ep.sv.1]
    } else {
        vec![ep.sv.1]
    }
}

/// Try each candidate run row in order: the first candidate whose run
/// corridor is clear of node cells and whose bridge succeeds wins.
fn west_run_first(
    ep: &Endpoints,
    rows: &Rows,
    ctx: &Ctx,
    candidates: &[usize],
) -> Option<Vec<Cell>> {
    for &home in candidates {
        let Some(&g) = rows.gap_rows.get(home).and_then(|v| v.first()) else {
            continue;
        };
        let run = west_run(ep.gv, ep.span_v.0, ep.gu.0, g);
        if run.iter().all(|&c| !ctx.occupied.contains(&c)) {
            if let Some(path) = west_run_path(ep.gu, ep.span_u.1, &run, g, ctx) {
                return Some(path);
            }
        }
        // The walk gave up short of this run row: fall through
        // to the next candidate (gently-0is).
    }
    None
}

/// No candidate worked: fall back to the primary (first) candidate's
/// gap row unvalidated, or — when no gap row exists at all
/// (span-hosted attachment, gently-n0d) — ride the target's own span
/// rows.
fn west_fallback_or_hosted(
    ep: &Endpoints,
    rows: &Rows,
    ctx: &Ctx,
    candidates: &[usize],
) -> Vec<Cell> {
    let primary = candidates
        .first()
        .and_then(|&home| rows.gap_rows.get(home))
        .and_then(|v| v.first())
        .copied();
    match primary {
        Some(g) => west_fallback(ep, ctx, g),
        None => west_span_hosted(ep, ctx),
    }
}

/// The westward run cells at gap row `g`, from the target's east face
/// back to (and including) the source column.
fn west_run(gv: Cell, span_cx: usize, gu_x: usize, g: usize) -> Vec<Cell> {
    ((gv.0 + span_cx - 1)..=gu_x)
        .rev()
        .map(|c| (c, g))
        .collect()
}

/// The primary run row was dirty but is the only candidate: run it
/// anyway, truncating to the bare walk when the bridge gives up.
fn west_fallback(ep: &Endpoints, ctx: &Ctx, g: usize) -> Vec<Cell> {
    let run = west_run(ep.gv, ep.span_v.0, ep.gu.0, g);
    west_run_path(ep.gu, ep.span_u.1, &run, g, ctx)
        .unwrap_or_else(|| walk_column(ep.gu.0, walk_from(ep.gu.1, ep.span_u.1, g), g, ctx))
}

/// Span-hosted attachment (gently-n0d): no gap row exists at `home`
/// because the endpoints' ROW SPANS OVERLAP — the corridor-clear
/// accounting (gently-a6g) reserved no gap rows, the walk rides the
/// target's own span rows. Vertical at the source's column to the
/// target's band row, then west into the target's east face — the run
/// stops one column east of the face, so no node cell is entered and
/// the last path cell abuts the span.
fn west_span_hosted(ep: &Endpoints, ctx: &Ctx) -> Vec<Cell> {
    let (gu, gv) = (ep.gu, ep.gv);
    let from = walk_from(gu.1, ep.span_u.1, gv.1);
    let mut path = walk_column(gu.0, from, gv.1, ctx);
    let east_face = gv.0 + ep.span_v.0;
    if west_dodge_close(&mut path, gu.0, gv.1, east_face, ctx) {
        return path;
    }
    if !west_short_close(&mut path, gu.0, gv.1, ctx) {
        if path.is_empty() {
            // The walk never started: the effective start row (the
            // source's span bottom row, gently-0is's walk_from) is
            // adjacent to the target's band row, so there is no
            // interior cell. Fall back to the corner lane — the column
            // abutting the source's west face and the target's east
            // face — instead of reporting an empty path (gently-bb5).
            if let Some(lane) = corner_lane(ep, from, gv.1, ctx) {
                return lane;
            }
            // Or the source's own grown row span already hosts the
            // target's band row: run west along that shared row.
            if let Some(run) = hosted_run(ep, gv.1, ctx) {
                return run;
            }
            return path;
        }
        return path; // gave up: truncate, never jump
    }
    path.extend((east_face..gu.0).rev().map(|c| (c, gv.1)));
    path
}

/// The straight west run along the target's band row when the source's
/// own grown row span hosts that row: the row is one of the source's
/// face rows, so the edge runs the free corridor between the two
/// faces. `None` when the row is outside the source's span or a cell
/// is taken.
fn hosted_run(ep: &Endpoints, dest: usize, ctx: &Ctx) -> Option<Vec<Cell>> {
    if dest < ep.gu.1 || dest >= ep.gu.1 + ep.span_u.1 {
        return None;
    }
    let run: Vec<Cell> = ((ep.gv.0 + ep.span_v.0)..ep.gu.0)
        .rev()
        .map(|c| (c, dest))
        .collect();
    (!run.iter().any(|&cell| ctx.occupied.contains(&cell))).then_some(run)
}

/// The corner lane beside a span-hosted west pair whose walk came back
/// empty: the single column abutting the source's west face AND the
/// target's east face — exists only when the source's west face sits
/// directly east of the target's — covering every row from the walk's
/// start row to the target's band row. `None` keeps the typed render
/// error.
fn corner_lane(ep: &Endpoints, from: usize, dest: usize, ctx: &Ctx) -> Option<Vec<Cell>> {
    if ep.gu.0.checked_sub(1) != Some(ep.gv.0 + ep.span_v.0) {
        return None;
    }
    let dc = ep.gu.0 - 1;
    let rows = if dest > from {
        (from..=dest).collect::<Vec<_>>()
    } else {
        ((dest..=from).rev()).collect()
    };
    let lane: Vec<Cell> = rows.iter().map(|&r| (dc, r)).collect();
    (!lane.iter().any(|&cell| ctx.occupied.contains(&cell))).then_some(lane)
}

/// Dodge-column close on the band row: continue ALONG the band row to
/// the run's west end (`east_face`, the cell abutting the target's east
/// border). Bridging east to the source column and running back west
/// would 180°-turn (gently-0is: edge 1603). The band row can hold the
/// target's own cells — validate the corridor and truncate rather than
/// jump. Returns true when the path was closed here.
fn west_dodge_close(
    path: &mut Vec<Cell>,
    gu_x: usize,
    band_row: usize,
    east_face: usize,
    ctx: &Ctx,
) -> bool {
    let dodge_closed = matches!(path.last(), Some(&(dc, dr)) if dr == band_row && dc != gu_x);
    if !dodge_closed {
        return false;
    }
    let Some(&(dc, _)) = path.last() else {
        return true;
    };
    let (lo, hi) = if dc > east_face {
        (east_face, dc)
    } else {
        (dc, east_face)
    };
    if !(lo..=hi).all(|c| !ctx.occupied.contains(&(c, band_row))) {
        return true; // corridor blocked: truncate
    }
    if dc > east_face {
        path.extend((east_face..dc).rev().map(|c| (c, band_row)));
    } else {
        path.extend((dc + 1..=east_face).map(|c| (c, band_row)));
    }
    true
}

/// The walk is one row short of the band row at the source column:
/// close the final cell so the path abuts the target's face. Returns
/// false for any other end (the walk gave up mid-column).
fn west_short_close(path: &mut Vec<Cell>, gu_x: usize, band_row: usize, ctx: &Ctx) -> bool {
    match path.last() {
        Some(&(dc, dr)) if dr.abs_diff(band_row) == 1 && dc == gu_x => {
            if !ctx.occupied.contains(&(gu_x, band_row)) {
                path.push((gu_x, band_row));
            }
            true
        }
        _ => false,
    }
}

/// The east-going route: run along the source's band row toward the
/// target column, walk vertically to the face row nearest the source,
/// and approach the target's column along the face-adjacent row.
fn east_path(ep: &Endpoints, ctx: &Ctx) -> Vec<Cell> {
    let (gu, gv) = (ep.gu, ep.gv);
    let mut walk_col = gv.0;
    let mut path = east_run(ep, ctx, &mut walk_col);
    let face = east_face_row(gu, gv, ep.span_v.1);
    // A walk down the source's OWN column must start below the
    // source's row span — the band row is the span's top row, and the
    // span's cells would force a jog with no lane at the grid's west
    // edge (gently-0is: edge 68). Walks from columns east of the span
    // (the advanced walk_col) keep the band row — the span is not in
    // their way, and the pinned tier-1 shapes start there.
    let from = if walk_col >= gu.0 && walk_col < gu.0 + ep.span_u.0 {
        walk_from(gu.1, ep.span_u.1, face)
    } else {
        gu.1
    };
    path.extend(walk_column(walk_col, from, face, ctx));
    if walk_col != gv.0 {
        east_approach(ep, ctx, &mut path, walk_col, face);
    }
    if path.is_empty() && walk_col == gv.0 && gu.0 == gv.0 {
        // Same-column vertical edge the walk could not realize
        // (gently-481, gently-bb5's sweep repro): either the target's
        // grown face row neighbours the walk's effective start row
        // (`from` — no interior cell at all), or every interior row is
        // walled and no jog lane exists because the corridor at
        // `from`'s row passes through the source's own grown span.
        // Fall back to the flank: a contiguous run down (up) the free
        // column beside both spans' faces, from the source's face row
        // to the target's — the east-ring geometry of gently-a6g. Both
        // flanks walled keeps the typed render error.
        path = flank_path(ep, from, face, ctx).unwrap_or(path);
    }
    path
}

/// The flank run beside a same-column vertical pair whose walk came
/// back empty: the free column abutting BOTH spans' side faces (only
/// when the two spans are equally wide — otherwise no single column
/// abuts both), covering every row from the walk's start row down to
/// the target's face row. `None` when a cell is taken.
fn flank_path(ep: &Endpoints, from: usize, face: usize, ctx: &Ctx) -> Option<Vec<Cell>> {
    if ep.span_u.0 != ep.span_v.0 {
        return None;
    }
    let dc = [Some(ep.gu.0 + ep.span_u.0), ep.gu.0.checked_sub(1)];
    let rows = if face > from {
        (from..=face).collect::<Vec<_>>()
    } else {
        ((face..=from).rev()).collect()
    };
    dc.into_iter().flatten().find_map(|c| {
        let lane: Vec<Cell> = rows.iter().map(|&r| (c, r)).collect();
        (!lane.iter().any(|&cell| ctx.occupied.contains(&cell))).then_some(lane)
    })
}

/// The face row nearest the source (gently-a6g): a grown target entered
/// from the south is met at its bottom span row — the walk never
/// crosses the target's own cells.
fn east_face_row(gu: Cell, gv: Cell, span_cy: usize) -> usize {
    if gu.1 > gv.1 {
        gv.1 + span_cy - 1
    } else {
        gv.1
    }
}

/// The straight run east at the source's band row, truncated at the
/// first occupied cell — a sibling target stacked at the source's row
/// (the fan-out's first target, gently-206) may not be driven
/// through; the walk then continues down the last free column and
/// steps east into the target's column at the face-adjacent row.
/// Advances `walk_col` to the run's last free column.
fn east_run(ep: &Endpoints, ctx: &Ctx, walk_col: &mut usize) -> Vec<Cell> {
    let (gu, gv) = (ep.gu, ep.gv);
    let mut path: Vec<Cell> = Vec::new();
    *walk_col = gv.0;
    if gu.0 >= gv.0 {
        return path;
    }
    // When the run's row lies INSIDE the target's row span (a grown
    // target whose bottom face row is the source's band row), the
    // run stops west of the span — entering the target's own
    // columns at that row would drive into its cells; the
    // face-adjacent approach run below connects instead.
    let capped_end = if gu.1 >= gv.1 && gu.1 < gv.1 + ep.span_v.1 {
        gv.0.saturating_sub(1)
    } else {
        gv.0
    };
    if capped_end < gu.0 + ep.span_u.0 {
        return path;
    }
    for c in gu.0 + ep.span_u.0..=capped_end {
        if ctx.occupied.contains(&(c, gu.1)) {
            break;
        }
        path.push((c, gu.1));
        *walk_col = c;
    }
    path
}

/// The walk ended short of the target's column: run east (or west)
/// along the face-adjacent row into the target's column, one cell per
/// column, so the last path cell abuts the target's north (or south)
/// face. A blocked cell aborts the run — the renderer reports the
/// shape as a typed error rather than drawing through a node
/// (gently-206).
fn east_approach(ep: &Endpoints, ctx: &Ctx, path: &mut Vec<Cell>, walk_col: usize, face: usize) {
    let (gu, gv) = (ep.gu, ep.gv);
    // Connect vertically through the walk column from the walk's
    // last row to the approach row. The walk is exclusive of its
    // endpoint rows, so a SOUTH walk (source north of the face but
    // below the target's top row) stops at face-1 while the
    // approach row is face+1 — the face row itself still separates
    // them, and a single-cell insert jumped it (gently-hpd: edge 5
    // walked to (5,3), inserted only (5,5), and the orthogonality
    // gate rejected (5,3) -> (5,5)). Fill EVERY row between the
    // walk's last row and the approach row at walk_col — the
    // column lies strictly west of the target (walk_col < gv.0),
    // so the intermediate rows are outside the box. Any blocked
    // intermediate leaves the connect out entirely: the renderer
    // reports the shape as a typed error rather than drawing
    // through a node.
    let approach = if gu.1 > gv.1 { face + 1 } else { face - 1 };
    let mut cells: Vec<Cell> = if walk_col < gv.0 {
        (walk_col + 1..=gv.0).map(|c| (c, approach)).collect()
    } else {
        ((gv.0..walk_col).rev().map(|c| (c, approach))).collect()
    };
    connect_vertical(ctx, path, walk_col, approach, &mut cells);
    if cells.iter().all(|&c| !ctx.occupied.contains(&c)) {
        path.extend(cells);
    } else if ep.span_v.1 > 1 {
        east_ride(ctx, path, walk_col, gu, gv, ep.span_v.1, face);
    }
}

/// Splice the vertical connect from the walk's last row to the
/// approach row in front of `cells` when the whole connect is free.
fn connect_vertical(
    ctx: &Ctx,
    path: &[Cell],
    walk_col: usize,
    approach: usize,
    cells: &mut Vec<Cell>,
) {
    if let Some(&(_, last_y)) = path.last() {
        if last_y != approach && !ctx.occupied.contains(&(walk_col, approach)) {
            let connect: Vec<Cell> = if last_y < approach {
                (last_y + 1..=approach).map(|r| (walk_col, r)).collect()
            } else {
                ((approach..last_y).rev().map(|r| (walk_col, r))).collect()
            };
            if connect.iter().all(|&c| !ctx.occupied.contains(&c)) {
                cells.splice(0..0, connect);
            }
        }
    }
}

/// The face-adjacent approach is taken (gently-0is): a fan-in
/// hub with a sibling stacked north/south of its band leaves
/// no free approach cell at the target's column. Ride the
/// target's own span rows at the walk column instead (the
/// west-side mirror of the span-hosted attachment, gently-n0d
/// west branch): the walk column lies strictly west of the
/// span, so the rows outside the box are approachable — the
/// last path cell abuts the target's WEST face on a row
/// inside its span (the entry's west-side rule, gently-206).
/// North of the target the ride extends south to the span's
/// bottom row; south of it, north to the first row inside
/// the span. Any blocked ride row keeps the typed error
/// rather than drawing through a node.
#[allow(clippy::too_many_arguments)]
fn east_ride(
    ctx: &Ctx,
    path: &mut Vec<Cell>,
    walk_col: usize,
    gu: Cell,
    gv: Cell,
    span_cy: usize,
    face: usize,
) {
    let ride: Vec<Cell> = if gu.1 > gv.1 {
        (gv.1 + 1..=face).rev().map(|r| (walk_col, r)).collect()
    } else {
        (face..=gv.1 + span_cy - 1).map(|r| (walk_col, r)).collect()
    };
    if ride.iter().all(|&c| !ctx.occupied.contains(&c)) {
        path.extend(ride);
    }
}
