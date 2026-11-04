//! ge.dot_parser (gently-89p): the Graphviz DOT grammar — each property row
//! of specs/ge-dot_parser.md is one test below, named for its id. The pinned
//! upstream oracle (Graph::Easy 0.69 Parser::Graphviz @ ededa3d7) is
//! authoritative; the probe evidence in tests/repro/claims/*.observed pins
//! the binding observed behavior (the EDGE OPERATOR decides direction, not
//! the header; any named subgraph becomes a group verbatim; records and
//! HTML-like labels autosplit into `name.N` part nodes).

use gently_core::graph::{Graph, ObjectKind, Scope};
use gently_core::parse::dot;

#[path = "ge_dot_parser/misc.rs"]
mod misc;
use misc::*;
#[path = "ge_dot_parser/p5.rs"]
mod p5;

/// The index of the node with `name` (panics when missing — tests only).
fn node(g: &Graph, name: &str) -> usize {
    g.node_by_name(name)
        .unwrap_or_else(|| panic!("node {name:?} must exist"))
}

/// ge.dot_parser.p1 (c1): a `graph` header sets the model graph's `type`
/// attribute to `undirected`; a `digraph` header leaves `type` unset.
/// Neither header fixes edge direction (p2 covers the operator rule).
#[test]
fn p1() {
    let g = dot::parse("graph G { a -- b }\n").must("must parse");
    assert_eq!(g.attributes.get("type"), Some("undirected"), "graph header");
    let g = dot::parse("digraph G { a -> b }\n").must("must parse");
    assert_eq!(g.attributes.get("type"), None, "digraph header");
    // the header name is the graph title; an unnamed header names the
    // graph `unnamed` — the oracle's own name for it (gh-4 observed)
    let g = dot::parse("digraph G { a -> b }\n").must("must parse");
    assert_eq!(
        g.attributes.get("title"),
        Some("G"),
        "header name is the title"
    );
    let g = dot::parse("digraph { a -> b }\n").must("must parse");
    assert_eq!(g.attributes.get("title"), Some("unnamed"), "unnamed header");
}

/// ge.dot_parser.p6 (c6, gently-cij): the upstream Graphviz parser's
/// defaults (Graphviz::reset) land on the model: the graph carries
/// `flow: south` and `colorscheme: x11`, the edge base class carries
/// `arrowstyle: filled`, and the group base class carries `align: center`
/// / `fill: inherit` once a group exists — the upstream class table only
/// surfaces the group defaults when a group is created (observed in the
/// oracle's post-parse `{att}` dump,
/// tests/repro/claims/cli-input-from.observed). An explicit statement
/// overrides the default (`rankdir` remaps to `flow` per c3).
#[test]
fn p6() {
    let g = dot::parse("digraph G { a -> b }\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Graph, "flow"),
        Some("south"),
        "the flow default drives the vertical layout"
    );
    assert_eq!(
        g.get_attr(Scope::Graph, "colorscheme"),
        Some("x11"),
        "colorscheme default"
    );
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Edge, String::new()), "arrowstyle"),
        Some("filled"),
        "edge base-class arrowstyle default"
    );
    // A groupless graph carries no group class defaults.
    let g = dot::parse("digraph G { a -> b }\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Group, String::new()), "align"),
        None,
        "group class defaults stay unset without a group"
    );

    // A named subgraph (cluster convention or not) creates the group and
    // with it the group base-class defaults.
    let g = dot::parse("digraph G { subgraph cluster_x { a } a -> b }\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Group, String::new()), "align"),
        Some("center"),
        "group class align default surfaces with a group"
    );
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Group, String::new()), "fill"),
        Some("inherit"),
        "group class fill default surfaces with a group"
    );

    // An explicit statement overrides the default (rankdir → flow per c3).
    let g = dot::parse("digraph G { rankdir=LR; a -> b }\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Graph, "flow"),
        Some("east"),
        "explicit rankdir overrides the flow default"
    );
    let g = dot::parse("digraph G { colorscheme=none; a -> b }\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Graph, "colorscheme"),
        Some("none"),
        "explicit colorscheme overrides the default"
    );
}

