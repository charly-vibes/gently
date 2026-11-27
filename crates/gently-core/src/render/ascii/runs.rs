//! Purpose: the straight-run edge drawers for ge.ascii_render — the
//! horizontal/vertical gap-arrow stampers and their run/label helpers
//! (upstream `_draw_hor`/`_draw_ver` decomposition). Rationale: moved
//! out of `edges.rs` past the gate file limit; every byte mirrors the
//! probed oracle exactly.

use super::canvas::{centered, Canvas};
use super::edges::EdgeNodeEnds;
use super::labels::{Align, LabelLine};

use super::styles::{self, byte_width, edge_fill, edge_run_at};

/// The run mirrored for a westward edge: reversed, head flipped.
fn mirror_run(run: &str) -> String {
    run.chars()
        .rev()
        .map(|c| if c == '>' { '<' } else { c })
        .collect()
}

/// Stamp the straight run of one edge: the style run centered in the gap
/// column, mirrored when the target lies west; a labelled run is the
/// style's fill pattern widened to the gap plus a `>` head (no head on an
/// undirected run — the oracle draws none, gently-bpq), with the label
/// centered on the box-top row (oracle bytes). A bidirectional edge
/// (gently-7qz) renders the BD run ` <fill> ` — both arrowheads, one
/// column wider — with the label in [2, w-3) of the widened cell
/// (upstream _insert_label's field).
///
/// Endpoint flag stripping for `shape: edge` nodes (gently-cbk, upstream
/// Scout.pm's short-cell rules): a shape:edge SOURCE loses every START
/// flag (the run hangs flush off its border — `--->` into Ulm), a
/// shape:edge TARGET loses the START flag on the edge's far side — for an
/// undirected run only `START_W` survives (the `--------` through
/// Bonn–train–Ulm); a DIRECTED run's arrowhead always survives (rent's
/// `>` rides into the car edge cell).
pub(super) fn draw_straight(
    canvas: &mut Canvas,
    sr: usize,
    c: usize,
    westward: bool,
    edge_index: usize,
    lines: &[LabelLine],
    ends: EdgeNodeEnds,
) {
    if (ends.src || ends.dst) && !canvas.edge_bidir[edge_index] {
        stamp_flagged(canvas, sr, c, westward, edge_index, lines, ends);
        return;
    }
    let run = straight_run_for(canvas, c, edge_index, lines, westward);
    canvas.stamp(
        canvas.band_top(sr) + canvas.row_height[sr] - 2,
        c,
        &centered(&run, canvas.col_width[c]),
    );
    stamp_label(canvas, sr, c, westward, lines);
}

/// The straight horizontal run for one edge at column `c`: bidirectional
/// edges use the style's own bidi run, labelless edges the arrow-aware
/// edge run, labelled edges the shared labelled run.
fn straight_run_for(
    canvas: &Canvas,
    c: usize,
    edge_index: usize,
    lines: &[LabelLine],
    westward: bool,
) -> String {
    let style = canvas.edge_style[edge_index].clone();
    let width = canvas.col_width[c];
    let arrowed = canvas.edge_arrow_end[edge_index];
    let bidir = canvas.edge_bidir[edge_index];
    let run = if bidir {
        styles::bd_run_at(&style, width, canvas.offsets[c])
    } else if lines.is_empty() {
        edge_run_at(&style, arrowed, width, canvas.offsets[c])
    } else {
        labelled_straight_run(&style, width, arrowed)
    };
    let west = westward && !canvas.edge_bidir[edge_index];
    if west {
        mirror_run(&run)
    } else {
        run
    }
}

