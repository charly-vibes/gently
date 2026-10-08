//! Purpose: ascii-art renderer for the tracer slice (tb.cli, gently-2po.9).
//! Responsibilities: draw the laid-out grid as node boxes (`+---+` /
//! `| a |` / `+---+`) with edge arrows (` --> `) through the routed gap
//! cells, byte-identical to the pinned oracle for the tracer shape; return a
//! typed `RenderError` for geometry this slice does not support (vertical or
//! westward routing) — never panic.
//! Rationale: thin slice of ge-ascii_render; the full capability (variable
//! gaps, multi-row bands, box-drawing styles) lands with later slices.
//!
//! # Char-grid mapping (explicit and deterministic)
//!
//! Each layout cell maps to a fixed char block, 3 lines tall (top border,
//! label row, bottom border) and `width(c)` chars wide per grid column `c`:
//!
//! - node columns: box width = `name.chars().count() + 4`
//!   (`|` + `" name "` + `|`, bordered by `+` + `-`*inner + `+`);
//! - gap columns (no node): 5 chars, the width of `" --> "`.
//!
//! Cell `(c, r)` starts at char offset `offset(c) = Σ widths(<c)` and line
//! `3 * r`. Edge-path cells draw `" --> "` on their row's middle line
//! (`3 * r + 1`) — the tracer layout routes every edge eastward through gap
//! columns only, so the arrow fills the gap exactly.

use crate::graph::Graph;
use crate::layout::Layout;

/// Width in chars of a gap (edge-routing) grid column.
const GAP_WIDTH: usize = 5;
/// Height in lines of one grid row's char band (border, label, border).
const BAND_HEIGHT: usize = 3;

/// A typed render failure: a human description of the unsupported geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    /// Human-readable description of why the layout cannot be rendered.
    pub message: String,
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
/// `add_edge("a","b"); as_ascii`) for the tracer shape, with one trailing
/// newline. The empty graph renders as the empty string.
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
}

impl Canvas {
    /// Build the blank grid: column widths per the char-grid mapping, then
    /// prefix offsets, then `height * BAND_HEIGHT` blank lines.
    fn new(graph: &Graph, layout: &Layout) -> Canvas {
        let mut col_width = vec![GAP_WIDTH; layout.width];
        for (idx, &(cx, _)) in layout.node_cells.iter().enumerate() {
            col_width[cx] = graph.nodes[idx].name.chars().count() + 4;
        }
        let mut offsets = vec![0usize; layout.width + 1];
        for c in 0..layout.width {
            offsets[c + 1] = offsets[c] + col_width[c];
        }
        let total_width = offsets[layout.width];
        let grid = vec![vec![' '; total_width]; layout.height * BAND_HEIGHT];
        Canvas { grid, offsets }
    }

    /// Stamp every node's box: top border, label row, bottom border.
    fn draw_nodes(&mut self, graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            let label = &graph.nodes[idx].name;
            let inner = "-".repeat(self.width_of(cx) - 2);
            let top = format!("+{inner}+");
            let mid = format!("| {label} |");
            let rows: [String; 3] = [top.clone(), mid, top];
            for (row, text) in rows.iter().enumerate() {
                self.stamp(cy * BAND_HEIGHT + row, cx, text);
            }
        }
    }

    /// Draw one edge's arrows through its routed path cells: eastward
    /// same-row edges only, one `" --> "` per gap cell on the source row
    /// band's middle line.
    fn draw_edge(
        &mut self,
        graph: &Graph,
        layout: &Layout,
        edge_index: usize,
    ) -> Result<(), RenderError> {
        let edge = &graph.edges[edge_index];
        let (ax, ay) = layout.node_cells[edge.from];
        let (bx, by) = layout.node_cells[edge.to];
        if by != ay || bx <= ax {
            return Err(RenderError {
                message: format!(
                    "unsupported edge routing: only eastward same-row edges render in this slice (edge {edge_index}: cell {ax},{ay} -> cell {bx},{by})"
                ),
            });
        }
        for &(px, py) in &layout.edge_paths[edge_index] {
            if px == 0 || px + 1 >= layout.width || py != ay {
                return Err(RenderError {
                    message: format!(
                        "unsupported edge routing: path cell {px},{py} is not an interior gap cell on the source row"
                    ),
                });
            }
            self.stamp(ay * BAND_HEIGHT + 1, px, " --> ");
        }
        Ok(())
    }

    /// Write `text` starting at grid column `cx`'s char offset on `line`.
    fn stamp(&mut self, line: usize, cx: usize, text: &str) {
        let x0 = self.offsets[cx];
        for (k, ch) in text.chars().enumerate() {
            self.grid[line][x0 + k] = ch;
        }
    }

    fn width_of(&self, cx: usize) -> usize {
        self.offsets[cx + 1] - self.offsets[cx]
    }

    /// Join the grid lines, each with one trailing newline.
    fn into_string(self) -> String {
        let mut out = String::new();
        for line in &self.grid {
            out.extend(line.iter());
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
}