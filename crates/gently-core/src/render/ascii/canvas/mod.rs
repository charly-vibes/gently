//! Purpose: the char-grid canvas the ascii renderer draws on — the
//! blank grid, per-node box stamping, per-edge dispatch, selfloop
//! elbows and the polyline plumbing.
//! Responsibilities: build the blank grid from the layout's cell grid
//! (column widths per `geometry`, widened by group titles and expanded
//! by the group-frame margins), stamp bordered/rounded/point/invisible
//! nodes and edge runs, stamp the group frames, and join the lines with
//! per-line trailing-space trimming (the oracle does not pad lines).
//! Rationale: split from monolithic mod.rs past the tidy file limit; the
//! glyph tables live in styles.rs, the label/style/column resolution in
//! geometry.rs, the group frames in groups.rs (gently-b0z).

mod build;
mod draw;
mod stamp;

use super::groups::Frame;
use super::labels::LabelLine;
use super::styles::Shape;
use crate::graph::{Graph, Scope};

/// The grid-cell span of every node, in model order (gently-awd): the
/// `columns`/`rows` attributes as grid cells — upstream `_calc_size`
/// reads `abs($attr || 1)`, so an absent/empty/0 value is one cell.
pub(super) fn node_spans(graph: &Graph) -> Vec<(usize, usize)> {
    (0..graph.nodes.len())
        .map(|i| {
            let side = |key: &str| {
                graph
                    .get_attr(Scope::Node(i), key)
                    .and_then(|v| v.parse::<usize>().ok())
                    .filter(|&n| n > 0)
                    .unwrap_or(1)
            };
            (side("columns"), side("rows"))
        })
        .collect()
}

/// Width in chars of a gap (edge-routing) grid column.
pub(super) const GAP_WIDTH: usize = 5;
/// Minimum height in lines of one grid row's char band (border, label,
/// border) — grown when a label in the row spans more lines (gently-527:
/// upstream sizes every cell from its label's line count and grows the
/// row to the tallest cell, `Layout::Grid::_prepare_layout`).
pub(super) const BAND_HEIGHT: usize = 3;

/// The blank char grid plus the column-width mapping it is drawn against.
pub(super) struct Canvas {
    /// Byte grid: rows of UTF-8 bytes — label arithmetic counts UTF-8
    /// bytes the way the oracle's byte-string `length()` does
    /// (gently-bvb), so a wide glyph occupies its byte length in the
    /// row, not its display columns.
    pub(super) grid: Vec<Vec<u8>>,
    /// Char-line height of each grid row's band (≥ [`BAND_HEIGHT`]).
    pub(super) row_height: Vec<usize>,
    /// Unshifted start line of each band (prefix of `row_height`),
    /// before the group-frame margin shift.
    pub(super) band_tops: Vec<usize>,
    /// Start char offset of each grid column (`offsets[c]`; length width+1).
    pub(super) offsets: Vec<usize>,
    /// Char width of each grid column.
    pub(super) col_width: Vec<usize>,
    /// Resolved label lines per node, in model order (gently-527/y7i).
    pub(super) node_lines: Vec<Vec<LabelLine>>,
    /// Aligned label lines per edge, in model order — `Some` only for a
    /// straight horizontal labelled edge (the full-gap label cell);
    /// bent and vertical edges keep their single-line surface.
    pub(super) edge_lines: Vec<Option<Vec<LabelLine>>>,
    /// Resolved render data per node, in model order.
    pub(super) node_style: Vec<(String, Shape)>,
    /// Grid-cell span of every node, in model order (gently-awd).
    pub(super) spans: Vec<(usize, usize)>,
    /// Resolved edge style per edge, in model order.
    pub(super) edge_style: Vec<String>,
    /// Target-side arrowhead presence per edge, in model order.
    pub(super) edge_arrow_end: Vec<bool>,
    /// Source-side arrowhead presence per edge, in model order (gently-cbk:
    /// the undirected test for the shape:edge flag stripping).
    pub(super) edge_start_arrow: Vec<bool>,
    /// Bidirectionality (arrowheads at BOTH ends) per edge, in model order.
    pub(super) edge_bidir: Vec<bool>,
    /// Group frames in unshifted coordinates, stamped after the edges.
    pub(super) frames: Vec<Frame>,
    /// Left/top canvas margins the frames demanded.
    pub(super) margin_left: usize,
    pub(super) margin_top: usize,
    /// Last drawn line any node box can reach (its box-bottom border line
    /// or the border:none blank beneath the label) — trailing blank lines
    /// beyond it are dropped (see `new`).
    last_band_floor: usize,
}

