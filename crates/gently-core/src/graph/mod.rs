//! Purpose: the shared in-memory graph model — capability ge.graph_model
//! (gently-4ht), the single model every parser, layout, and renderer consumes.
//! Responsibilities: own nodes, edges, and the group stub; unique named
//! nodes with duplicate-merge insertion (upstream add_node semantics) and
//! anonymous nodes unreferencable by name; per-object AND class-scoped
//! attribute tables storing values verbatim with derived border components
//! computed at assignment time exactly as upstream Graph::Easy; edge
//! integrity (exactly two live endpoints, self-loops legal, node removal
//! drops incident edges); direction and per-end arrow-head presence
//! preserved bit-exactly; the published accessor contract for downstream
//! consumers.
//! Rationale: the tracer-era `Vec`-of-nodes model is deepened in place —
//! slab indices stay stable, all mutations keep every edge endpoint live
//! (upstream `del_node` drops incident edges, so nothing dangles), and the
//! tracer pipeline call sites were adapted without changing public tracer
//! behavior.
//!
//! # Supported attribute set (explicit)
//!
//! - Every key/value pair is stored **verbatim** per scope (strings, no
//!   loss, no type coercion); unknown keys are accepted and kept.
//! - The key `border` additionally derives three components **at assignment
//!   time**, exactly as upstream Graph::Easy 0.69 `split_border_attributes`
//!   (lib/Graph/Easy/Attributes.pm): `border_style`, `border_width`,
//!   `border_color`. Style is the first occurrence of one of `solid`,
//!   `dotted`, `dot-dot-dash`, `dot-dash`, `dashed`, `double-dash`,
//!   `double`, `bold-dash`, `bold`, `broad`, `wide`, `wave`, `none`,
//!   defaulting to `solid`; width is digits-only (`"2px"` → `"2"`, absent →
//!   `""`); color is the remaining token (absent → `""`). The value `0`
//!   special-cases to style `none`, width `""`, color `""`.
//! - Class scopes are `(kind, class-name)`; the empty class name is the
//!   base class. Groups are a documented v1 stub (no membership semantics
//!   yet — see specs/ge-graph_model.md).

mod attributes;

pub use attributes::AttributeTable;

/// Which object kind a class scope applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Node,
    Edge,
    Group,
}

/// The scope an attribute assignment targets: one object, the graph itself,
/// or a class of objects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// Per-object attribute on the node at this index.
    Node(usize),
    /// Per-object attribute on the edge at this index.
    Edge(usize),
    /// Per-object attribute on the group stub at this index.
    Group(usize),
    /// Graph-level attribute.
    Graph,
    /// Class-scoped attribute: every object of this kind in this class
    /// (empty class name = the base class).
    Class(ObjectKind, String),
}

/// A named node; the empty name marks an anonymous node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub attributes: AttributeTable,
}

impl Node {
    /// A node with the given name and an empty attribute table.
    pub fn named(name: &str) -> Node {
        Node {
            name: name.to_string(),
            attributes: AttributeTable::default(),
        }
    }
}

/// Per-end arrow-head presence: an arrowhead drawn at the edge's start
/// (source) and/or end (target).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Arrows {
    pub start: bool,
    pub end: bool,
}

/// An edge between two nodes, by node index, with its direction and
/// per-end arrow-head presence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    /// Directed (`true`) or undirected (`false`).
    pub directed: bool,
    pub arrows: Arrows,
    pub attributes: AttributeTable,
}

impl Edge {
    /// A directed edge: arrowhead at the target end only.
    pub fn directed(from: usize, to: usize) -> Edge {
        Edge {
            from,
            to,
            directed: true,
            arrows: Arrows {
                start: false,
                end: true,
            },
            attributes: AttributeTable::default(),
        }
    }

    /// An undirected edge: no arrowheads.
    pub fn undirected(from: usize, to: usize) -> Edge {
        Edge {
            from,
            to,
            directed: false,
            arrows: Arrows::default(),
            attributes: AttributeTable::default(),
        }
    }

    /// A bidirectional edge: arrowheads at both ends.
    pub fn bidirectional(from: usize, to: usize) -> Edge {
        Edge {
            from,
            to,
            directed: true,
            arrows: Arrows {
                start: true,
                end: true,
            },
            attributes: AttributeTable::default(),
        }
    }
}

/// A group stub — membership semantics are out of scope for v1
/// (see specs/ge-graph_model.md); groups exist as attribute-bearing
/// objects so class-scoped group attributes have a home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub name: String,
    pub attributes: AttributeTable,
}

/// A graph: unique named nodes, live-endpoint edges, a group stub, and
/// per-object plus class-scoped attribute tables.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// Group stubs, in first-seen order.
    pub groups: Vec<Group>,
    /// Graph-level attributes.
    pub attributes: AttributeTable,
    /// Class-scoped attribute tables, keyed by `(kind, class)`, in
    /// first-touched order.
    pub class_attributes: Vec<(ObjectKind, String, AttributeTable)>,
}