/// Stamp the shape:edge-stripped straight run (gently-cbk): the effective
/// `_draw_hor` START flags after the Scout.pm stripping — eastward the
/// source's `START_W` dies with a shape:edge source and the undirected
/// `START_E` dies with a shape:edge target; westward the roles mirror
/// (`START_E` is the source's, `START_W` the undirected target's) — then
/// the run and its label, the run NOT centered (it hangs off the
/// surviving border).
fn stamp_flagged(
    canvas: &mut Canvas,
    sr: usize,
    c: usize,
    westward: bool,
    edge_index: usize,
    lines: &[LabelLine],
    ends: EdgeNodeEnds,
) {
    let style = canvas.edge_style[edge_index].clone();
    let arrowed = canvas.edge_arrow_end[edge_index];
    let undirected = !arrowed && !canvas.edge_bidir[edge_index] && !start_arrow(canvas, edge_index);
    let (start_w, start_e) = if westward {
        (undirected && !ends.src && !ends.dst, !ends.src)
    } else {
        (!ends.src, undirected && !ends.src && !ends.dst)
    };
    let (run, x) = flagged_run(
        &style,
        canvas.col_width[c],
        canvas.offsets[c],
        arrowed,
        westward,
        undirected,
        (start_w, start_e),
    );
    let line = canvas.band_top(sr) + canvas.row_height[sr] - 2;
    canvas.stamp_at(line, canvas.offsets[c] + x, &run);
    stamp_label(canvas, sr, c, westward, lines);
}

/// Whether the edge carries a start-side arrowhead (a `<--` edge): the
/// canvas records only the end arrow and the bidirectional pair, so the
/// start-only arrow is recovered from the graph-side flag.
fn start_arrow(canvas: &Canvas, edge_index: usize) -> bool {
    canvas.edge_start_arrow[edge_index]
}

/// The straight short-cell run for an explicitly flagged cell (upstream
/// `_draw_hor` with the shape:edge-stripped flags, gently-cbk): the style
/// pattern fills the width (the undirected `rx % len` ofs shift applies —
/// directed SHORT cells skip it), one char drops per present START flag
/// (`START_W` also shifts the run one cell east), and a directed end
/// flag chops one more and replaces the new last char with the head.
/// Returns the run and its cell-x offset — a stripped cell is NOT
/// centered; the run hangs off the surviving border.
fn flagged_run(
    style: &str,
    w: usize,
    rx: usize,
    arrowed: bool,
    westward: bool,
    undirected: bool,
    (start_w, start_e): (bool, bool),
) -> (String, usize) {
    let pat = styles::edge_pattern(style);
    let len = pat.len();
    let mut line = pat.repeat(2 + w / len);
    if undirected {
        let ofs = rx % len;
        if ofs != 0 {
            line.replace_range(..ofs, "");
        }
    }
    line.truncate(w);
    let mut x = 0;
    if start_w {
        line.pop();
        x = 1;
    }
    if start_e {
        line.pop();
    }
    stamp_run_head(&mut line, arrowed, westward);
    (line, x)
}

/// The arrowhead on a flagged run: END_W rides the run's first two cells
/// (` <` + fill); END_E chops one more, then the head takes the new last
/// character.
fn stamp_run_head(line: &mut String, arrowed: bool, westward: bool) {
    if westward {
        if arrowed {
            line.replace_range(..2, " <");
        }
    } else if arrowed {
        line.pop();
        line.pop();
        line.push('>');
    }
}

/// Place a straight edge's aligned label lines in its gap column
/// (gently-527/gently-y7i): the field starts at byte offset 2 (upstream
/// `_draw_hor`'s `xs = 2`), is `width-5` bytes wide eastward (`ws =
/// xs + xr = 3`, the east arrowhead reserving the last byte) and
/// `width-4` westward (`ws = 2`, the west head reserving byte 1), and
/// the rows sit bottom-aligned in the band's `H-2` field lines
/// (upstream `align_ver => 'bottom'`) — a single line rides the band's
/// top row, two lines ride the top two rows. Widths count UTF-8 bytes
/// (gently-bvb).
fn stamp_label(canvas: &mut Canvas, sr: usize, c: usize, westward: bool, lines: &[LabelLine]) {
    if lines.is_empty() {
        return;
    }
    let field_w = canvas.col_width[c] - 5 + usize::from(westward);
    let start = canvas.band_top(sr) + canvas.row_height[sr].saturating_sub(2 + lines.len());
    for (i, line) in lines.iter().enumerate() {
        let len = byte_width(&line.text);
        let x = 2 + match line.align {
            Align::Left => 0,
            Align::Center => (field_w.saturating_sub(len)) / 2,
            Align::Right => field_w.saturating_sub(len),
        };
        canvas.stamp_at(start + i, canvas.offsets[c] + x, &line.text);
    }
}

