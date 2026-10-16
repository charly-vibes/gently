//! Purpose: the edge-path realization half of the layout — gap-slot
//! accounting, grid-row assembly, and each edge's routed cell path by
//! travel kind (split from layout.rs past the tidy file limit,
//! gently-cbk).
//! Responsibilities: count the gap rows each boundary needs, assemble the
//! band/gap row sequence, and realize every route as an orthogonal cell
//! path — straight, bend, port dip, parallel channel, selfloop, and the
//! multi-band walks (east-going along the source band; west-going bending
//! out of the source's north/south first, gently-cbk), both vertical
//! walks jogging east around band rows occupied by stacked nodes
//! (gently-4ga).
//! Rationale: shaped by the recorded v0.69 @ ededa3d7 companions —
//! ge.layout.c1 (pure function) and c4 (orthogonal routes).

use super::{Cell, Placement, Route, Rows};
use crate::graph::Graph;
use std::collections::HashSet;

/// Shared routing context: the occupied node cells (the walks never
/// enter one) and the eastmost grid column (the dodge side guard,
/// gently-4ga).
struct Ctx<'a> {
    occupied: &'a HashSet<Cell>,
    max_x: usize,
}

/// Both endpoints of one edge as the routing pass sees them: the
/// placement slots, the grid cells, and the grid-cell spans (gently-awd).
struct Endpoints {
    su: Cell,
    sv: Cell,
    gu: Cell,
    gv: Cell,
    span_u: (usize, usize),
    span_v: (usize, usize),
}

/// How many gap rows each boundary needs (boundary `k` is the gap above
/// band `k`; `k == bands` trails the last band). Only edges that route
/// vertically or bend need one; a plain chain row stays gap-free (oracle:
/// mixed_isolated). Parallel multi-edges take one gap per extra channel.
/// Span-aware (gently-a6g): a vertical travel whose endpoints' row spans
/// overlap needs no gap at all (the fan-in star's east-row attachment —
/// the hub's grown span hosts the attachment row), and one whose
/// intermediate band rows are node-free at the walked column routes
/// through those rows (the star's below-band stem rides the empty
/// band between hub and leaf).
pub(super) fn required_slots(
    graph: &Graph,
    placement: &Placement,
    spans: &[(usize, usize)],
) -> Vec<usize> {
    let mut slot = vec![0usize; placement.bands + 1];
    for (ei, e) in graph.edges.iter().enumerate() {
        let pu = placement.cell[e.from].unwrap();
        let pv = placement.cell[e.to].unwrap();
        mark_slot(
            &mut slot,
            placement.route[ei],
            pu,
            pv,
            placement,
            spans,
            spans[e.from],
            spans[e.to],
        );
    }
    slot
}

/// A vertical travel needs no gap rows when its endpoints' band-row
/// spans leave a clear corridor: either the spans overlap (the attachment
/// row is hosted by the other node's grown span — the fan-in star's east
/// ring), or the band rows strictly between the spans are node-free at
/// BOTH endpoints' columns (the star's below-band stem rides the empty
/// band between hub and leaf). Blocked corridors keep the packed
/// stack's gap rows (the fan-out's below-attachments).
fn corridor_clear(
    placement: &Placement,
    spans: &[(usize, usize)],
    pu: Cell,
    span_u: (usize, usize),
    pv: Cell,
    span_v: (usize, usize),
) -> bool {
    // Overlapping spans host each other's attachment rows outright.
    let (u0, u1) = (pu.1, pu.1 + span_u.1 - 1);
    let (v0, v1) = (pv.1, pv.1 + span_v.1 - 1);
    if u0 <= v1 && v0 <= u1 {
        return true;
    }
    let upper_end = u1.min(v1);
    let lower_start = u0.max(v0);
    // Adjacent extents (empty corridor) keep the packed stack's gap row.
    if upper_end + 1 >= lower_start {
        return false;
    }
    let free_at = |rank: usize, band: usize| {
        !placement
            .cell
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.map(|c| (i, c)))
            .any(|(i, (r, b))| {
                r <= rank && rank < r + spans[i].0 && b <= band && band < b + spans[i].1
            })
    };
    (upper_end + 1..lower_start).all(|b| free_at(pu.0, b) && free_at(pv.0, b))
}

/// Raise a boundary's gap count to `count` when lower.
fn raise(slot: &mut [usize], boundary: usize, count: usize) {
    if slot[boundary] < count {
        slot[boundary] = count;
    }
}

