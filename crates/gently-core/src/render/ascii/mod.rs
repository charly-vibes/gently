//! Purpose: ascii-art renderer byte-compatible with the pinned oracle's
//! classic ascii output for the recorded corpus (gently-0of).
//! Responsibilities: draw the laid-out grid as node boxes (centered labels,
//! selfloop cells widened to fit the loop elbow) and edges as centered
//! ` --> `/` <-- ` gap arrows or polylines (see the `polyline` submodule);
//! return a typed `RenderError` for geometry this renderer does not support
//! — never panic.
//! Rationale: a minimal deterministic subset of upstream Graph::Easy 0.69
//! `lib/Graph/Easy/As_ascii.pm`, shaped by the recorded v0.69 @ ededa3d7
//! companions (the full ge-ascii_render property contracts stay with
//! gently-0cq).
//!
//! # Char-grid mapping (explicit and deterministic)
//!
//! Each layout column maps to a fixed char width, each grid row to a fixed
//! 3-line band (top border, label row, bottom border):
//!
//! - node columns: box width = `name.chars().count() + 4`, widened to at
//!   least 8 for selfloop nodes (the loop elbow spans the box);
//! - gap columns (no node): 5 chars, the width of `" --> "`;
//! - gap rows (no node): 3 blank lines that edge polylines traverse.
//!
//! Cell `(c, r)` starts at char offset `offset(c)` and line `3 * r`. A
//! horizontal edge run draws on the band's middle line (`3 * r + 1`), a
//! vertical run on the column's center char. Lines are not padded out to
//! the canvas width — trailing filler spaces are trimmed.

mod polyline;

use crate::graph::Graph;
use crate::layout::Layout;

/// Width in chars of a gap (edge-routing) grid column.
const GAP_WIDTH: usize = 5;
/// Height in lines of one grid row's char band (border, label, border).
const BAND_HEIGHT: usize = 3;
/// Minimum box width for a selfloop node: the loop elbow spans the box,
/// so the cell widens to fit both corners (oracle: `+------+` over `a`).
const SELFLOOP_MIN_WIDTH: usize = 8;

/// A typed render failure: a human description of the unsupported geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    /// Human-readable description of why the layout cannot be rendered.
    pub message: String,
}

