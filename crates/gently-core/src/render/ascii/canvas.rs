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

use super::geometry::{
    band_layout, column_geometry, column_offsets, edge_styles, label_lines, node_styles, selfloops,
};
use super::groups::{self, Frame};
use super::labels::{Align, LabelLine};
use super::polyline;
use super::styles::{self, border, byte_width, h_run, Shape};
use super::RenderError;
use crate::graph::Graph;
use crate::layout::Layout;

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
    band_tops: Vec<usize>,
    /// Start char offset of each grid column (`offsets[c]`; length width+1).
    pub(super) offsets: Vec<usize>,
    /// Char width of each grid column.
    pub(super) col_width: Vec<usize>,
    /// Resolved label lines per node, in model order (gently-527/y7i).
    node_lines: Vec<Vec<LabelLine>>,
    /// Aligned label lines per edge, in model order — `Some` only for a
    /// straight horizontal labelled edge (the full-gap label cell);
    /// bent and vertical edges keep their single-line surface.
    edge_lines: Vec<Option<Vec<LabelLine>>>,
    /// Resolved render data per node, in model order.
    node_style: Vec<(String, Shape)>,
    /// Resolved edge style per edge, in model order.
    pub(super) edge_style: Vec<String>,
    /// Target-side arrowhead presence per edge, in model order.
    pub(super) edge_arrow_end: Vec<bool>,
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

/// The point node's label row: blank sides with a centered `*`.
fn centered_point_row(inner: usize) -> String {
    let left = (inner - 1) / 2;
    format!(" {}*{} ", " ".repeat(left), " ".repeat(inner - 1 - left))
}

/// The style's border with all four corners blanked (the `rounded`
/// shape's ASCII rendering: ` --- ` top and bottom rows).
fn blank_corners(b: styles::Border) -> styles::Border {
    styles::Border { tl: ' ', tr: ' ', br: ' ', bl: ' ', ..b }
}

/// `text` centered in a span of `width` byte cells.
pub(super) fn centered(text: &str, width: usize) -> String {
    let len = super::styles::byte_width(text);
    let left = (width - len) / 2;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(width - len - left))
}

/// The last band's box-bottom line: trailing blank lines beyond it never
/// render — the oracle's as_ascii ends at the last drawn line (the port-*
/// dip-below fixtures end at their gap-row run), while a trailing blank
/// WITHIN the last band still renders (the border:none node pins
/// `\n x\n\n`).
fn last_band_floor(
    layout: &Layout,
    row_height: &[usize],
    band_tops: &[usize],
    grid_len: usize,
    mt: usize,
) -> usize {
    layout
        .node_cells
        .iter()
        .map(|&(_, r)| band_tops[r] + row_height[r] - 1 + mt)
        .max()
        .map(|line| line.min(grid_len.saturating_sub(1)))
        .unwrap_or(0)
}

/// Shift the column offsets by the left margin and allocate the blank
/// byte grid padded by the right/bottom margins.
fn shifted_grid(
    offsets: Vec<usize>,
    width: usize,
    ml: usize,
    mr: usize,
    mt: usize,
    mb: usize,
    total_lines: usize,
) -> (Vec<Vec<u8>>, Vec<usize>) {
    let offsets: Vec<usize> = offsets.into_iter().map(|o| o + ml).collect();
    let total_width = offsets[width] + mr;
    let grid = vec![vec![b' '; total_width]; total_lines + mt + mb];
    (grid, offsets)
}

/// The group-frame margins (c8), the blank byte grid padded by them,
/// the offsets shifted by the left margin, and the last drawable line.
#[allow(clippy::type_complexity)]
fn frame_grid(
    graph: &Graph,
    layout: &Layout,
    col_width: &[usize],
    offsets: &[usize],
    band_tops: &[usize],
    row_height: &[usize],
) -> (Vec<Vec<u8>>, Vec<usize>, Vec<Frame>, usize, usize, usize) {
    let (frames, margins) =
        groups::frame_margins(graph, layout, col_width, offsets, band_tops, row_height);
    let (ml, mt, mr, mb) = margins;
    let total_lines = band_tops[layout.height];
    let (grid, offsets) =
        shifted_grid(offsets.to_vec(), layout.width, ml, mr, mt, mb, total_lines);
    let last_band_floor = last_band_floor(layout, row_height, band_tops, grid.len(), mt);
    (grid, offsets, frames, ml, mt, last_band_floor)
}