/// Mark the gap slots one edge needs.
#[allow(clippy::too_many_arguments)]
fn mark_slot(
    slot: &mut [usize],
    route: Route,
    pu: Cell,
    pv: Cell,
    placement: &Placement,
    spans: &[(usize, usize)],
    span_u: (usize, usize),
    span_v: (usize, usize),
) {
    match route {
        Route::Selfloop => raise(slot, pu.1, 1),
        // A slot route rides the slotted node's own continuation rows
        // (grown to `N+1` by the port, gently-byq) — no gap rows.
        Route::Slot { .. } => {}
        // A south-port dip takes the first gap row below the band; a
        // north-port bend the first gap row above it.
        Route::PortBelow => raise(slot, pu.1 + 1, 1),
        Route::PortAbove => raise(slot, pu.1, 1),
        Route::Parallel(prior) => {
            let (boundary, k) = parallel_channel(prior, pu.1);
            raise(slot, boundary, k + 1);
        }
        // A stacked channel rides the boxes' row-span continuation rows
        // (gently-awd) — no gap rows; the span reserves them.
        Route::Stacked(_) => {}
        Route::Grew | Route::Around => mark_travel(slot, pu, pv, placement, spans, span_u, span_v),
    }
}

/// The boundary and 0-based gap index a parallel edge with `prior` prior
/// edges on its pair travels: odd priors bend above the band (first,
/// second, ... gap above), even priors below it — channels 1/2 keep the
/// recorded two-parallel oracle shape (above, then below).
fn parallel_channel(prior: usize, band: usize) -> (usize, usize) {
    if prior % 2 == 1 {
        (band, (prior - 1) / 2)
    } else {
        (band + 1, prior / 2 - 1)
    }
}

/// Mark the gap slots one vertical or bending route needs.
fn mark_travel(
    slot: &mut [usize],
    pu: Cell,
    pv: Cell,
    placement: &Placement,
    spans: &[(usize, usize)],
    span_u: (usize, usize),
    span_v: (usize, usize),
) {
    let (b1, b2) = (pu.1.min(pv.1), pu.1.max(pv.1));
    if b1 != b2 {
        // Span-aware skip (gently-a6g): a clear corridor needs no gap
        // rows — the walk rides the existing band rows.
        if corridor_clear(placement, spans, pu, span_u, pv, span_v) {
            return;
        }
        for boundary in b1 + 1..=b2 {
            raise(slot, boundary, 1);
        }
    } else if pu.0.abs_diff(pv.0) != 1 {
        // Same band but not adjacent ranks: bend above the band.
        raise(slot, pu.1, 1);
    }
}

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
pub(super) fn assemble_rows(
    placement: &Placement,
    spans: &[(usize, usize)],
    slots: &[usize],
) -> Rows {
    let bands = placement.bands;
    // Per-band anchored nodes, and per-band covering nodes (earlier
    // bands' row spans that reach over b) — precomputed once so the
    // walk stays linear (ge.layout.c6).
    let mut anchored: Vec<Vec<usize>> = vec![Vec::new(); bands];
    let mut covering: Vec<Vec<usize>> = vec![Vec::new(); bands];
    for (i, c) in placement.cell.iter().enumerate() {
        if let Some((_, bi)) = *c {
            if bi < bands {
                anchored[bi].push(i);
            }
            for cover in covering[bi + 1..(bi + spans[i].1).min(bands)].iter_mut() {
                cover.push(i);
            }
        }
    }
    for list in covering.iter_mut() {
        list.sort_by_key(|&i| placement.cell[i].map(|c| c.1).unwrap_or(0));
    }
    let mut row_of_band = vec![0usize; bands];
    let mut gap_rows: Vec<Vec<usize>> = vec![Vec::new(); bands + 1];
    // Occupied rectangles of every resolved node, in (rank r0, rank r1,
    // row y0, row y1) form — the ride-collision ledger (gently-0is).
    let mut occupied: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut row = 0usize;
    for b in 0..bands {
        push_gaps(&mut gap_rows[b], slots[b], &mut row);
        row_of_band[b] = resolve_band(
            placement,
            spans,
            &anchored,
            &covering[b],
            &row_of_band,
            slots,
            b,
            row,
            &occupied,
            &gap_rows,
        );
        // The cursor stays south of every span anchored on this band.
        let cy = anchored[b].iter().map(|&i| spans[i].1).max().unwrap_or(1);
        row = row.max(row_of_band[b] + cy);
        for &i in &anchored[b] {
            let (r, _) = placement.cell[i].unwrap();
            let (cx, cy) = spans[i];
            occupied.push((
                r,
                r + cx.saturating_sub(1),
                row_of_band[b],
                row_of_band[b] + cy - 1,
            ));
        }
    }
    push_gaps(&mut gap_rows[bands], slots[bands], &mut row);
    // A node's span extends south of its band's row: the grid height
    // must cover the deepest span extent (the 10-row so-flowchart-size
    // boxes; the star's 3-row hub).
    let height = placement
        .cell
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.map(|c| (i, c)))
        .map(|(i, (_, b))| row_of_band[b] + spans[i].1)
        .chain(std::iter::once(row))
        .max()
        .unwrap_or(row);
    Rows {
        row_of_band,
        gap_rows,
        height,
    }
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
#[allow(clippy::too_many_arguments)]
fn resolve_band(
    placement: &Placement,
    spans: &[(usize, usize)],
    anchored: &[Vec<usize>],
    covering: &[usize],
    row_of_band: &[usize],
    slots: &[usize],
    b: usize,
    cursor: usize,
    occupied: &[(usize, usize, usize, usize)],
    gap_rows: &[Vec<usize>],
) -> usize {
    let ride = covering.first().and_then(|&i| {
        let (ri, bi) = placement.cell[i].unwrap();
        // The span hosts band b: column-disjoint from every node
        // anchored on b? (rank space: one rank is two grid columns.)
        let disjoint = anchored[b].iter().all(|&j| {
            let (rj, _) = placement.cell[j].unwrap();
            rj + spans[j].0 <= ri || ri + spans[i].0 <= rj
        });
        if !disjoint {
            return None;
        }
        // The ride offset honours the gap rows reserved on the
        // boundaries it crosses (gently-n0d): a riding band whose
        // vertical travel reserved a gap row above it (same-column
        // band-adjacent stacked states — the corpus's t4->diagnosing
        // shape) may not sit grid-adjacent to the band above it, or
        // the walk has no cell to route through and the edge
        // realizes as a zero-cell path.
        let reserved: usize = (bi + 1..=b).map(|k| slots[k]).sum();
        let candidate = row_of_band[bi] + (b - bi) + reserved;
        // (c) The ride lives INSIDE the host span — the formula's row
        // count assumes every intermediate band packs one row; a cursor
        // fallback (collision below) re-packs real rows past that
        // assumption, and an out-of-span candidate is its signature.
        let host_bottom = row_of_band[bi] + spans[i].1;
        if candidate >= host_bottom {
            return None;
        }
        Some(candidate)
    });
    // (b) No reserved gap row may fall inside the ride's row extent —
    // gap rows are the routing corridors; a band row on one leaves its
    // vertical walks no cell to route through.
    if let Some(ride) = ride {
        let max_cy = anchored[b].iter().map(|&i| spans[i].1).max().unwrap_or(1);
        let hit = gap_rows[..(b + 1).min(gap_rows.len())]
            .iter()
            .flatten()
            .any(|&g| g >= ride && g < ride + max_cy);
        if hit {
            return cursor;
        }
    }
    if let Some(ride) = ride {
        let free = anchored[b].iter().all(|&j| {
            let (rj, _) = placement.cell[j].unwrap();
            let (cx, cy) = spans[j];
            occupied.iter().all(|&(r0, r1, y0, y1)| {
                rj > r1 || r0 > rj + cx.saturating_sub(1) || ride > y1 || y0 > ride + cy - 1
            })
        });
        if free {
            return ride;
        }
    }
    cursor
}

