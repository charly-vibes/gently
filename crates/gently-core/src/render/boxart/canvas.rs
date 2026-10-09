//! Purpose: the char-grid canvas the boxart renderer draws on — blank grid
//! from the layout's cell grid, per-node border/shape resolution, per-edge
//! straight-run cells, selfloop elbows and polyline plumbing.
//! Responsibilities: build the grid from the column mapping (`columns`),
//! stamp bordered/rounded/point/invisible nodes and edge runs, and join
//! the lines with per-line trailing-space trimming (the oracle does not
//! pad lines).
//! Rationale: split from mod.rs past the tidy file limit; the glyph tables
//! live in styles.rs, the column mapping in columns.rs — probed from the
//! pinned oracle (tests/repro/claims/boxart-render-tables.observed).

use super::columns::{column_geometry, edge_styles, node_styles, object_attr, Shape};
use super::polyline;
use super::styles::{self, border, h_run, rounded, shifted_run};
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::Layout;
use crate::render::ascii::RenderError;
use crate::render::ascii::display_width;

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
    col_width: Vec<usize>,
    /// Resolved render data per node, in model order.
    node_style: Vec<(String, Shape)>,
    /// Resolved edge style per edge, in model order.
    edge_style: Vec<String>,
    /// Source-side arrowhead presence per edge, in model order.
    edge_arrow_start: Vec<bool>,
    /// Target-side arrowhead presence per edge, in model order.
    edge_arrow_end: Vec<bool>,
}

impl Canvas {
    /// Build the blank grid: column widths per the char-grid mapping,
    /// then prefix offsets, then `height * BAND_HEIGHT` blank lines.
    pub(super) fn new(graph: &Graph, layout: &Layout) -> Canvas {
        let node_style = node_styles(graph);
        let edge_style = edge_styles(graph);
        let (col_width, offsets) = column_geometry(graph, layout, &node_style);
        let total_width = offsets[layout.width];
        let grid = vec![vec![' '; total_width]; layout.height * BAND_HEIGHT];
        let edge_arrow_end: Vec<bool> = graph.edges.iter().map(|e| e.arrows.end).collect();
        let edge_arrow_start: Vec<bool> = graph.edges.iter().map(|e| e.arrows.start).collect();
        Canvas {
            grid,
            offsets,
            col_width,
            node_style,
            edge_style,
            edge_arrow_start,
            edge_arrow_end,
        }
    }

    /// The resolved edge style of edge `ei`.
    pub(super) fn edge_style(&self, ei: usize) -> &str {
        &self.edge_style[ei]
    }

    /// The column a bend's vertical pieces attach at: two chars into the
    /// node's column — upstream draws corner cells at fixed cell-x 2
    /// (probed: also for a 3-wide none-border box, whose label sits at 1).
    pub(super) fn attach_col(&self, c: usize) -> usize {
        self.offsets[c] + 2
    }

    /// Char line of row `r`'s middle (label) line.
    pub(super) fn mid(&self, r: usize) -> usize {
        r * BAND_HEIGHT + 1
    }

