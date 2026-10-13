//! Purpose: straight-edge run drawing for the ascii renderer
//! (gently-7qz) — the glyph runs a single-gap straight edge stamps.
//! Responsibilities: render a straight edge's gap column — the BD run
//! (both arrowheads, upstream _draw_hor END_W/END_E) or the plain style
//! run (arrowed or not, gently-bpq: no head on an undirected labeled
//! run) — mirroring westward runs, and hosting labels on the box-top row
//! per the upstream _insert_label field rule.
//! Rationale: extracted from canvas.rs past the tidy file limit; the run
//! rules live in styles.rs, pinned by the recorded claims
//! (pod-bidir*.observed, pod-label-attr.observed).

use super::canvas::{centered, Canvas};
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
pub(super) fn draw_straight(
    canvas: &mut Canvas,
    sr: usize,
    c: usize,
    westward: bool,
    edge_index: usize,
    lines: &[LabelLine],
) {
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
    let run = if westward && !bidir {
        mirror_run(&run)
    } else {
        run
    };
    canvas.stamp(canvas.band_top(sr) + canvas.row_height[sr] - 2, c, &centered(&run, width));
    stamp_label(canvas, sr, c, westward, lines);
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
        let x = 2
            + match line.align {
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