/// Target-side arrowhead presence and bidirectionality per edge, in
/// model order.
fn edge_arrow_flags(graph: &Graph) -> (Vec<bool>, Vec<bool>) {
    let end = graph.edges.iter().map(|e| e.arrows.end).collect();
    let bidir = graph.edges.iter().map(|e| e.arrows.start && e.arrows.end).collect();
    (end, bidir)
}

impl Canvas {
    /// Build the blank grid: column widths per the byte-grid mapping
    /// (widened for labelled straight edges, whose gap hosts the label
    /// rows — oracle: `+---+  put msg    +---+` over ` ----------> `),
    /// the row-band heights from the label lines, the group-frame
    /// margins, then one blank byte row per canvas line.
    pub(super) fn new(graph: &Graph, layout: &Layout) -> Canvas {
        let selfloop_node = selfloops(graph);
        let node_style = node_styles(graph);
        let edge_style = edge_styles(graph);
        // label lines resolve FIRST (gently-emd: the wrapped lines drive
        // the box widths), then the column geometry consumes them
        let (node_lines, edge_lines) = label_lines(graph, layout);
        let col_width =
            column_geometry(graph, layout, &node_style, &selfloop_node, &edge_style, &node_lines);
        let offsets = column_offsets(&col_width);
        // Row band heights + their unshifted start lines: every cell is
        // grown to the tallest cell of its row (upstream
        // `_prepare_layout` rewrites each cell's h to the row height).
        let (row_height, band_tops) = band_layout(graph, layout, &node_lines, &edge_lines);
        let (grid, offsets, frames, ml, mt, last_band_floor) =
            frame_grid(graph, layout, &col_width, &offsets, &band_tops, &row_height);
        let (edge_arrow_end, edge_bidir) = edge_arrow_flags(graph);
        Canvas {
            grid,
            row_height,
            band_tops,
            node_lines,
            edge_lines,
            offsets,
            col_width,
            node_style,
            edge_style,
            edge_arrow_end,
            edge_bidir,
            frames,
            margin_left: ml,
            margin_top: mt,
            last_band_floor,
        }
    }