impl RenderError {
    pub(super) fn unsupported(message: String) -> RenderError {
        RenderError { message }
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RenderError {}

/// Render `graph` over its `layout` as ascii art.
///
/// Byte-identical to the pinned oracle (Graph::Easy v0.69 @ ededa3d7,
/// `as_ascii`) for the recorded corpus, with one trailing newline. The
/// empty graph renders as the empty string.
pub fn render(graph: &Graph, layout: &Layout) -> Result<String, RenderError> {
    let mut canvas = Canvas::new(graph, layout);
    canvas.draw_nodes(graph, layout);
    for edge_index in 0..graph.edges.len() {
        canvas.draw_edge(graph, layout, edge_index)?;
    }
    Ok(canvas.into_string())
}

/// The blank char grid plus the column-width mapping it is drawn against.
struct Canvas {
    grid: Vec<Vec<char>>,
    /// Start char offset of each grid column (`offsets[c]`; length width+1).
    offsets: Vec<usize>,
    /// Char width of each grid column.
    col_width: Vec<usize>,
}

/// Box width of a node: label + two border chars per side, widened for
/// selfloop nodes so the loop elbow fits above the box.
fn box_width(name: &str, selfloop: bool) -> usize {
    let w = name.chars().count() + 4;
    if selfloop {
        w.max(SELFLOOP_MIN_WIDTH)
    } else {
        w
    }
}

impl Canvas {
    /// Build the blank grid: column widths per the char-grid mapping, then
    /// prefix offsets, then `height * BAND_HEIGHT` blank lines.
    fn new(graph: &Graph, layout: &Layout) -> Canvas {
        let selfloop_node: Vec<bool> = (0..graph.nodes.len())
            .map(|i| graph.edges.iter().any(|e| e.from == i && e.to == i))
            .collect();
        let mut col_width = vec![GAP_WIDTH; layout.width];
        for (idx, &(cx, _)) in layout.node_cells.iter().enumerate() {
            let w = box_width(&graph.nodes[idx].name, selfloop_node[idx]);
            if w > col_width[cx] {
                col_width[cx] = w;
            }
        }
        let mut offsets = vec![0usize; layout.width + 1];
        for c in 0..layout.width {
            offsets[c + 1] = offsets[c] + col_width[c];
        }
        let total_width = offsets[layout.width];
        let grid = vec![vec![' '; total_width]; layout.height * BAND_HEIGHT];
        Canvas {
            grid,
            offsets,
            col_width,
        }
    }

    /// Stamp every node's box: top border, centered label row, bottom border.
    fn draw_nodes(&mut self, graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            let label = &graph.nodes[idx].name;
            let w = self.col_width[cx];
            let inner = w - 2;
            let len = label.chars().count();
            let left = (inner - len) / 2;
            let mid = format!(
                "|{}{}{}|",
                " ".repeat(left),
                label,
                " ".repeat(inner - len - left)
            );
            let border = format!("+{}+", "-".repeat(inner));
            let top_line = cy * BAND_HEIGHT;
            self.stamp(top_line, cx, &border);
            self.stamp(top_line + 1, cx, &mid);
            self.stamp(top_line + 2, cx, &border);
        }
    }

    /// Draw one edge: selfloop elbow, straight gap arrow, or polyline.
    fn draw_edge(
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
            // Straight through the single gap column: centered arrow.
            let arrow = if tc > sc { " --> " } else { " <-- " };
            self.stamp(sr * BAND_HEIGHT + 1, path[0].0, arrow);
            return Ok(());
        }

        polyline::draw(self, layout, edge_index, sc, sr, tc, tr)
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
    fn mid(&self, r: usize) -> usize {
        r * BAND_HEIGHT + 1
    }

    /// Center char of column `c`.
    fn center(&self, c: usize) -> usize {
        self.offsets[c] + self.col_width[c] / 2
    }

    /// Write `text` starting at grid column `cx`'s char offset on `line`.
    fn stamp(&mut self, line: usize, cx: usize, text: &str) {
        let x0 = self.offsets[cx];
        for (k, ch) in text.chars().enumerate() {
            self.grid[line][x0 + k] = ch;
        }
    }

    /// Join the grid lines, each with one trailing newline. Trailing
    /// filler spaces are trimmed per line — the oracle does not pad lines
    /// out to the canvas width (the isolated node's box row is just
    /// `+---+`).
    fn into_string(self) -> String {
        let mut out = String::new();
        for line in &self.grid {
            let text: String = line.iter().collect();
            out.push_str(text.trim_end_matches(' '));
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{Edge, Graph, Node};
    use crate::layout;
    use crate::render::ascii;

    #[test]
    fn tracer_shape_matches_oracle() {
        let g = Graph::tracer();
        let art = ascii::render(&g, &layout::layout(&g)).expect("must render");
        assert_eq!(art, "+---+     +---+\n| a | --> | b |\n+---+     +---+\n");
    }

    #[test]
    fn longer_labels_widen_boxes_and_keep_the_gap() {
        let g = Graph {
            nodes: vec![Node::named("node1"), Node::named("node2")],
            edges: vec![Edge::directed(0, 1)],
            ..Graph::default()
        };
        let art = ascii::render(&g, &layout::layout(&g)).expect("must render");
        assert_eq!(
            art,
            "+-------+     +-------+\n| node1 | --> | node2 |\n+-------+     +-------+\n"
        );
    }

    #[test]
    fn edgeless_nodes_render_without_arrows() {
        let g = Graph {
            nodes: vec![Node::named("a"), Node::named("b")],
            edges: vec![],
            ..Graph::default()
        };
        let art = ascii::render(&g, &layout::layout(&g)).expect("must render");
        assert_eq!(art, "+---+     +---+\n| a |     | b |\n+---+     +---+\n");
    }

    #[test]
    fn empty_graph_renders_empty() {
        let g = Graph::default();
        assert_eq!(ascii::render(&g, &layout::layout(&g)).unwrap(), "");
    }

    #[test]
    fn westward_edge_draws_left_arrow() {
        let g = Graph {
            nodes: vec![Node::named("a"), Node::named("c"), Node::named("b")],
            edges: vec![Edge::directed(0, 1), Edge::directed(2, 1)],
            ..Graph::default()
        };
        let art = ascii::render(&g, &layout::layout(&g)).expect("must render");
        assert_eq!(
            art,
            "+---+     +---+     +---+\n| a | --> | c | <-- | b |\n+---+     +---+     +---+\n"
        );
    }
}