/// The labelled straight run: the style's fill pattern widened to the
/// gap, with a `>` head only when the target end carries an arrowhead
/// (gently-bpq: the oracle draws no head on an undirected labeled run).
fn labelled_straight_run(style: &str, width: usize, arrowed: bool) -> String {
    let run_len = width - 2;
    let fill = edge_fill(style);
    let chars: Vec<char> = fill.chars().collect();
    let head = usize::from(arrowed);
    let mut s: String = (0..run_len - head)
        .map(|i| chars[i % chars.len()])
        .collect();
    if head == 1 {
        s.push('>');
    }
    s
}
/// The target-adjacent row index of a vertical gap-row run: 0 when the
/// target sits above (the run's first row), h-1 when it sits below.
/// END_N puts the head on the run's first (target-adjacent) row, END_S
/// on its last — an upward run heads its first char, a downward one
/// its last; a bidirectional run heads both ends.
fn stamp_ver_heads(
    canvas: &mut Canvas,
    top: usize,
    x: usize,
    h: usize,
    upward: bool,
    arrowed: bool,
    bidir: bool,
) {
    let head_row = target_row(upward, h);
    if arrowed {
        canvas.grid[top + head_row][x] = if upward { b'^' } else { b'v' };
    }
    if bidir {
        canvas.grid[top + h - 1 - head_row][x] = if upward { b'v' } else { b'^' };
    }
}

fn target_row(upward: bool, h: usize) -> usize {
    if upward {
        0
    } else {
        h - 1
    }
}

/// Stamp the straight run of one VERTICAL edge through its single gap row
/// (gently-cbk, upstream `_draw_ver` for the EDGE_VER short cell): pipes
/// at the fixed offset-2 column down the gap band's full height, the
/// arrowhead at the target-adjacent end (`^` entering the target's south
/// from below, `v` its north from above; both for a bidirectional edge),
/// and the label lines left of the pipe on the band's middle line at
/// byte offset 4 (upstream `_insert_label`'s `xs = 4`) — left-aligned
/// (the edge `align` default), bottom-anchored around the middle line.
/// START flags never render on a vertical run (upstream: "starting
/// points are currently invisible").
pub(super) fn draw_straight_ver(
    canvas: &mut Canvas,
    gap_row: usize,
    c: usize,
    upward: bool,
    edge_index: usize,
    lines: &[LabelLine],
) {
    let h = canvas.row_height[gap_row];
    let top = canvas.band_top(gap_row);
    let x = canvas.vert_x(c);
    let arrowed = canvas.edge_arrow_end[edge_index];
    let bidir = canvas.edge_bidir[edge_index];
    for r in &mut canvas.grid[top..top + h] {
        r[x] = b'|';
    }
    stamp_ver_heads(canvas, top, x, h, upward, arrowed, bidir);
    if lines.is_empty() {
        return;
    }
    let mid = canvas.mid(gap_row);
    let start = mid - (lines.len() - 1) / 2;
    for (i, line) in lines.iter().enumerate() {
        canvas.stamp_at(start + i, canvas.offsets[c] + 4, &line.text);
    }
}

// The per-edge dispatch and the `shape: edge` node cell moved here from
// canvas.rs past the tidy file limit (gently-cbk).