    /// Stamp every node's box per its resolved shape: bordered box,
    /// rounded box (blank corners), point (`*`), or invisible (nothing).
    pub(super) fn draw_nodes(&mut self, _graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            let lines = self.node_lines[idx].clone();
            let (style, shp) = self.node_style[idx].clone();
            let b = border(&style);
            match shp {
                Shape::Invisible => {}
                Shape::Point => self.draw_point(cy, cx),
                Shape::Rounded => self.draw_box(cy, cx, &lines, blank_corners(b), false),
                Shape::Box => self.draw_box(cy, cx, &lines, b, style == "none"),
            }
        }
    }

    /// Stamp one bordered box: top border, the label rows (one per
    /// aligned-label line, gently-527/y7i), bottom border. The box fills
    /// the row band's height — a multiline label or a taller
    /// neighbor's label grows the band, and the label rows center
    /// vertically in the grown box (upstream `_draw_label`'s middle
    /// alignment with `ys = 0.5`: rows `1 + int((H-2-lines)/2 + i)` for
    /// a bordered box). Each line is placed by its `l`/`r`/`c`
    /// alignment inside the label field (`x0 = 2`, width `w-4`;
    /// `none` borders widen the field to `w-2` at `x0 = 1`); widths
    /// count UTF-8 bytes (gently-bvb). `none` is a `border: none` box:
    /// its field widens to `w-2` at `x0 = 1` (upstream `_draw_label`).
    fn draw_box(&mut self, cy: usize, cx: usize, lines: &[LabelLine], b: styles::Border, none: bool) {
        let w = self.col_width[cx];
        let inner = w - 2;
        let run = h_run(&b, inner);
        let top_line = self.band_top(cy);
        let h = self.row_height[cy];
        self.stamp(top_line, cx, &format!("{}{}{}", b.tl, run, b.tr));
        self.stamp(top_line + h - 1, cx, &format!("{}{}{}", b.bl, run, b.br));
        let (field_x, field_w, label_h, y_pad) = if none {
            (1, w - 2, h, 0)
        } else {
            (2, w - 4, h - 2, 1)
        };
        let start = top_line + y_pad + (label_h.saturating_sub(lines.len())) / 2;
        let x0 = self.offsets[cx];
        // the box's side glyphs run the FULL grown height — every
        // interior row carries them (the oracle's box is drawn border
        // first, then the label over it)
        for row in top_line + 1..top_line + h - 1 {
            self.stamp_at(row, x0, &b.side.to_string());
            self.stamp_at(row, x0 + w - 1, &b.side.to_string());
        }
        for (i, line) in lines.iter().enumerate() {
            let row = start + i;
            self.stamp_at(row, x0, &b.side.to_string());
            self.stamp_at(row, x0 + w - 1, &b.side.to_string());
            let len = byte_width(&line.text);
            let x = field_x
                + match line.align {
                    Align::Left => 0,
                    Align::Center => (field_w.saturating_sub(len)) / 2,
                    Align::Right => field_w.saturating_sub(len),
                };
            self.stamp_at(row, x0 + x, &line.text);
        }
    }

    /// Stamp a point node: blank borders, a centered `*` (oracle bytes:
    /// blank, `  *`, blank for a width-5 point node) on the band's
    /// vertically centered line.
    fn draw_point(&mut self, cy: usize, cx: usize) {
        let mid = centered_point_row(self.col_width[cx] - 2);
        self.stamp(self.mid(cy), cx, &mid);
    }

    /// Stamp the straight run of one edge (see `edges::draw_straight`).
    /// Draw one edge: selfloop elbow, straight gap arrow, or polyline.
    pub(super) fn draw_edge(
        &mut self,
        graph: &Graph,
        layout: &Layout,
        edge_index: usize,
    ) -> Result<(), RenderError> {
        let edge = &graph.edges[edge_index];
        let (sc, sr) = layout.node_cells[edge.from];
        let (tc, tr) = layout.node_cells[edge.to];
        let path = &layout.edge_paths[edge_index];

        if sc == tc && sr == tr {
            self.draw_selfloop((sc, sr));
            return Ok(());
        }
        if sr == tr && path.len() == 1 && sc.abs_diff(tc) == 2 {
            // Straight through the single gap column: centered run —
            // arrow-less or style-headed (c2), widened to a pattern fill
            // plus head when the edge carries a label (oracle bytes:
            // ` ----> ` under `  go  `); the label rides as its aligned
            // lines (gently-527/y7i).
            let lines = self.edge_lines[edge_index].clone().unwrap_or_default();
            super::edges::draw_straight(self, sr, path[0].0, tc < sc, edge_index, &lines);
            return Ok(());
        }

        polyline::draw(self, layout, edge_index, (sc, sr), (tc, tr), edge.arrows.start)
    }

    /// Draw a selfloop: a small elbow over the box, departing through the
    /// east corner and arriving through the west one with a `v` arrowhead.
    fn draw_selfloop(&mut self, (c, r): (usize, usize)) {
        let x0 = self.offsets[c];
        let w = self.col_width[c];
        let y_mid = self.mid(r - 1);
        let y_attach = self.band_top(r) - 1;
        let (xl, xr) = (x0 + 2, x0 + w - 3);
        for x in xl + 1..xr {
            self.grid[y_mid][x] = b'-';
        }
        self.grid[y_mid][xl] = b'+';
        self.grid[y_mid][xr] = b'+';
        self.grid[y_attach][xr] = b'|';
        self.grid[y_attach][xl] = b'v';
    }

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

    /// Char line one past row `r`'s bottom border line — the next band's
    /// top border (for the standard 3-line band this is `top+3`, the
    /// southern dip line).
    pub(super) fn band_below(&self, r: usize) -> usize {
        self.band_top(r) + self.row_height[r]
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
