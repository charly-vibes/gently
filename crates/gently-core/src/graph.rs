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

/// An ordered attribute table: verbatim `(key, value)` pairs in insertion
/// order, last-set wins on duplicates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttributeTable {
    entries: Vec<(String, String)>,
}

impl AttributeTable {
    /// Store `value` verbatim under `key`; overwrites an earlier value.
    /// The key `border` additionally derives `border_style`, `border_width`
    /// and `border_color` at assignment time (upstream
    /// `split_border_attributes` semantics — see the module header).
    pub fn set(&mut self, key: &str, value: &str) {
        set_entry(&mut self.entries, key, value);
        if key == "border" {
            let (style, width, color) = split_border_attributes(value);
            set_entry(&mut self.entries, "border_style", &style);
            set_entry(&mut self.entries, "border_width", &width);
            set_entry(&mut self.entries, "border_color", &color);
        }
    }

    /// The verbatim value stored under `key`, if any.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// True when no attribute is stored.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Overwrite or append `(key, value)` preserving first-seen position.
fn set_entry(entries: &mut Vec<(String, String)>, key: &str, value: &str) {
    match entries.iter_mut().find(|(k, _)| k == key) {
        Some(slot) => slot.1 = value.to_string(),
        None => entries.push((key.to_string(), value.to_string())),
    }
}

/// Split a border value into `(style, width, color)` exactly as upstream
/// Graph::Easy 0.69 `split_border_attributes` does: the style is the first
/// (leftmost-position, alternation-order) occurrence of a style word, and
/// defaults to `solid`; every `\d+(px|em|%)` token is removed with the last
/// one remembered digits-only as the width; the whitespace-free remainder
/// is the color.
fn split_border_attributes(border: &str) -> (String, String, String) {
    // upstream special case: `border: 0` → none
    if border == "0" {
        return ("none".into(), String::new(), String::new());
    }
    let (mut rest, matched) = strip_first_style_word(border);
    let style = matched.unwrap_or("solid");
    // width: remove every `\d+(px|em|%)` token, remember the last, digits only
    let ranges = width_ranges(&rest);
    let mut width = String::new();
    if let Some(&(start, end)) = ranges.last() {
        width = rest[start..end].chars().filter(|c| c.is_ascii_digit()).collect();
    }
    for &(start, end) in ranges.iter().rev() {
        rest.replace_range(start..end, "");
    }
    let color: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    (style.to_string(), width, color)
}

/// The upstream style words in alternation order — leftmost position wins,
/// then this order decides.
const STYLES: [&str; 13] = [
    "solid", "dotted", "dot-dot-dash", "dot-dash", "dashed", "double-dash", "double",
    "bold-dash", "bold", "broad", "wide", "wave", "none",
];

/// Remove the first style word from `s` (leftmost position, upstream
/// alternation order — no word boundaries, exactly as upstream's regex
/// substitution does), returning the remainder and the matched style.
fn strip_first_style_word(s: &str) -> (String, Option<&'static str>) {
    for pos in 0..s.len() {
        for candidate in STYLES {
            if s[pos..].starts_with(candidate) {
                let mut rest = s.to_string();
                rest.replace_range(pos..pos + candidate.len(), "");
                return (rest, Some(candidate));
            }
        }
    }
    (s.to_string(), None)
}

/// All `\d+(px|em|%)` token ranges in `s`, left to right (upstream's global
/// width substitution: the unit is required, bare digits are left alone).
fn width_ranges(s: &str) -> Vec<(usize, usize)> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let end = if b[i].is_ascii_digit() { digit_token_end(s, i) } else { i };
        if end > i {
            out.push((i, end));
        }
        i = end.max(i + 1);
    }
    out
}

/// End of the `\d+(px|em|%)` token starting at `i`, or `i` when the digits
/// are not followed by a unit (upstream requires the unit).
fn digit_token_end(s: &str, i: usize) -> usize {
    let digits = s[i..].bytes().take_while(u8::is_ascii_digit).count();
    let after = i + digits;
    for unit in ["px", "em", "%"] {
        if s[after..].starts_with(unit) {
            return after + unit.len();
        }
    }
    i
}

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
        todo!("gently-4ht RED: node identity")
    }

    /// Create a fresh anonymous node (upstream add_anon_node): unnamed,
    /// distinct from every other node, unreferencable by name after
    /// creation. Returns its index.
    pub fn add_anonymous_node(&mut self) -> usize {
        todo!("gently-4ht RED: node identity")
    }

    /// The index of the named node, if it exists. Anonymous nodes are
    /// never found by name.
    pub fn node_by_name(&self, name: &str) -> Option<usize> {
        todo!("gently-4ht RED: node identity")
    }

    /// Intern a group stub by name (duplicate names merge, as nodes do).
    pub fn add_group(&mut self, name: &str) -> usize {
        todo!("gently-4ht RED: group stub")
    }

    /// Add an edge between two **live** nodes (c3): both indices must be
    /// in range or the edge is rejected (returns `None`) — no dangling
    /// edge ever escapes a mutation. Self-loops (`from == to`) are legal.
    /// Returns the new edge's index.
    pub fn add_edge(&mut self, from: usize, to: usize, directed: bool) -> Option<usize> {
        todo!("gently-4ht RED: edge integrity")
    }

    /// Remove a node (upstream `del_node`): all incident edges are dropped
    /// with it, so no dangling edge escapes the mutation. Returns `false`
    /// (no-op) when the index is out of range, upstream-faithful.
    pub fn remove_node(&mut self, index: usize) -> bool {
        todo!("gently-4ht RED: edge integrity")
    }

    /// Store `value` verbatim under `key` in the table `scope` targets
    /// (c2). Assignment to one scope never touches any other scope.
    pub fn set_attr(&mut self, scope: Scope, key: &str, value: &str) {
        todo!("gently-4ht RED: attributes")
    }

    /// Read the value stored under `key` in the table `scope` targets.
    pub fn get_attr(&self, scope: Scope, key: &str) -> Option<&str> {
        todo!("gently-4ht RED: attributes")
    }

    /// Published consumer accessor: all nodes, in model order.
    pub fn nodes(&self) -> &[Node] {
        todo!("gently-4ht RED: published contract")
    }

    /// Published consumer accessor: all edges, in model order.
    pub fn edges(&self) -> &[Edge] {
        todo!("gently-4ht RED: published contract")
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
