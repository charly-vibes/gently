//! ge.graphviz_render (gently-b4v): the Graphviz DOT output renderer —
//! each property row of specs/ge-graphviz_render.md is one test below,
//! named for its id. The probed oracle bytes (tests/repro/claims/
//! graphviz-round-trip.observed, Graph::Easy v0.69 @ ededa3d7) pin the
//! emitted DOT shape: per-node/per-edge attribute lists after quoted
//! names, `subgraph cluster… { label=… }` groups. Where the oracle is
//! lossy (always-`digraph`, always-`->`, group-edge crashes) the spec
//! row wins: arrows mirror the MODEL edge direction (c2).

use gently_core::graph::{Graph, Scope};
use gently_core::parse::dot;
use gently_core::render::graphviz;

/// Count of `needle` occurrences in `haystack` (tests only).
fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// ge.graphviz_render.p1 (c1): every node emits exactly one DOT node
/// statement with a safely quoted name — spaces, unicode, embedded
/// quotes, and `#N` anonymous ids — and node attributes map onto their
/// DOT counterparts (`fill` -> `fillcolor`).
#[test]
fn p1() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let spaced = g.add_node("a b");
    let uni = g.add_node("üñî");
    let quoted = g.add_node("q\"x");
    let anon = g.add_anonymous_node();
    g.set_attr(Scope::Node(a), "color", "#000000");
    g.set_attr(Scope::Node(a), "fill", "white");
    g.set_attr(Scope::Node(a), "shape", "box");
    let out = graphviz::render(&g);

    // exactly one node statement per node, with the safely quoted name
    assert_eq!(count(&out, "\"a\" ["), 1, "bare name a");
    assert_eq!(count(&out, "\"a b\" ["), 1, "spaced name quoted");
    assert_eq!(count(&out, "\"üñî\" ["), 1, "unicode name quoted");
    assert_eq!(count(&out, "\"q\\\"x\" ["), 1, "embedded quote escaped");
    assert_eq!(count(&out, "\"#4\" ["), 1, "anonymous node as #<index>");

    // node attributes map onto DOT counterparts
    assert!(out.contains("\"a\" [ color=\"#000000\", shape=box, fillcolor=\"white\" ]"),
        "attrs on a: {out:?}");
}

/// ge.graphviz_render.p2 (c2): one edge statement per model edge, the
/// arrow matching the MODEL edge direction (`->` directed, `--`
/// undirected — the spec row wins over the oracle's always-`->`), with
/// edge style attributes mapped onto DOT attribute values.
#[test]
fn p2() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    g.add_edge(a, b, true);
    g.add_edge(b, c, false);
    g.set_attr(Scope::Edge(0), "style", "dotted");
    g.set_attr(Scope::Edge(0), "color", "#000000");
    let out = graphviz::render(&g);

    assert_eq!(count(&out, "\"a\" -> \"b\""), 1, "directed edge uses ->");
    assert_eq!(count(&out, "\"b\" -- \"c\""), 1, "undirected edge uses --");
    assert!(out.contains("\"a\" -> \"b\" [ style=dotted, color=\"#000000\" ]"),
        "edge attrs: {out:?}");
}

/// ge.graphviz_render.p3 (c3): every group emits a subgraph cluster
/// containing exactly its member nodes, with the group label as cluster
/// label; named groups keep their name, anonymous groups (empty name)
/// emit `cluster<N>` in internal-id order.
#[test]
fn p3() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let grp = g.add_group("grp");
    g.set_node_group(a, grp);
    g.set_node_group(b, grp);
    g.set_attr(Scope::Group(grp), "label", "my grp");
    let anon = g.add_group("");
    g.set_node_group(c, anon);
    let out = graphviz::render(&g);

    // named group keeps its name; cluster label is the group label
    let cluster = out.split("subgraph \"cluster_grp\" {").nth(1).expect("named cluster emitted");
    let cluster = cluster.split('}').next().unwrap();
    assert!(cluster.contains("\"a\""), "member a inside: {cluster:?}");
    assert!(cluster.contains("\"b\""), "member b inside: {cluster:?}");
    assert!(!cluster.contains("\"c\""), "non-member c outside: {cluster:?}");
    assert!(out.contains("label=\"my grp\""), "group label as cluster label: {out:?}");

    // anonymous group emits cluster<N> in internal-id order (index 1)
    let anon_cluster = out.split("cluster1").nth(1).expect("anonymous cluster1 emitted");
    assert!(anon_cluster.contains("\"c\""), "anon member c inside: {out:?}");
    // grouped nodes appear exactly once (inside their cluster, not top-level)
    assert_eq!(count(&out, "\"a\" ["), 1, "a emitted once");
    assert_eq!(count(&out, "\"c\" ["), 1, "c emitted once");
}

/// The source model for p4: mixed-direction edges, attrs, one group.
fn p4_source() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let d = g.add_node("d e");
    g.add_edge(a, b, true);
    g.add_edge(b, c, false);
    g.add_edge(c, d, true);
    g.set_attr(Scope::Edge(0), "style", "dotted");
    g.set_attr(Scope::Node(a), "color", "red");
    let grp = g.add_group("grp");
    g.set_node_group(b, grp);
    g.set_node_group(c, grp);
    g
}

/// Sorted node-name multiset of `g` (isomorphism: same node names).
fn node_names(g: &Graph) -> Vec<&str> {
    let mut names: Vec<&str> = g.nodes.iter().map(|n| n.name.as_str()).collect();
    names.sort();
    names
}

/// (from-name, to-name, directed) per edge in emission order.
fn edge_triples(g: &Graph) -> Vec<(&str, &str, bool)> {
    g.edges
        .iter()
        .map(|e| (g.nodes[e.from].name.as_str(), g.nodes[e.to].name.as_str(), e.directed))
        .collect()
}

/// Member names of the group whose name contains `marker`.
fn member_names(g: &Graph, marker: &str) -> Vec<String> {
    let grp = g
        .groups
        .iter()
        .find(|x| x.name.contains(marker))
        .unwrap_or_else(|| panic!("group containing {marker:?} must exist"));
    grp.members.iter().map(|&i| g.nodes[i].name.clone()).collect()
}

/// ge.graphviz_render.p4 (c4): feeding the emitted DOT through
/// ge.dot_parser yields a model isomorphic to the source model — same
/// node names, same edges (endpoints, direction, attributes), same group
/// membership.
#[test]
fn p4() {
    let g = p4_source();
    let out = graphviz::render(&g);
    let reparsed = dot::parse(&out).expect("emitted DOT must re-parse");

    assert_eq!(node_names(&g), node_names(&reparsed), "same node names");
    assert_eq!(edge_triples(&g), edge_triples(&reparsed), "same edges incl. direction");
    assert_eq!(reparsed.edges[0].attributes.get("style"), Some("dotted"), "edge attr preserved");
    assert_eq!(member_names(&g, "grp"), member_names(&reparsed, "grp"), "same group membership");
}