//! ge.dot_parser (gently-89p): the Graphviz DOT grammar — each property row
//! of specs/ge-dot_parser.md is one test below, named for its id. The pinned
//! upstream oracle (Graph::Easy 0.69 Parser::Graphviz @ ededa3d7) is
//! authoritative; the probe evidence in tests/repro/claims/*.observed pins
//! the binding observed behavior (the EDGE OPERATOR decides direction, not
//! the header; any named subgraph becomes a group verbatim; records and
//! HTML-like labels autosplit into `name.N` part nodes).

use gently_core::graph::{Graph, Scope};
use gently_core::parse::dot;

/// The index of the node with `name` (panics when missing — tests only).
fn node(g: &Graph, name: &str) -> usize {
    g.node_by_name(name).unwrap_or_else(|| panic!("node {name:?} must exist"))
}

/// ge.dot_parser.p1 (c1): a `graph` header sets the model graph's `type`
/// attribute to `undirected`; a `digraph` header leaves `type` unset.
/// Neither header fixes edge direction (p2 covers the operator rule).
#[test]
fn p1() {
    let g = dot::parse("graph G { a -- b }\n").expect("must parse");
    assert_eq!(g.attributes.get("type"), Some("undirected"), "graph header");
    let g = dot::parse("digraph G { a -> b }\n").expect("must parse");
    assert_eq!(g.attributes.get("type"), None, "digraph header");
    // the header name is the graph title
    let g = dot::parse("digraph G { a -> b }\n").expect("must parse");
    assert_eq!(g.attributes.get("title"), Some("G"), "header name is the title");
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
    let g = dot::parse("digraph G { a -> b -> c }\n").expect("must parse");
    assert_eq!(g.edges.len(), 2, "one edge per arrow in a chain");
    assert_eq!(g.edges[0].to, g.edges[1].from, "b is shared");
    // mixed operators in one chain stay independent
    let g = dot::parse("digraph G { a -> b -- c }\n").expect("must parse");
    assert!(g.edges[0].directed, "-> stays directed");
    assert!(!g.edges[1].directed, "-- stays undirected");
}

/// ge.dot_parser.p3 (c3): attribute lists `[k=v, k2=v2]` map onto the model
/// attributes of the right object (node or edge), with quoted values
/// unescaped exactly once.
#[test]
fn p3() {
    // node attributes, quoted value with an escaped quote and backslash
    let g = dot::parse("digraph G { a [color=red, label=\"x \\\"y\\\\\" ] }\n").expect("must parse");
    let a = node(&g, "a");
    assert_eq!(g.nodes[a].attributes.get("color"), Some("red"));
    assert_eq!(g.nodes[a].attributes.get("label"), Some("x \"y\\"), "unescaped exactly once");
    // edge attributes land on the edge
    let g = dot::parse("digraph G { a -> b [style=dotted] }\n").expect("must parse");
    assert_eq!(g.edges[0].attributes.get("style"), Some("dotted"));
    // bare values and multiple pairs on edges
    let g = dot::parse("digraph G { a -> b [dir=both, weight=2] }\n").expect("must parse");
    assert_eq!(g.edges[0].attributes.get("dir"), Some("both"));
    assert_eq!(g.edges[0].attributes.get("weight"), Some("2"));
}

/// ge.dot_parser.p4 (c4): a named subgraph becomes a model group with the
/// name verbatim containing only its directly declared nodes (nested nodes
/// belong to the innermost group); a bare `{ }` scope keeps its nodes
/// ungrouped — with the oracle's pinned left-edge-stack caveat — and the
/// nameless `subgraph` keyword form (with or without an attribute list) is
/// a tokenizing error.
#[test]
fn p4() {
    // named subgraphs become groups under their verbatim names
    let g = dot::parse("digraph G { a -> b; subgraph cluster_x { c d } subgraph named { e } }\n")
        .expect("must parse");
    let cx = g.groups.iter().position(|grp| grp.name == "cluster_x").expect("cluster_x group");
    let nm = g.groups.iter().position(|grp| grp.name == "named").expect("named group");
    assert_eq!(g.groups[cx].members, vec![node(&g, "c"), node(&g, "d")]);
    assert_eq!(g.groups[nm].members, vec![node(&g, "e")]);

    // nested subgraphs: each level its own group, innermost membership wins
    let g =
        dot::parse("digraph G { subgraph outer { subgraph inner { a } b } }\n").expect("must parse");
    let inner = g.groups.iter().position(|grp| grp.name == "inner").expect("inner group");
    let outer = g.groups.iter().position(|grp| grp.name == "outer").expect("outer group");
    assert_eq!(g.groups[inner].members, vec![node(&g, "a")], "innermost membership");
    assert_eq!(g.groups[outer].members, vec![node(&g, "b")], "directly declared only");

    // bare { } scope: nodes ungrouped; the left-edge stack survives the
    // scope boundary, so `a -> b; { c -> d }` also yields the pinned
    // spurious `b -> d` edge (observed, Graph::Easy v0.69 @ ededa3d7)
    let g = dot::parse("digraph G { a -> b; { c -> d } }\n").expect("must parse");
    assert!(g.groups.is_empty(), "bare scope keeps nodes ungrouped");
    assert_eq!(g.edges.len(), 3, "a->b, c->d, plus the pinned spurious b->d");
    assert_eq!((g.edges[0].from, g.edges[0].to), (node(&g, "a"), node(&g, "b")));
    assert_eq!((g.edges[1].from, g.edges[1].to), (node(&g, "c"), node(&g, "d")));
    assert_eq!((g.edges[2].from, g.edges[2].to), (node(&g, "b"), node(&g, "d")), "spurious edge");
    // a bare scope as an edge target links the chain through it
    let g = dot::parse("digraph G { a -> { b } }\n").expect("must parse");
    assert!(g.groups.is_empty());
    assert_eq!((g.edges[0].from, g.edges[0].to), (node(&g, "a"), node(&g, "b")));

    // nameless `subgraph` keyword form is a tokenizing error (with or
    // without an attribute list), quoting the offending input
    for src in ["digraph G { a -> b; subgraph { c -> d } }",
        "digraph G { subgraph [color=red] { x } }"]
    {
        let err = dot::parse(src).expect_err("nameless subgraph must fail");
        assert!(err.line >= 1, "{src:?} must name a line");
        assert!(err.message.contains("subgraph"), "{src:?}: error quotes the input");
    }
}