    /// Stamp every node's box per its resolved shape: bordered box,
    /// rounded box (per-style corner overlay), point (`★`), or invisible
    /// (nothing).
    pub(super) fn draw_nodes(&mut self, graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            let label = graph.nodes[idx].name.clone();
            let (style, shp) = self.node_style[idx].clone();
            match shp {
                Shape::Invisible => {}
                Shape::Point => self.draw_point(cy, cx),
                Shape::Rounded => self.draw_box(cy, cx, &label, rounded(&style)),
                Shape::Box => self.draw_box(cy, cx, &label, border(&style)),
            }
        }
    }

    /// Stamp one bordered box: top border, centered label row, bottom
    /// border (centering follows display width; the horizontal patterns
    /// cycle from offset 1, the sides come per side).
    fn draw_box(&mut self, cy: usize, cx: usize, label: &str, b: styles::Border) {
        let w = self.col_width[cx];
        let inner = w - 2;
        let len = display_width(label);
        let left = (inner - len) / 2;
        let mid = format!(
            "{}{}{}{}{}",
            b.side_l,
            " ".repeat(left),
            label,
            " ".repeat(inner - len - left),
            b.side_r
        );
        let top_line = cy * BAND_HEIGHT;
        self.stamp(top_line, cx, &format!("{}{}{}", b.tl, h_run(&b, inner), b.tr));
        self.stamp(top_line + 1, cx, &mid);
        self.stamp(
            top_line + 2,
            cx,
            &format!("{}{}{}", b.bl, shifted_run(b.h_bottom, inner), b.br),
        );
    }

    /// Stamp a point node: blank borders, a centered ★.
    fn draw_point(&mut self, cy: usize, cx: usize) {
        let inner = self.col_width[cx] - 2;
        let left = (inner - 1) / 2;
        let mid = format!(
            " {}{}{} ",
            " ".repeat(left),
            styles::POINT,
            " ".repeat(inner - 1 - left)
        );
        let top_line = cy * BAND_HEIGHT;
        self.stamp(top_line + 1, cx, &mid);
    }

    /// Draw one edge: selfloop elbow, straight gap run, or polyline.
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
            self.draw_selfloop((sc, sr), edge_index);
            return Ok(());
        }
        if sr == tr && path.len() == 1 && sc.abs_diff(tc) == 2 {
            let label = object_attr(graph, Scope::Edge(edge_index), ObjectKind::Edge, "label")
                .map(str::to_string);
            self.draw_straight(sr, path[0].0, tc < sc, edge_index, label);
            return Ok(());
        }

        polyline::draw(self, layout, edge_index, sc, sr, tc, tr)
    }

    /// Stamp the straight run of one edge through its gap column and, when
    /// the edge carries a label, the label on the box-top row.
    fn draw_straight(
        &mut self,
        sr: usize,
        c: usize,
        westward: bool,
        edge_index: usize,
        label: Option<String>,
    ) {
        let style = self.edge_style[edge_index].to_string();
        let arrowed = self.edge_arrow_end[edge_index];
        let bidi = self.edge_arrow_start[edge_index] && arrowed;
        let w = self.col_width[c];
        let gap_start = self.offsets[c];
        let top_line = sr * BAND_HEIGHT;
        if let Some(label) = &label {
            let (xs, ws) = if arrowed { (2, 3) } else { (2, 2) };
            let left = (w - xs - ws).saturating_sub(display_width(label)) / 2;
            self.stamp(top_line, c, &format!("{}{}", " ".repeat(xs + left), label));
        }
        let (x, text) = straight_run(&style, w, gap_start, westward, arrowed, bidi);
        self.stamp(top_line + 1, c, &format!("{}{}", " ".repeat(x), text));
    }

    /// Draw a selfloop: a small elbow over the box, departing through the
    /// east corner and arriving through the west one with a `∨` arrowhead
    /// (the edge style's corner/hor/ver glyphs; arrowless loops attach the
    /// ver glyph on both sides).
    fn draw_selfloop(&mut self, (c, r): (usize, usize), edge_index: usize) {
        let e = styles::edge(self.edge_style(edge_index));
        let arrowed = self.edge_arrow_end[edge_index];
        let x0 = self.offsets[c];
        let w = self.col_width[c];
        let y_mid = (r - 1) * BAND_HEIGHT + 1;
        let y_attach = r * BAND_HEIGHT - 1;
        let (xl, xr) = (x0 + 2, x0 + w - 3);
        let pat: Vec<char> = e.hor.chars().collect();
        let len = pat.len();
        for x in xl + 1..xr {
            self.grid[y_mid][x] = pat[x % len];
        }
        self.grid[y_mid][xl] = e.corners[0];
        self.grid[y_mid][xr] = e.corners[1];
        self.grid[y_attach][xr] = e.ver;
        self.grid[y_attach][xl] =
            if arrowed { styles::arrow(polyline::Dir::South) } else { e.ver };
    }

    /// Write `text` starting at grid column `cx`'s char offset on `line`.
    fn stamp(&mut self, line: usize, cx: usize, text: &str) {
        let x0 = self.offsets[cx];
        for (k, ch) in text.chars().enumerate() {
            self.grid[line][x0 + k] = ch;
        }
    }

    /// Join the grid lines, each with one trailing newline. Trailing
    /// filler spaces are trimmed per line — the oracle does not pad lines.
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

/// The straight-run line of one edge through a gap column of width `w`
/// starting at char `gap_start`, following the upstream `_draw_hor` cell
/// algorithm: the style's horizontal pattern cycled from phase 0 for the
/// SHORT arrowed cells (which skip the rx offset) and from
/// `gap_start mod len` otherwise, truncated to the column width, then the
/// flag chops — two for the eastward arrowed/arrowless cells (the last
/// remaining char becoming the `>` head), a two-char ` <` head for
/// westward arrowed and bidirectional cells. Returns the stamp offset
/// (cell-x 1 for eastward/arrowless, 0 for westward/bidirectional) and
/// the run.
fn straight_run(
    style: &str,
    w: usize,
    gap_start: usize,
    westward: bool,
    arrowed: bool,
    bidi: bool,
) -> (usize, String) {
    let pat: Vec<char> = styles::edge(style).hor.chars().collect();
    let len = pat.len();
    let phase = if arrowed && !bidi { 0 } else { gap_start % len };
    let mut line: Vec<char> = (0..w).map(|i| pat[(phase + i) % len]).collect();
    let mut x = 1;
    if westward && arrowed && !bidi {
        // SHORT_W: one chop, then the two-char ' <' head at x 0.
        line.truncate(w - 1);
        line[0] = ' ';
        line[1] = '<';
        x = 0;
    } else if bidi {
        // BD_EW: chop, '>' head, then the two-char ' <' head at x 0.
        line.truncate(w - 1);
        *line.last_mut().expect("non-empty") = '>';
        line[0] = ' ';
        line[1] = '<';
        x = 0;
    } else if arrowed {
        // SHORT_E: two chops, the last remaining char becomes '>'.
        line.truncate(w - 2);
        *line.last_mut().expect("non-empty") = '>';
    } else {
        // UN_EW: two chops, x 1.
        line.truncate(w - 2);
    }
    (x, line.into_iter().collect())
}