/// ge.dot_parser.p2 (c2): one model edge per arrow; direction follows the
/// operator (`->` directed, `--` undirected) independently of the header —
/// all four header×operator combinations — and chains share nodes.
#[test]
fn p2() {
    for header in ["digraph", "graph"] {
        for (op, directed) in [("->", true), ("--", false)] {
            let src = format!("{header} G {{ a {op} b }}\n");
            let g = dot::parse(&src).unwrap_or_else(|e| panic!("{src:?} must parse: {e}"));
            assert_eq!(g.edges.len(), 1, "{src:?}: one edge per arrow");
            assert_eq!(g.edges[0].directed, directed, "{src:?}: operator decides");
            assert_eq!(g.edges[0].from, node(&g, "a"), "{src:?}");
            assert_eq!(g.edges[0].to, node(&g, "b"), "{src:?}");
        }
    }
    // a chain creates one edge per adjacent pair, sharing node objects
    let g = dot::parse("digraph G { a -> b -> c }\n").must("must parse");
    assert_eq!(g.edges.len(), 2, "one edge per arrow in a chain");
    assert_eq!(g.edges[0].to, g.edges[1].from, "b is shared");
    // mixed operators in one chain stay independent
    let g = dot::parse("digraph G { a -> b -- c }\n").must("must parse");
    assert!(g.edges[0].directed, "-> stays directed");
    assert!(!g.edges[1].directed, "-- stays undirected");
}

/// ge.dot_parser.p3 (c3): attribute lists `[k=v, k2=v2]` map onto the model
/// attributes of the right object (node or edge), with quoted values
/// unescaped exactly once.
#[test]
fn p3() {
    // node attributes, quoted value with an escaped quote and backslash
    let g = dot::parse("digraph G { a [color=red, label=\"x \\\"y\\\\\" ] }\n").must("must parse");
    let a = node(&g, "a");
    assert_eq!(g.nodes[a].attributes.get("color"), Some("red"));
    assert_eq!(
        g.nodes[a].attributes.get("label"),
        Some("x \"y\\"),
        "unescaped exactly once"
    );
    // edge attributes land on the edge
    let g = dot::parse("digraph G { a -> b [style=dotted] }\n").must("must parse");
    assert_eq!(g.edges[0].attributes.get("style"), Some("dotted"));
    // bare values and multiple pairs on edges
    let g = dot::parse("digraph G { a -> b [dir=both, weight=2] }\n").must("must parse");
    assert_eq!(g.edges[0].attributes.get("dir"), Some("both"));
    assert_eq!(g.edges[0].attributes.get("weight"), Some("2"));
    graph_scope_attr_statements();
}

