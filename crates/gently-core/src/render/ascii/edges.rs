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

use super::canvas::{centered, BAND_HEIGHT, Canvas};
use super::styles::{self, edge_fill, edge_run_at};

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
/// column wider — with the label centered in [2, w-3) of the widened cell
/// (upstream _insert_label's field).
pub(super) fn draw_straight(
    canvas: &mut Canvas,
    sr: usize,
    c: usize,
    westward: bool,
    edge_index: usize,
    label: Option<String>,
) {
    let style = canvas.edge_style[edge_index].clone();
    let width = canvas.col_width[c];
    let arrowed = canvas.edge_arrow_end[edge_index];
    let run = if canvas.edge_bidir[edge_index] {
        if let Some(label) = &label {
            canvas.stamp(
                sr * BAND_HEIGHT,
                c,
                &format!("  {}", centered(label, width - 5)),
            );
        }
        styles::bd_run_at(&style, width, canvas.offsets[c])
    } else {
        plain_straight_run(canvas, sr, c, &style, width, arrowed, label.as_deref())
    };
    let run = if westward && !canvas.edge_bidir[edge_index] {
        mirror_run(&run)
    } else {
        run
    };
    canvas.stamp(sr * BAND_HEIGHT + 1, c, &centered(&run, width));
}

/// The plain (non-bidirectional) straight run of one edge: the style's
/// minimum run (arrowed or not), or — when the edge carries a label — the
/// fill pattern widened to the gap, with a `>` head only when the target
/// end carries an arrowhead (gently-bpq: the oracle draws no head on an
/// undirected labeled run). The label stamps centered on the box-top row
/// (oracle bytes).
fn plain_straight_run(
    canvas: &mut Canvas,
    sr: usize,
    c: usize,
    style: &str,
    width: usize,
    arrowed: bool,
    label: Option<&str>,
) -> String {
    match label {
        Some(label) => {
            let run_len = width - 2;
            let fill = edge_fill(style);
            let chars: Vec<char> = fill.chars().collect();
            let head = if arrowed { 1 } else { 0 };
            let mut s: String = (0..run_len - head)
                .map(|i| chars[i % chars.len()])
                .collect();
            if head == 1 {
                s.push('>');
            }
            canvas.stamp(sr * BAND_HEIGHT, c, &centered(label, width));
            s
        }
        None => edge_run_at(style, arrowed, width, canvas.offsets[c]),
    }
}