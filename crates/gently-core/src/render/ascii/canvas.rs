//! Purpose: the char-grid canvas the ascii renderer draws on — column
//! widths per the char-grid mapping, per-node border/shape resolution,
//! per-edge style runs, selfloop elbows and polyline plumbing.
//! Responsibilities: build the blank grid from the layout's cell grid
//! (node columns take their boxes' width, gap columns the 5-char floor,
//! labelled straight edges widen their gap), stamp bordered/rounded/
//! point/invisible nodes and edge runs, and join the lines with per-line
//! trailing-space trimming (the oracle does not pad lines).
//! Rationale: split from monolithic mod.rs past the tidy file limit; the
//! glyph tables live in styles.rs, probed from the pinned oracle
//! (tests/repro/claims/ascii-render-tables.observed — gently-0cq).

use super::polyline;
use super::styles::{self, border, display_width, edge_gap_width, h_run, shape, Shape};
use super::RenderError;
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::{Cell, Layout};

/// Width in chars of a gap (edge-routing) grid column.
pub(super) const GAP_WIDTH: usize = 5;
/// Height in lines of one grid row's char band (border, label, border).
pub(super) const BAND_HEIGHT: usize = 3;
/// Minimum box width for a selfloop node: the loop elbow spans the box,
/// so the cell widens to fit both corners (oracle: `+------+` over `a`).
const SELFLOOP_MIN_WIDTH: usize = 8;

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

/// Box width of a node: display width of the label (double-width glyphs
/// count two columns, ge.ascii_render.c5) plus two border chars per side
/// — or one per side for a `none` border — widened for selfloop nodes so
/// the loop elbow fits above the box.
fn box_width(name: &str, style: &str, selfloop: bool) -> usize {
    let pad = if style == "none" { 1 } else { 2 };
    let w = display_width(name) + 2 * pad;
    if selfloop {
        w.max(SELFLOOP_MIN_WIDTH)
    } else {
        w
    }
}

/// The value of `key` on the object at `scope`, falling back to the base
/// class of `kind` (per-object attributes win over class attributes).
fn object_attr<'a>(graph: &'a Graph, scope: Scope, kind: ObjectKind, key: &str) -> Option<&'a str> {
    graph
        .get_attr(scope, key)
        .or_else(|| graph.get_attr(Scope::Class(kind, String::new()), key))
}

/// Which nodes carry a selfloop edge (their boxes widen for the elbow).
fn selfloops(graph: &Graph) -> Vec<bool> {
    (0..graph.nodes.len())
        .map(|i| graph.edges.iter().any(|e| e.from == i && e.to == i))
        .collect()
}

/// Per-node (border style, shape), per-object attributes over class ones.
fn node_styles(graph: &Graph) -> Vec<(String, Shape)> {
    (0..graph.nodes.len())
        .map(|i| {
            let scope = Scope::Node(i);
            let style = object_attr(graph, scope.clone(), ObjectKind::Node, "border_style")
                .unwrap_or("solid")
                .to_string();
            (style, shape(object_attr(graph, scope, ObjectKind::Node, "shape")))
        })
        .collect()
}

/// Per-edge style string, per-object attributes over class ones.
fn edge_styles(graph: &Graph) -> Vec<String> {
    (0..graph.edges.len())
        .map(|i| {
            object_attr(graph, Scope::Edge(i), ObjectKind::Edge, "style")
                .unwrap_or("solid")
                .to_string()
        })
        .collect()
}

