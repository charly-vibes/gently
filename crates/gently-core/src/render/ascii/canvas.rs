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
    column_geometry, column_offsets, edge_styles, node_label, node_styles, object_attr, selfloops,
};
use super::groups::{self, Frame};
use super::polyline;
use super::styles::{self, border, display_width, h_run, Shape};
use super::RenderError;
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::Layout;

/// Width in chars of a gap (edge-routing) grid column.
pub(super) const GAP_WIDTH: usize = 5;
/// Height in lines of one grid row's char band (border, label, border).
pub(super) const BAND_HEIGHT: usize = 3;

/// The blank char grid plus the column-width mapping it is drawn against.
pub(super) struct Canvas {
    pub(super) grid: Vec<Vec<char>>,
    /// Start char offset of each grid column (`offsets[c]`; length width+1).
    pub(super) offsets: Vec<usize>,
    /// Char width of each grid column.
    pub(super) col_width: Vec<usize>,
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

/// `text` centered in a span of `width` chars.
pub(super) fn centered(text: &str, width: usize) -> String {
    let len = display_width(text);
    let left = (width - len) / 2;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(width - len - left))
}

impl Canvas {
    /// Build the blank grid: column widths per the char-grid mapping
    /// (widened for labelled straight edges, whose gap hosts the label on
    /// the box-top row — oracle: `+---+  go   +---+` over ` ----> `),
    /// then prefix offsets, then `height * BAND_HEIGHT` blank lines.
    pub(super) fn new(graph: &Graph, layout: &Layout) -> Canvas {
        let selfloop_node = selfloops(graph);
        let node_style = node_styles(graph);
        let edge_style = edge_styles(graph);
        let (mut col_width, title_widths) =
            column_geometry(graph, layout, &node_style, &selfloop_node, &edge_style);
        // group titles widen their leftmost member's column (c8)
        for (c, w) in title_widths {
            if c < col_width.len() {
                col_width[c] = col_width[c].max(w);
            }
        }
        let offsets = column_offsets(&col_width);
        // group frames demand canvas margins (c8)
        let frames = groups::frames(graph, layout, &col_width, &offsets);
        let (ml, mt, mr, mb) =
            groups::margins(&frames, offsets[layout.width], layout.height * BAND_HEIGHT);
        let offsets: Vec<usize> = offsets.into_iter().map(|o| o + ml).collect();
        let total_width = offsets[layout.width] + mr;
        let grid = vec![vec![' '; total_width]; layout.height * BAND_HEIGHT + mt + mb];
        let edge_arrow_end: Vec<bool> = graph.edges.iter().map(|e| e.arrows.end).collect();
        let edge_bidir: Vec<bool> = graph.edges.iter().map(|e| e.arrows.start && e.arrows.end).collect();
        Canvas {
            grid,
            offsets,
            col_width,
            node_style,
            edge_style,
            edge_arrow_end,
            edge_bidir,
            frames,
            margin_left: ml,
            margin_top: mt,
        }
    }

    /// Stamp every node's box per its resolved shape: bordered box,
    /// rounded box (blank corners), point (`*`), or invisible (nothing).
    pub(super) fn draw_nodes(&mut self, graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            let label = node_label(graph, idx).to_string();
            let (style, shp) = self.node_style[idx].clone();
            let b = border(&style);
            match shp {
                Shape::Invisible => {}
                Shape::Point => self.draw_point(cy, cx),
                Shape::Rounded => self.draw_box(cy, cx, &label, blank_corners(b)),
                Shape::Box => self.draw_box(cy, cx, &label, b),
            }
        }
    }

    /// Stamp one bordered box: top border, centered label row, bottom
    /// border (centering follows display width, ge.ascii_render.c5).
    fn draw_box(&mut self, cy: usize, cx: usize, label: &str, b: styles::Border) {
        let w = self.col_width[cx];
        let inner = w - 2;
        let run = h_run(&b, inner);
        let len = display_width(label);
        let left = (inner - len) / 2;
        let mid = format!(
            "{}{}{}{}{}",
            b.side,
            " ".repeat(left),
            label,
            " ".repeat(inner - len - left),
            b.side
        );
        let top_line = self.band_top(cy);
        self.stamp(top_line, cx, &format!("{}{}{}", b.tl, run, b.tr));
        self.stamp(top_line + 1, cx, &mid);
        self.stamp(top_line + 2, cx, &format!("{}{}{}", b.bl, run, b.br));
    }

    /// Stamp a point node: blank borders, a centered `*` (oracle bytes:
    /// blank, `  *`, blank for a width-5 point node).
    fn draw_point(&mut self, cy: usize, cx: usize) {
        let mid = centered_point_row(self.col_width[cx] - 2);
        let top_line = self.band_top(cy);
        self.stamp(top_line + 1, cx, &mid);
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
            // ` ----> ` under `  go  `).
            let label = object_attr(graph, Scope::Edge(edge_index), ObjectKind::Edge, "label")
                .map(str::to_string);
            super::edges::draw_straight(self, sr, path[0].0, tc < sc, edge_index, label);
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
            self.grid[y_mid][x] = '-';
        }
        self.grid[y_mid][xl] = '+';
        self.grid[y_mid][xr] = '+';
        self.grid[y_attach][xr] = '|';
        self.grid[y_attach][xl] = 'v';
    }

    /// Char line of band row `r`'s top border line (margin-shifted).
    pub(super) fn band_top(&self, r: usize) -> usize {
        r * BAND_HEIGHT + self.margin_top
    }

    /// Char line of row `r`'s middle (label) line.
    pub(super) fn mid(&self, r: usize) -> usize {
        self.band_top(r) + 1
    }

    /// Center char of column `c`.
    pub(super) fn center(&self, c: usize) -> usize {
        self.offsets[c] + self.col_width[c] / 2
    }

    /// Write `text` starting at grid column `cx`'s char offset on `line`.
    pub(super) fn stamp(&mut self, line: usize, cx: usize, text: &str) {
        let x0 = self.offsets[cx];
        for (k, ch) in text.chars().enumerate() {
            self.grid[line][x0 + k] = ch;
        }
    }

    /// Write `text` starting at absolute char offset `x` on `line`,
    /// bounds-guarded (never panics; typed render philosophy).
    pub(super) fn stamp_at(&mut self, line: usize, x: usize, text: &str) {
        if line >= self.grid.len() {
            return;
        }
        for (k, ch) in text.chars().enumerate() {
            if x + k < self.grid[line].len() {
                self.grid[line][x + k] = ch;
            }
        }
    }

    /// Join the grid lines, each with one trailing newline. Trailing
    /// filler spaces are trimmed per line — the oracle does not pad lines
    /// out to the canvas width (the isolated node's box row is just
    /// `+---+`).
    pub(super) fn into_string(self) -> String {
        let mut out = String::new();
        for line in &self.grid {
            let text: String = line.iter().collect();
            out.push_str(text.trim_end_matches(' '));
            out.push('\n');
        }
        out
    }
}