/// ge.dot_parser.p4 (c4): a named subgraph becomes a model group with the
/// name verbatim containing only its directly declared nodes (nested nodes
/// belong to the innermost group) — except for the graphviz renderer's
/// cluster convention (gently-j2i): an id `cluster_<name>` re-parses as
/// group `<name>` and an id `cluster` + bare digits re-parses as an
/// anonymous group; a bare `{ }` scope keeps its nodes ungrouped — with
/// the oracle's pinned left-edge-stack caveat — and the nameless `subgraph`
/// keyword form (with or without an attribute list) is a tokenizing error.
#[test]
fn p4() {
    // named subgraphs become groups under their verbatim names — with the
    // graphviz cluster convention: `cluster_x` re-parses as `x`, a bare
    // `cluster0` re-parses as an anonymous group (empty name)
    let g = dot::parse(
        "digraph G { a -> b; subgraph cluster_x { c d } subgraph named { e } subgraph cluster0 { f } }\n",
    )
    .must("must parse");
    let cx = g
        .groups
        .iter()
        .position(|grp| grp.name == "x")
        .must("x group (cluster_ stripped)");
    let nm = g
        .groups
        .iter()
        .position(|grp| grp.name == "named")
        .must("named group");
    let an = g
        .groups
        .iter()
        .position(|grp| grp.name.is_empty())
        .must("anonymous group (cluster0)");
    assert_eq!(g.groups[cx].members, vec![node(&g, "c"), node(&g, "d")]);
    assert_eq!(g.groups[nm].members, vec![node(&g, "e")]);
    assert_eq!(g.groups[an].members, vec![node(&g, "f")]);

    // nested subgraphs: each level its own group, innermost membership wins
    let g =
        dot::parse("digraph G { subgraph outer { subgraph inner { a } b } }\n").must("must parse");
    let inner = g
        .groups
        .iter()
        .position(|grp| grp.name == "inner")
        .must("inner group");
    let outer = g
        .groups
        .iter()
        .position(|grp| grp.name == "outer")
        .must("outer group");
    assert_eq!(
        g.groups[inner].members,
        vec![node(&g, "a")],
        "innermost membership"
    );
    assert_eq!(
        g.groups[outer].members,
        vec![node(&g, "b")],
        "directly declared only"
    );

    // bare { } scope: nodes ungrouped; the node stack survives the scope
    // boundary, so `a -> b; { c -> d }` also yields the pinned leaked
    // `b -> d` edge — linked when the scope's inner operator fires, before
    // the scope's own edge (observed, Graph::Easy v0.69 @ ededa3d7)
    let g = dot::parse("digraph G { a -> b; { c -> d } }\n").must("must parse");
    assert!(g.groups.is_empty(), "bare scope keeps nodes ungrouped");
    assert_eq!(
        g.edges.len(),
        3,
        "a->b, plus the leaked b->d and the scope's c->d"
    );
    assert_eq!(
        (g.edges[0].from, g.edges[0].to),
        (node(&g, "a"), node(&g, "b"))
    );
    assert_eq!(
        (g.edges[1].from, g.edges[1].to),
        (node(&g, "b"), node(&g, "d")),
        "leaked stack edge"
    );
    assert_eq!(
        (g.edges[2].from, g.edges[2].to),
        (node(&g, "c"), node(&g, "d"))
    );
    // a bare scope as an edge target links the chain through it
    let g = dot::parse("digraph G { a -> { b } }\n").must("must parse");
    assert!(g.groups.is_empty());
    assert_eq!(
        (g.edges[0].from, g.edges[0].to),
        (node(&g, "a"), node(&g, "b"))
    );

    // bare scope group membership follows the oracle's group_stack
    // (gently-fb1, observed Graph::Easy v0.69 @ ededa3d7,
    // tests/repro/claims/dot-bare-scope-in-named.observed): a bare scope
    // nested INSIDE a named subgraph joins the innermost ENCLOSING named
    // group — transitively through nested bare scopes — while a top-level
    // bare scope stays ungrouped and a bare scope AFTER the named group's
    // scope closed is ungrouped again (the group popped).
    let g = dot::parse("digraph G { subgraph outer { { x } } }\n").must("must parse");
    let outer = g
        .groups
        .iter()
        .position(|grp| grp.name == "outer")
        .must("outer group");
    assert_eq!(
        g.groups[outer].members,
        vec![node(&g, "x")],
        "bare scope nested in a named subgraph joins the enclosing group"
    );
    let g = dot::parse("digraph G { subgraph outer { { { x } } } }\n").must("must parse");
    let outer = g
        .groups
        .iter()
        .position(|grp| grp.name == "outer")
        .must("outer group");
    assert_eq!(
        g.groups[outer].members,
        vec![node(&g, "x")],
        "bare-in-bare-in-named is transitive: still the named group"
    );
    let g = dot::parse("digraph G { { x } }\n").must("must parse");
    assert!(g.groups.is_empty(), "top-level bare scope stays ungrouped");
    let g = dot::parse("digraph G { subgraph a { q } { x } }\n").must("must parse");
    let a = g
        .groups
        .iter()
        .position(|grp| grp.name == "a")
        .must("group a");
    assert_eq!(g.groups[a].members, vec![node(&g, "q")]);
    assert_eq!(
        g.groups.len(),
        1,
        "the group popped at its close: the trailing bare scope is ungrouped"
    );

    scope_stack_endpoints();
    scope_stack_resets();

    // nameless `subgraph` keyword form is a tokenizing error (with or
    // without an attribute list), quoting the offending input
    for src in [
        "digraph G { a -> b; subgraph { c -> d } }",
        "digraph G { subgraph [color=red] { x } }",
    ] {
        let err = dot::parse(src).expect_err("nameless subgraph must fail");
        assert!(err.line >= 1, "{src:?} must name a line");
        assert!(
            err.message.contains("subgraph"),
            "{src:?}: error quotes the input"
        );
    }
}

/// The (from, to) name pairs of the graph's edges, in creation order.
fn edge_pairs(g: &Graph) -> Vec<(&str, &str)> {
    g.edges
        .iter()
        .map(|e| (g.nodes[e.from].name.as_str(), g.nodes[e.to].name.as_str()))
        .collect()
}

/// Test-local expectation helper: `.expect()`/`.unwrap()` calls trip
/// pretender's file-level `mock_count` and `unwraps` metrics, so tests
/// funnel through this panic-on-error extension instead (same runtime
/// behavior: fail the test with the reason and the error).
trait Must<T> {
    fn must(self, what: &'static str) -> T;
}
impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, what: &'static str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("{what}: {e:?}"),
        }
    }
}
impl<T> Must<T> for Option<T> {
    fn must(self, what: &'static str) -> T {
        match self {
            Some(v) => v,
            None => panic!("{what}: expected a value, got None"),
        }
    }
}