/// Append `count` consecutive gap rows to one boundary's list.
fn push_gaps(gaps: &mut Vec<usize>, count: usize, row: &mut usize) {
    for _ in 0..count {
        gaps.push(*row);
        *row += 1;
    }
}

/// Realize each edge's travel as a cell path over the assembled rows.
/// `spans` carries every node's (column, row) grid-cell span (gently-awd):
/// the occupied set covers a multi-celled node's WHOLE span (the walks
/// never enter any of its cells), and the straight-gap detection uses
/// the span to find the single gap column east/west of a node.
pub(super) fn route_edges(
    graph: &Graph,
    placement: &Placement,
    node_cells: &[Cell],
    rows: &Rows,
    max_x: usize,
    spans: &[(usize, usize)],
) -> Vec<Vec<Cell>> {
    let mut occupied: HashSet<Cell> = HashSet::new();
    for (&(c, r), &(cx, cy)) in node_cells.iter().zip(spans) {
        for dc in 0..cx {
            for dr in 0..cy {
                occupied.insert((c + dc, r + dr));
            }
        }
    }
    let ctx = Ctx {
        occupied: &occupied,
        max_x,
    };
    // The cells earlier edges' paths took (model order): an indexed
    // end-slot edge dodges EAST when its drop column is taken by one of
    // them (gently-byq).
    let mut used: HashSet<Cell> = HashSet::new();
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(ei, e)| {
            let su = placement.cell[e.from].unwrap();
            let sv = placement.cell[e.to].unwrap();
            let path = edge_path(
                placement.route[ei],
                &Endpoints {
                    su,
                    sv,
                    gu: node_cells[e.from],
                    gv: node_cells[e.to],
                    span_u: spans[e.from],
                    span_v: spans[e.to],
                },
                rows,
                &ctx,
                &used,
            );
            used.extend(path.iter().copied());
            path
        })
        .collect()
}