/// ge.dot_parser.p5 (c5): record labels (shape=record with a vertical bar)
/// and HTML-like table labels autosplit into `name.N` part nodes, port
/// markers stripped and edges reattached to the referenced part; plain
/// endpoints of a split node reattach to its first part; the failing
/// constructs (nameless `subgraph`, malformed HTML-like label, unresolvable
/// port reference) produce the oracle's typed errors and no partial graph.
#[test]
fn p5() {
    // record label WITHOUT shape=record: no split — the label stays an attr
    let g = dot::parse("digraph G { a [label=\"A|B\"] a -> b }\n").expect("must parse");
    assert_eq!(g.nodes.len(), 2, "no autosplit without shape=record");
    assert_eq!(g.nodes[node(&g, "a")].attributes.get("label"), Some("A|B"));

    // shape=record without a pipe: no split either
    let g = dot::parse("digraph G { a [shape=record label=\"AB\"] }\n").expect("must parse");
    assert_eq!(g.nodes.len(), 1, "no autosplit without a vertical bar");
    assert_eq!(g.nodes[0].name, "a");

    // record autosplit: parts a.0/a.1, the original becomes the label text
    // with `basename` set (observed: `[ A|B ] { basename: a; }`), and the
    // plain endpoint reattaches to the first part
    let g = dot::parse("digraph G { a [shape=record label=\"A|B\"] a -> b }\n").expect("must parse");
    node(&g, "a.0");
    node(&g, "a.1");
    node(&g, "b");
    assert_eq!(g.edges[0].from, node(&g, "a.0"), "plain endpoint reattaches to the first part");
    assert_eq!(g.edges[0].to, node(&g, "b"));
    let orig = g.nodes.iter().position(|n| n.name == "A|B").expect("renamed original");
    assert_eq!(g.nodes[orig].attributes.get("basename"), Some("a"), "record split pins basename");

    // explicit ports resolve to their parts; compass ports resolve to part 0
    let g =
        dot::parse("digraph G { a [shape=record label=\"A|B\"] a:0 -> b c -> a:1 }\n")
            .expect("must parse");
    assert_eq!((g.edges[0].from, g.edges[0].to), (node(&g, "a.0"), node(&g, "b")));
    assert_eq!((g.edges[1].from, g.edges[1].to), (node(&g, "c"), node(&g, "a.1")));
    let g = dot::parse("digraph G { a [shape=record label=\"A|B\"] a:n -> b }\n").expect("must parse");
    assert_eq!(g.edges[0].from, node(&g, "a.0"), "compass port resolves to the first part");

    html_table_autosplits();
    failing_constructs_are_typed_errors();
}

/// HTML-like table label autosplits into one part per <TD> cell.
fn html_table_autosplits() {
    let g = dot::parse(
        "digraph G { a [label=<<TABLE><TR><TD>one</TD><TD>two</TD></TR></TABLE>>] a -> b }\n",
    )
    .expect("must parse");
    node(&g, "a.0");
    node(&g, "a.1");
    assert_eq!(g.edges[0].from, node(&g, "a.0"));
}

/// The failing constructs: typed errors naming a line and quoting the
/// offending input (tokenizing) or the `base:port` and edge id (ports).
fn failing_constructs_are_typed_errors() {
    // malformed HTML-like label (non-tag text inside the label)
    let err = dot::parse("digraph G { a [label=<<table>...</table>>] }\n")
        .expect_err("malformed HTML-like label must fail");
    assert!(err.line >= 1);
    assert!(err.message.contains("<<table>...</table>>"), "error quotes the input: {}", err.message);
    // unresolvable port reference names base:port and the edge id
    for src in ["digraph G { b:p1 -> c }\n",
        "digraph G { a [shape=record label=\"A|B\"] a:zz -> b }\n",
        "digraph G { b:nw -> c }\n"]
    {
        let err = dot::parse(src).expect_err("port without a matching part must fail");
        let head = err.message.split(" on edge").next().unwrap_or("");
        assert!(
            head.starts_with("Cannot find autosplit node for ") && head.contains(':'),
            "oracle error text for {src:?}, got: {}", err.message
        );
        assert!(err.message.contains(" on edge "), "the edge id is named: {}", err.message);
    }
}
