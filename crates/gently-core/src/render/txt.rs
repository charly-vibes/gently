//! Purpose: canonical txt serialization for the tracer slice (tb.txt-render,
//! gently-2po.7). Responsibilities: emit each edge as
//! `[ <from> ] --> [ <to> ]\n` and each edge-less node as `[ <name> ]\n`.
//! Rationale: byte-identical to the pinned oracle (Graph::Easy v0.69 @
//! ededa3d7, `add_edge("a","b"); as_txt` → `"[ a ] --> [ b ]\n"`). No class
//! sections, styles, or ordering logic — the full contracts (c1–c4, p1–p4)
//! land with ge.txt_render (gently-3hv).

use crate::graph::Graph;

/// Serialize the tracer slice to canonical txt form.
pub fn render(graph: &Graph) -> String {
    let mut out = String::new();
    let mut edge_used = vec![false; graph.nodes.len()];
    for edge in &graph.edges {
        let from = &graph.nodes[edge.from].name;
        let to = &graph.nodes[edge.to].name;
        out.push_str(&format!("[ {from} ] --> [ {to} ]\n"));
        edge_used[edge.from] = true;
        edge_used[edge.to] = true;
    }
    for (i, node) in graph.nodes.iter().enumerate() {
        if !edge_used[i] {
            out.push_str(&format!("[ {} ]\n", node.name));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::graph::{Edge, Graph, Node};

    #[test]
    fn edge_less_node_emits_bare_node_line() {
        let g = Graph {
            nodes: vec![Node::named("a"), Node::named("b")],
            edges: vec![Edge::directed(0, 1), Edge::directed(0, 1)],
            ..Graph::default()
        };
        let mut g2 = g.clone();
        g2.nodes.push(Node::named("c"));
        assert_eq!(super::render(&g2), "[ a ] --> [ b ]\n[ a ] --> [ b ]\n[ c ]\n");
    }

    #[test]
    fn empty_graph_emits_empty_output() {
        assert_eq!(super::render(&Graph::default()), "");
    }
}