/// The cell path of one edge, by travel kind. Straight: the single gap
/// column between adjacent neighbours on one band. Bend: through the gap
/// row above (or below) the band, entering/exiting through the node
/// columns. Multi-band: east-going along the source band to the target
/// column, then vertically through the gap rows (oracle: the diamond's
/// c->d); west-going bends out of the source's north/south first and runs
/// along the gap row adjacent to the target band (oracle: bg-edge-node's
/// Ulm->car — the source-band walk would drive through the nodes between).
fn edge_path(
    route: Route,
    ep: &Endpoints,
    rows: &Rows,
    ctx: &Ctx,
    used: &HashSet<Cell>,
) -> Vec<Cell> {
    let (su, gu) = (ep.su, ep.gu);
    let (gv, span_u, span_v) = (ep.gv, ep.span_u, ep.span_v);
    match route {
        Route::Selfloop => selfloop_path(su, gu, rows),
        Route::Parallel(_) => bend_path(
            gu,
            gv,
            span_u,
            span_v,
            bend_gap_row(route, su.1, rows, gu.1),
        ),
        // The port dips run one gap row, all the way between the two node
        // columns — straight through under (or over) everything between.
        Route::PortBelow | Route::PortAbove => {
            let boundary = if route == Route::PortBelow {
                su.1 + 1
            } else {
                su.1
            };
            let g = rows
                .gap_rows
                .get(boundary)
                .and_then(|v| v.first())
                .copied()
                .unwrap_or(gu.1);
            bend_path(gu, gv, span_u, span_v, g)
        }
        Route::Grew | Route::Around => {
            // Straight through the single gap column between the two
            // nodes — east of the source's column span or west of the
            // target's (gently-awd: a multi-celled node's east border
            // sits at `col + cx - 1`, so the gap column shifts east).
            match straight_gap(gu, gv, span_u, span_v) {
                // Straight through the gap column when the pair shares a
                // row — literally, or because one node's grown span hosts
                // the other's row (gently-a6g: the fan-in star's east-ring
                // attachment — n2's row IS the hub's bottom span row, the
                // edge runs straight west into the hub's east face).
                Some(c)
                    if gu.1 == gv.1
                        || (gu.1 >= gv.1 && gu.1 < gv.1 + span_v.1)
                        || (gv.1 >= gu.1 && gv.1 < gu.1 + span_u.1) =>
                {
                    // The straight run rides the row the two spans
                    // OVERLAP on: gu's band row when gu is the rider
                    // inside gv's span (the fan-in star's east ring),
                    // gv's band row when gv rides inside gu's span —
                    // the fan-OUT stack (gently-206): the source grew a
                    // row span hosting the targets below it, and the
                    // straight cell must sit on the TARGET's row to
                    // abut its west face, not on the source's top row
                    // (which is diagonal to every rider below the
                    // first).
                    let row = if gu.1 >= gv.1 && gu.1 < gv.1 + span_v.1 {
                        gu.1
                    } else {
                        gv.1
                    };
                    vec![(c, row)]
                }
                _ if gu.1 == gv.1 => bend_path(
                    gu,
                    gv,
                    span_u,
                    span_v,
                    bend_gap_row(route, su.1, rows, gu.1),
                ),
                _ => multiband_path(ep, rows, ctx),
            }
        }
        // An indexed port slot (gently-byq): the decoded upstream routes.
        Route::Slot { end, n } => slot_path(ep, ctx, used, end, n),
        // A stacked channel (gently-awd): straight through the gap
        // column at grid row `band + k` — the pair's boxes' row span
        // hosts the channel rows (the so-flowchart-size oracle shape);
        // a non-adjacent pair bends through the stacked row.
        Route::Stacked(k) => {
            let row = gu.1 + k;
            match straight_gap(gu, gv, span_u, span_v) {
                Some(c) => vec![(c, row)],
                None => bend_path(gu, gv, span_u, span_v, row),
            }
        }
    }
}