/// Column widths per the char-grid mapping and their prefix offsets:
/// node columns take exactly their boxes' width (a `none`-border box is
/// narrower than the gap floor — oracle: ` x`), gap columns keep
/// GAP_WIDTH, and labelled straight edges widen their gap column (the
/// label hosts on the box-top row — oracle: `+---+  go   +---+` over
/// ` ----> `).
fn column_geometry(
    graph: &Graph,
    layout: &Layout,
    node_style: &[(String, Shape)],
    selfloop_node: &[bool],
    edge_style: &[String],
) -> (Vec<usize>, Vec<usize>) {
    let mut node_width = vec![0usize; layout.width];
    let mut node_col = vec![false; layout.width];
    for (idx, &(cx, _)) in layout.node_cells.iter().enumerate() {
        let (style, _) = &node_style[idx];
        let w = box_width(&graph.nodes[idx].name, style, selfloop_node[idx]);
        node_width[cx] = node_width[cx].max(w);
        node_col[cx] = true;
    }
    let mut col_width = vec![GAP_WIDTH; layout.width];
    for c in 0..layout.width {
        if node_col[c] {
            col_width[c] = node_width[c];
        }
    }
    for (ei, path) in layout.edge_paths.iter().enumerate() {
        if path.len() == 1 {
            widen_edge_column(graph, layout, edge_style, &mut col_width, ei, path);
        }
    }
    let mut offsets = vec![0usize; layout.width + 1];
    for c in 0..layout.width {
        offsets[c + 1] = offsets[c] + col_width[c];
    }
    (col_width, offsets)
}

/// Widen the gap column of one straight single-gap edge (gently-7qz): the
/// label's width plus the 5-char label padding, the style's minimum gap
/// width (gently-oy2: upstream `_correct_size` widens the cell for
/// dot-dot-dash — the ` ..-> ` gap; the other styles keep the 5-column
/// floor), and the BD short-cell's +1 (upstream `_correct_size` w++ for a
/// bidirectional EDGE_HOR cell with arrows — the ` <--> ` gap), the last
/// only for the horizontal straight edge through the gap column.
fn widen_edge_column(
    graph: &Graph,
    layout: &Layout,
    edge_style: &[String],
    col_width: &mut [usize],
    ei: usize,
    path: &[Cell],
) {
    let c = path[0].0;
    let e = &graph.edges[ei];
    let bd_hor = e.arrows.start
        && e.arrows.end
        && layout.node_cells[e.from].1 == layout.node_cells[e.to].1;
    let extra = if bd_hor { 1 } else { 0 };
    if let Some(label) = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "label") {
        col_width[c] = col_width[c].max(display_width(label) + 5 + extra);
    }
    col_width[c] = col_width[c].max(edge_gap_width(&edge_style[ei]) + extra);
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
        let (col_width, offsets) =
            column_geometry(graph, layout, &node_style, &selfloop_node, &edge_style);
        let total_width = offsets[layout.width];
        let grid = vec![vec![' '; total_width]; layout.height * BAND_HEIGHT];
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
        }
    }

    /// Stamp every node's box per its resolved shape: bordered box,
    /// rounded box (blank corners), point (`*`), or invisible (nothing).
    pub(super) fn draw_nodes(&mut self, graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            let label = graph.nodes[idx].name.clone();
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
        let top_line = cy * BAND_HEIGHT;
        self.stamp(top_line, cx, &format!("{}{}{}", b.tl, run, b.tr));
        self.stamp(top_line + 1, cx, &mid);
        self.stamp(top_line + 2, cx, &format!("{}{}{}", b.bl, run, b.br));
    }

    /// Stamp a point node: blank borders, a centered `*` (oracle bytes:
    /// blank, `  *`, blank for a width-5 point node).
    fn draw_point(&mut self, cy: usize, cx: usize) {
        let mid = centered_point_row(self.col_width[cx] - 2);
        let top_line = cy * BAND_HEIGHT;
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
        let y_mid = (r - 1) * BAND_HEIGHT + 1;
        let y_attach = r * BAND_HEIGHT - 1;
        let (xl, xr) = (x0 + 2, x0 + w - 3);
        for x in xl + 1..xr {
            self.grid[y_mid][x] = '-';
        }
        self.grid[y_mid][xl] = '+';
        self.grid[y_mid][xr] = '+';
        self.grid[y_attach][xr] = '|';
        self.grid[y_attach][xl] = 'v';
    }

    /// Char line of row `r`'s middle (label) line.
    pub(super) fn mid(&self, r: usize) -> usize {
        r * BAND_HEIGHT + 1
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