impl Graph {
    /// The tracer shape: two nodes `a`, `b`, one directed edge `a -> b`.
    pub fn tracer() -> Graph {
        Graph {
            nodes: vec![Node::named("a"), Node::named("b")],
            edges: vec![Edge::directed(0, 1)],
            groups: Vec::new(),
            attributes: AttributeTable::default(),
            class_attributes: Vec::new(),
        }
    }

    /// Intern a node by name (c1): inserting a duplicate name merges into
    /// the existing node and returns its index — the existing node is
    /// returned, no second node is created (upstream add_node semantics).
    /// The empty name creates a fresh anonymous node every call.
    pub fn add_node(&mut self, name: &str) -> usize {
        if name.is_empty() {
            self.nodes.push(Node {
                name: String::new(),
                attributes: AttributeTable::default(),
            });
        } else if let Some(existing) = self.node_by_name(name) {
            return existing;
        } else {
            self.nodes.push(Node::named(name));
        }
        self.nodes.len() - 1
    }

    /// Create a fresh anonymous node (upstream add_anon_node): unnamed,
    /// distinct from every other node, unreferencable by name after
    /// creation. Returns its index.
    pub fn add_anonymous_node(&mut self) -> usize {
        self.add_node("")
    }

    /// The index of the named node, if it exists. Anonymous nodes are
    /// never found by name.
    pub fn node_by_name(&self, name: &str) -> Option<usize> {
        if name.is_empty() {
            return None;
        }
        self.nodes.iter().position(|n| n.name == name)
    }

    /// Intern a group stub by name (duplicate names merge, as nodes do).
    pub fn add_group(&mut self, name: &str) -> usize {
        match self.groups.iter().position(|grp| grp.name == name) {
            Some(existing) => existing,
            None => {
                self.groups.push(Group {
                    name: name.to_string(),
                    attributes: AttributeTable::default(),
                });
                self.groups.len() - 1
            }
        }
    }

    /// Add an edge between two **live** nodes (c3): both indices must be
    /// in range or the edge is rejected (returns `None`) — no dangling
    /// edge ever escapes a mutation. Self-loops (`from == to`) are legal.
    /// Returns the new edge's index.
    pub fn add_edge(&mut self, from: usize, to: usize, directed: bool) -> Option<usize> {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return None;
        }
        let edge = if directed {
            Edge::directed(from, to)
        } else {
            Edge::undirected(from, to)
        };
        self.edges.push(edge);
        Some(self.edges.len() - 1)
    }

    /// Remove a node (upstream `del_node`): all incident edges are dropped
    /// with it, so no dangling edge escapes the mutation. Remaining edge
    /// endpoints are re-indexed so they stay live. Returns `false` (no-op)
    /// when the index is out of range, upstream-faithful.
    pub fn remove_node(&mut self, index: usize) -> bool {
        if index >= self.nodes.len() {
            return false;
        }
        self.edges.retain(|e| e.from != index && e.to != index);
        self.nodes.remove(index);
        for edge in &mut self.edges {
            if edge.from > index {
                edge.from -= 1;
            }
            if edge.to > index {
                edge.to -= 1;
            }
        }
        true
    }

    /// Store `value` verbatim under `key` in the table `scope` targets
    /// (c2). Assignment to one scope never touches any other scope.
    /// Panics when an object-indexed scope is out of range.
    pub fn set_attr(&mut self, scope: Scope, key: &str, value: &str) {
        match scope {
            Scope::Node(i) => self.nodes[i].attributes.set(key, value),
            Scope::Edge(i) => self.edges[i].attributes.set(key, value),
            Scope::Group(i) => self.groups[i].attributes.set(key, value),
            Scope::Graph => self.attributes.set(key, value),
            Scope::Class(kind, class) => self.class_table(kind, class).set(key, value),
        }
    }

    /// Read the value stored under `key` in the table `scope` targets.
    /// Panics when an object-indexed scope is out of range.
    pub fn get_attr(&self, scope: Scope, key: &str) -> Option<&str> {
        match scope {
            Scope::Node(i) => self.nodes[i].attributes.get(key),
            Scope::Edge(i) => self.edges[i].attributes.get(key),
            Scope::Group(i) => self.groups[i].attributes.get(key),
            Scope::Graph => self.attributes.get(key),
            Scope::Class(kind, class) => self
                .class_attributes
                .iter()
                .find(|(k, c, _)| *k == kind && *c == class)
                .and_then(|(_, _, table)| table.get(key)),
        }
    }

    /// The class-scoped table for `(kind, class)`, created on first touch.
    fn class_table(&mut self, kind: ObjectKind, class: String) -> &mut AttributeTable {
        let existing = self
            .class_attributes
            .iter()
            .position(|(k, c, _)| *k == kind && *c == class);
        let idx = match existing {
            Some(idx) => idx,
            None => {
                self.class_attributes.push((kind, class, AttributeTable::default()));
                self.class_attributes.len() - 1
            }
        };
        let (_, _, table) = &mut self.class_attributes[idx];
        table
    }

    /// Published consumer accessor: all nodes, in model order.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Published consumer accessor: all edges, in model order.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
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
        assert_eq!(g.edges, vec![super::Edge::directed(0, 1)]);
    }
}
