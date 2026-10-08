//! Purpose: minimal graph model for the tracer-bullet slice (epic gently-2po).
//! Responsibilities: hold nodes by name and directed edges as node indices;
//! expose `Graph::tracer()` constructing the canonical walking-skeleton shape.
//! Rationale: only the shape the tb.txt-render slice (gently-2po.7) serializes
//! exists — the full model contracts land with ge.graph-model (gently-3hv).

/// A named node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
}

/// A directed edge, as indices into `Graph::nodes`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
}

/// A graph: named nodes and directed edges between them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    /// The tracer shape: two nodes `a`, `b`, one directed edge `a -> b`.
    pub fn tracer() -> Graph {
        Graph {
            nodes: vec![Node { name: "a".into() }, Node { name: "b".into() }],
            edges: vec![Edge { from: 0, to: 1 }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Graph;

    #[test]
    fn tracer_shape() {
        let g = Graph::tracer();
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.nodes[0].name, "a");
        assert_eq!(g.nodes[1].name, "b");
        assert_eq!(g.edges, vec![super::Edge { from: 0, to: 1 }]);
    }
}