/// The drawn geometry of one node: its anchor offset and top line plus
/// the summed width/height over its grid-cell span (gently-awd).
pub(super) struct NodeGeom {
    pub(super) x0: usize,
    pub(super) top: usize,
    pub(super) w: usize,
    pub(super) h: usize,
}

/// `text` centered in a span of `width` byte cells.
pub(super) fn centered(text: &str, width: usize) -> String {
    let len = super::styles::byte_width(text);
    let left = (width - len) / 2;
    format!(
        "{}{}{}",
        " ".repeat(left),
        text,
        " ".repeat(width - len - left)
    )
}

impl Canvas {
    /// Char line of band row `r`'s top border line (margin-shifted).
    pub(super) fn band_top(&self, r: usize) -> usize {
        self.band_tops[r] + self.margin_top
    }

    /// Char line of row `r`'s middle (label) line — the vertically
    /// centered line of the grown band (a one-line label centers at
    /// `int((H-1)/2)`, which is the standard band's `top+1`).
    pub(super) fn mid(&self, r: usize) -> usize {
        self.band_top(r) + (self.row_height[r] - 1) / 2
    }

    /// Char line one past a node box's bottom border over its row span
    /// (gently-awd) — for a 1×1 node this is `band_below`.
    pub(super) fn box_below(&self, r: usize, span_r: usize) -> usize {
        let h: usize = (r..r + span_r)
            .map(|rr| self.row_height.get(rr).copied().unwrap_or(0))
            .sum();
        self.band_top(r) + h
    }

    /// Char column of a vertical run through grid column `c`: offset +2
    /// from the cell's left edge — upstream edge cells draw their vertical
    /// line at fixed offset 2 (`As_ascii::_draw_cross`:
    /// `_printfb_ver($fb, 2, 0, $line)`), not at the node's center, so a
    /// wide box's north/south attachment sits 2 chars in from its left
    /// border (gently-klf: tests/repro/claims/pod-triangle.observed,
    /// pod-fan.observed).
    pub(super) fn vert_x(&self, c: usize) -> usize {
        self.offsets[c] + 2
    }

    /// Write `text` starting at grid column `cx`'s byte offset on `line`.
    pub(super) fn stamp(&mut self, line: usize, cx: usize, text: &str) {
        let x0 = self.offsets[cx];
        self.stamp_at(line, x0, text);
    }

    /// Write `text` starting at absolute byte offset `x` on `line`,
    /// bounds-guarded (never panics; typed render philosophy).
    pub(super) fn stamp_at(&mut self, line: usize, x: usize, text: &str) {
        if line >= self.grid.len() {
            return;
        }
        for (k, byte) in text.bytes().enumerate() {
            if x + k < self.grid[line].len() {
                self.grid[line][x + k] = byte;
            }
        }
    }

    /// Join the grid lines, each with one trailing newline. Trailing
    /// filler spaces are trimmed per line — the oracle does not pad lines
    /// out to the canvas width (the isolated node's box row is just
    /// `+---+`).
    pub(super) fn into_string(self) -> String {
        // Trailing blank lines never render: the oracle's as_ascii ends at
        // the last drawn line, so a trailing gap row's unused lines are
        // dropped (the port-* fixtures' dip-below runs end the output).
        let mut lines: &[Vec<u8>] = &self.grid;
        while lines.len() > self.last_band_floor + 1 {
            match lines.last() {
                Some(last) if last.iter().all(|&c| c == b' ') => {
                    lines = &lines[..lines.len() - 1];
                }
                _ => break,
            }
        }
        let mut out = String::new();
        for line in lines {
            let text = String::from_utf8_lossy(line);
            out.push_str(text.trim_end_matches(' '));
            out.push('\n');
        }
        out
    }
}
