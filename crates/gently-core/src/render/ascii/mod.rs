//! Purpose: ascii-art renderer byte-compatible with the pinned oracle's
//! classic ascii output for the recorded corpus (gently-0of), deepened to
//! the full ge.ascii_render capability (gently-0cq): border styles, edge
//! style runs, shapes, labelled edges, and display-width centering.
//! Responsibilities: resolve the graph against its layout into the char
//! grid (glyph tables in `styles`, canvas in `canvas`, polylines in
//! `polyline`); return a typed `RenderError` for unsupported geometry —
//! never panic.
//! Rationale: a port of upstream Graph::Easy 0.69 `lib/Graph/Easy/
//! As_ascii.pm`, shaped by the recorded v0.69 @ ededa3d7 companions and
//! the probed tables (tests/repro/claims/ascii-render-tables.observed).

mod canvas;
mod polyline;
mod styles;

use canvas::Canvas;
use crate::graph::Graph;
use crate::layout::Layout;


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