/// The single gap column between two same-band neighbours: east of the
/// source's column span, or west of the target's (`None` when the pair
/// is not adjacent).
fn straight_gap(
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
fn selfloop_path(su: Cell, gu: Cell, rows: &Rows) -> Vec<Cell> {
    match rows.gap_rows[su.1].first().copied() {
        Some(g) => vec![(gu.0, g)],
        None => Vec::new(),
    }
}

/// Band of a same-band bend's gap row: above the band for odd parallel
/// channels (or any non-parallel bend), below it for even channels —
/// each channel takes its own gap row.
fn bend_gap_row(route: Route, band: usize, rows: &Rows, fallback: usize) -> usize {
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
fn bend_path(
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

/// Cell path of a multi-band route. East-going: along the source band to
/// the target column, then the full-row walk down (or up) the target
/// column. West-going: the recorded oracle A* shape (bg-edge-node's
/// Ulm->car) bends out of the source's north/south first — the full-row
/// walk at the source column toward the gap row adjacent to the target
/// band — then runs along that gap row, ending at the target column (the
/// source-band walk would drive through the intervening node cells).
/// Both vertical walks jog east around band rows occupied by a stacked
/// node (gently-4ga) — the gap-only walk jumped the occupied band row
/// between two reserved gap rows, rendering non-adjacent paths.
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
fn west_run_path(gu: Cell, span_cy: usize, run: &[Cell], g: usize, ctx: &Ctx) -> Option<Vec<Cell>> {
    let gu0 = gu.0;
    let mut path = walk_column(gu0, walk_from(gu.1, span_cy, g), g, ctx);
    let Some(&(dc, dr)) = path.last() else {
        // The walk range is empty: the run row is adjacent to the
        // source band, so the run's first cell is adjacent to the
        // source's attachment (gently-0is: edge 68 rendered fine before
        // the helper — replicate that shape).
        if run.is_empty() {
            if !ctx.occupied.contains(&(gu0, g)) {
                path.push((gu0, g));
            }
            return Some(path);
        }
        path.extend(run.iter().copied());
        return Some(path);
    };
    if dr == g {
        // Dodge-column close on the run row. Bridging east to the
        // source column and then running back west would 180°-turn on
        // the spot (gently-0is: edge 1603) — instead continue ALONG
        // the run row toward the run's west end (the target's east-face
        // entry cell). The gap row holds no node cells, so the travel
        // is free whatever the dodge column; cells east of the close
        // are simply not drawn (any valid route is fine at this tier).
        match run.last() {
            Some(&(west_end, _)) => {
                if dc > west_end {
                    path.extend((west_end..dc).rev().map(|c| (c, g)));
                } else if dc < west_end {
                    path.extend((dc + 1..=west_end).map(|c| (c, g)));
                }
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
    } else if dr.abs_diff(g) == 1 && dc == gu0 {
        // Un-dodged end one row off the run row at the walk column.
        if run.is_empty() {
            if !ctx.occupied.contains(&(gu0, g)) {
                path.push((gu0, g));
            }
            return Some(path);
        }
        path.extend(run.iter().copied());
        Some(path)
    } else {
        // The give-up break stopped the walk short of the run row.
        None
    }
}

fn multiband_path(ep: &Endpoints, rows: &Rows, ctx: &Ctx) -> Vec<Cell> {
    let (su, sv) = (ep.su, ep.sv);
    let (gu, gv) = (ep.gu, ep.gv);
    let (span_u, span_v) = (ep.span_u, ep.span_v);
    if gu.0 > gv.0 {
        let north = sv.1 < su.1;
        // The gap row adjacent to the target band hosts the horizontal
        // run; the vertical walk covers every row between the source and
        // that gap row at the source column. Primary candidate (north):
        // one band south of the target — the recorded Ulm->car oracle
        // bend (the row just south of the TARGET band can sit inside a
        // stacked node's span, as with Hof's box). But once earlier
        // bands reserved gap rows, that boundary's first row can fall
        // INSIDE a later stacked node (gently-hpd: edge 25's primary
        // run rode row 8 straight through nodes 11 and 29) — so the
        // run is validated against the occupied set, and a dirty
        // primary falls back to the target band's own boundary gap row
        // (gap_rows[sv.1]: the rows between the target band and the
        // next, the geometrically adjacent corridor).
        let candidates: &[usize] = if north { &[sv.1 + 1, sv.1] } else { &[sv.1] };
        let mut primary: Option<usize> = None;
        for (attempt, &home) in candidates.iter().enumerate() {
            let Some(&g) = rows.gap_rows.get(home).and_then(|v| v.first()) else {
                continue;
            };
            let run: Vec<Cell> = ((gv.0 + span_v.0 - 1)..=gu.0)
                .rev()
                .map(|c| (c, g))
                .collect();
            if attempt == 0 {
                primary = Some(g);
            }
            if run.iter().all(|&c| !ctx.occupied.contains(&c)) {
                if let Some(path) = west_run_path(gu, span_u.1, &run, g, ctx) {
                    return path;
                }
                // The walk gave up short of this run row: fall through
                // to the next candidate (gently-0is).
            }
        }
        return match primary {
            Some(g) => {
                let run: Vec<Cell> = ((gv.0 + span_v.0 - 1)..=gu.0)
                    .rev()
                    .map(|c| (c, g))
                    .collect();
                west_run_path(gu, span_u.1, &run, g, ctx)
                    .unwrap_or_else(|| walk_column(gu.0, walk_from(gu.1, span_u.1, g), g, ctx))
            }
            None => {
                // Span-hosted attachment (gently-n0d): no gap row exists
                // at `home` because the endpoints' ROW SPANS OVERLAP —
                // the corridor-clear accounting (gently-a6g) reserved no
                // gap rows, the walk rides the target's own span rows.
                // Vertical at the source's column to the target's band
                // row, then west into the target's east face — the run
                // stops one column east of the face, so no node cell is
                // entered and the last path cell abuts the span.
                let mut path = walk_column(gu.0, walk_from(gu.1, span_u.1, gv.1), gv.1, ctx);
                let east_face = gv.0 + span_v.0;
                let dodge_closed =
                    matches!(path.last(), Some(&(dc, dr)) if dr == gv.1 && dc != gu.0);
                if dodge_closed {
                    // Dodge-column close on the band row: continue ALONG
                    // the band row to the run's west end (east_face, the
                    // cell abutting the target's east border). Bridging
                    // east to the source column and running back west
                    // would 180°-turn (gently-0is: edge 1603). The band
                    // row can hold the target's own cells — validate the
                    // corridor and truncate rather than jump.
                    let Some(&(dc, _)) = path.last() else {
                        return path;
                    };
                    let (lo, hi) = if dc > east_face {
                        (east_face, dc)
                    } else {
                        (dc, east_face)
                    };
                    if !(lo..=hi).all(|c| !ctx.occupied.contains(&(c, gv.1))) {
                        return path; // corridor blocked: truncate
                    }
                    if dc > east_face {
                        path.extend((east_face..dc).rev().map(|c| (c, gv.1)));
                    } else {
                        path.extend((dc + 1..=east_face).map(|c| (c, gv.1)));
                    }
                    return path;
                }
                match path.last() {
                    Some(&(dc, dr)) if dr.abs_diff(gv.1) == 1 && dc == gu.0 => {
                        // The walk is one row short of the band row:
                        // close the final cell at the source column so
                        // the path abuts the target's face.
                        if !ctx.occupied.contains(&(gu.0, gv.1)) {
                            path.push((gu.0, gv.1));
                        }
                    }
                    _ => return path, // gave up: truncate, never jump
                }
                path.extend((east_face..gu.0).rev().map(|c| (c, gv.1)));
                path
            }
        };
    }
    // The straight run east at the source's band row, truncated at the
    // first occupied cell — a sibling target stacked at the source's row
    // (the fan-out's first target, gently-206) may not be driven
    // through; the walk then continues down the last free column and
    // steps east into the target's column at the face-adjacent row.
    let mut path: Vec<Cell> = Vec::new();
    let mut walk_col = gv.0;
    if gu.0 < gv.0 {
        // When the run's row lies INSIDE the target's row span (a grown
        // target whose bottom face row is the source's band row), the
        // run stops west of the span — entering the target's own
        // columns at that row would drive into its cells; the
        // face-adjacent approach run below connects instead.
        let capped_end = if gu.1 >= gv.1 && gu.1 < gv.1 + span_v.1 {
            gv.0.saturating_sub(1)
        } else {
            gv.0
        };
        if capped_end >= gu.0 + span_u.0 {
            for c in gu.0 + span_u.0..=capped_end {
                if ctx.occupied.contains(&(c, gu.1)) {
                    break;
                }
                path.push((c, gu.1));
                walk_col = c;
            }
        }
    }
    // The walk stops at the target's face row nearest the source
    // (gently-a6g): a grown target entered from the south is met at its
    // bottom span row — the walk never crosses the target's own cells.
    let face = if gu.1 > gv.1 {
        gv.1 + span_v.1 - 1
    } else {
        gv.1
    };
    // A walk down the source's OWN column must start below the
    // source's row span — the band row is the span's top row, and the
    // span's cells would force a jog with no lane at the grid's west
    // edge (gently-0is: edge 68). Walks from columns east of the span
    // (the advanced walk_col) keep the band row — the span is not in
    // their way, and the pinned tier-1 shapes start there.
    let from = if walk_col >= gu.0 && walk_col < gu.0 + span_u.0 {
        walk_from(gu.1, span_u.1, face)
    } else {
        gu.1
    };
    path.extend(walk_column(walk_col, from, face, ctx));
    if walk_col != gv.0 {
        // The walk ended short of the target's column: run east (or
        // west) along the face-adjacent row into the target's column,
        // one cell per column, so the last path cell abuts the
        // target's north (or south) face. A blocked cell aborts the
        // run — the renderer reports the shape as a typed error rather
        // than drawing through a node (gently-206).
        let approach = if gu.1 > gv.1 { face + 1 } else { face - 1 };
        let mut cells: Vec<Cell> = if walk_col < gv.0 {
            (walk_col + 1..=gv.0).map(|c| (c, approach)).collect()
        } else {
            ((gv.0..walk_col).rev().map(|c| (c, approach))).collect()
        };
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
        if cells.iter().all(|&c| !ctx.occupied.contains(&c)) {
            path.extend(cells);
        } else if span_v.1 > 1 {
            // The face-adjacent approach is taken (gently-0is): a fan-in
            // hub with a sibling stacked north/south of its band leaves
            // no free approach cell at the target's column. Ride the
            // target's own span rows at the walk column instead (the
            // west-side mirror of the span-hosted attachment, gently-n0d
            // west branch): the walk column lies strictly west of the
            // span, so the rows outside the box are approachable — the
            // last path cell abuts the target's WEST face on a row
            // inside its span (the entry's west-side rule, gently-206).
            // North of the target the ride extends south to the span's
            // bottom row; south of it, north to the first row inside
            // the span. Any blocked ride row keeps the typed error
            // rather than drawing through a node.
            let ride: Vec<Cell> = if gu.1 > gv.1 {
                (gv.1 + 1..=face).rev().map(|r| (walk_col, r)).collect()
            } else {
                (face..=gv.1 + span_v.1 - 1)
                    .map(|r| (walk_col, r))
                    .collect()
            };
            if ride.iter().all(|&c| !ctx.occupied.contains(&c)) {
                path.extend(ride);
            }
        }
    }
    path
}

/// Push the horizontal corridor between two columns at one row: every
/// column strictly between the current column and the target, plus the
/// target itself. A WIDE jog lane (k > 1) validated by `walk_column`'s
/// `lanes` skips intermediate columns — without this run the path
/// stitched non-adjacent pairs at every wide jog (gently-0is: edges
/// 0/1/7 of the whole-corpus projection). The lane's validation already
/// checked the whole corridor free at that row; callers bridging onto a
/// row without that validation must check the corridor themselves.
/// The first row a south-going vertical walk may occupy: below the
/// source's own row span (the band row is the span's TOP row, so the
/// span's cells sit between the band row and the walk's far row).
/// Walking from the band row would jog around the source's own cells —
/// and at the grid's west edge there is no lane to jog to, so the walk
/// breaks empty (gently-0is: edge 68, a same-column south edge from a
/// 3-row source at column 0). Only a walk whose far row lies strictly
/// below the span is adjusted; a rider inside the span keeps the band
/// row (the pinned tier-1 shapes).
fn walk_from(from: usize, span_cy: usize, to: usize) -> usize {
    if to > from && to >= from + span_cy.max(1) {
        from + span_cy.max(1) - 1
    } else {
        from
    }
}

fn push_corridor(path: &mut Vec<Cell>, from_col: usize, to_col: usize, row: usize) {
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
fn walk_column(col: usize, from: usize, to: usize, ctx: &Ctx) -> Vec<Cell> {
    let rows: Vec<usize> = if to > from {
        (from + 1..to).collect()
    } else {
        ((to + 1)..from).rev().collect()
    };
    let mut path: Vec<Cell> = Vec::new();
    let mut broke = false;
    // The jog lane: a free column beside the walked one. The east gap
    // column is the oracle's lane (odd columns never hold BASE cells),
    // but a grown span can cover it (gently-0is: the corpus's acyclic
    // hub grows across the gap column) — so the lane is validated per
    // stretch and falls back west when east is taken.
    let mut dodging: Option<usize> = None;
    let lanes = |from_col: usize, prev: usize, r: usize, ctx: &Ctx| {
        // The oracle's lane (one east) first, then west, then wider —
        // grown spans can wall off several columns (gently-0is). A wider
        // lane is valid only when the whole horizontal corridor at BOTH
        // rows is free: the path walks from (from_col, prev) to (d, prev)
        // through every column between. The scan is unbounded (to the
        // grid edges): a corpus-scale wall can exceed any fixed lane
        // count, and the corridor check keeps every wider candidate
        // honest (gently-0is: the whole-corpus projection's row 99).
        (1..=ctx.max_x + 1).find_map(|k| {
            let east = from_col.checked_add(k).filter(|&d| d <= ctx.max_x);
            let west = from_col.checked_sub(k);
            [east, west].into_iter().flatten().find(|&d| {
                let lo = d.min(from_col);
                let hi = d.max(from_col);
                (lo..=hi).all(|c| !ctx.occupied.contains(&(c, prev)))
                    && !ctx.occupied.contains(&(d, r))
            })
        })
    };
    for (i, &r) in rows.iter().enumerate() {
        match (ctx.occupied.contains(&(col, r)), dodging) {
            (true, None) => {
                // Jog east on the previous (free) row, then step past the
                // blocked row in the gap column.
                let prev = path.last().map_or(from, |&(_, y)| y);
                match lanes(col, prev, r, ctx) {
                    Some(d) => {
                        push_corridor(&mut path, col, d, prev);
                        path.push((d, r));
                        dodging = Some(d);
                    }
                    // No free lane beside the walk: stop here rather than
                    // push through a node — the renderer reports the
                    // truncated path as a typed error (gently-0is).
                    None => {
                        broke = true;
                        break;
                    }
                }
            }
            (true, Some(d)) => {
                if ctx.occupied.contains(&(d, r)) {
                    // The lane itself is taken further down: switch to
                    // another free column beside the lane at this row.
                    let prev = path.last().map_or(from, |&(_, y)| y);
                    match lanes(d, prev, r, ctx) {
                        Some(d2) if d2 != col || !ctx.occupied.contains(&(col, r)) => {
                            push_corridor(&mut path, d, d2, prev);
                            path.push((d2, r));
                            dodging = Some(d2);
                        }
                        _ => {
                            broke = true;
                            break;
                        }
                    }
                } else {
                    path.push((d, r));
                }
            }
            (false, Some(d)) => {
                path.push((d, r));
                // Stay in the dodge column while the NEXT row is blocked
                // too (two stacked nodes one free row apart — the fan-in
                // star's below-hub stack, gently-206): returning to the
                // walk column for a single free row would step back east
                // on the very next row, reversing the path through the
                // dodge column — the renderer rejects a 180° turn as
                // 'turns without changing axis'.
                let next_blocked = rows
                    .get(i + 1)
                    .is_some_and(|&nr| ctx.occupied.contains(&(col, nr)));
                if !next_blocked {
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
                        push_corridor(&mut path, d, col, r);
                        dodging = None;
                    }
                }
            }
            (false, None) => path.push((col, r)),
        }
    }
    // The walk's terminal row was BLOCKED (a stacked multi-row node
    // fills the walked column down to the face — cli.t6 under cli.c5,
    // the corpus's e19 shape): the path sits in the dodge column one
    // row short of the face. Close through the dodge column to the
    // face row itself so the walk's endpoint stays grid-adjacent to
    // the face cell — the edge enters through the dodge-column side
    // (east face), never through the blocked face (gently-206).
    // Only close when the walk actually reached the terminal row — a
    // give-up break leaves the path mid-column, and a close here would
    // jump (gently-0is).
    if !broke {
        if let Some(d) = dodging {
            if !ctx.occupied.contains(&(d, to)) {
                path.push((d, to));
            }
        }
    }
    path
}

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
fn slot_path(ep: &Endpoints, ctx: &Ctx, used: &HashSet<Cell>, end: bool, n: usize) -> Vec<Cell> {
    let (gu, gv) = (ep.gu, ep.gv);
    let (span_u, span_v) = (ep.span_u, ep.span_v);
    if !end {
        // (B) start slot: east at the slot row, then north into the
        // target's south. The first cell is the gap column east of the
        // source's span (the node's own span covers the slot row).
        let att_row = gu.1 + n;
        let out = gu.0 + span_u.0;
        let mut path: Vec<Cell> = (out..=gv.0).map(|c| (c, att_row)).collect();
        // The vertical north: rows att_row-1 ..= band+cy_v (the first row
        // south of the target's box), descending — the last cell carries
        // the `^` arrow-end.
        let low = gv.1 + span_v.1;
        let mut r = att_row;
        while r > low {
            r -= 1;
            path.push((gv.0, r));
        }
        path
    } else {
        // (C) end slot: drop south at the source column, then west.
        let att_row = gv.1 + n;
        let top = gu.1 + span_u.1;
        let drop: Vec<Cell> = if top <= att_row {
            (top..=att_row).map(|r| (gu.0, r)).collect()
        } else {
            vec![(gu.0, att_row)]
        };
        let free = drop
            .iter()
            .all(|c| !ctx.occupied.contains(c) && !used.contains(c));
        if free {
            let mut path = drop;
            let east = gv.0 + span_v.0;
            let mut c = gu.0;
            while c > east {
                c -= 1;
                path.push((c, att_row));
            }
            path
        } else {
            // Dodge EAST: out of the source's east at the band row
            // through the east-gap channel column, down to the
            // attachment row, then west.
            let channel = gu.0 + span_u.0;
            let mut path: Vec<Cell> = ((gu.1)..=att_row).map(|r| (channel, r)).collect();
            let east = gv.0 + span_v.0;
            let mut c = channel;
            while c > east {
                c -= 1;
                path.push((c, att_row));
            }
            path
        }
    }
}
