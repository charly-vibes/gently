use super::misc::*;
use super::*;

/// ge.dot_parser.p5 (c5): record labels (shape=record with a vertical bar)
/// and HTML-like table labels autosplit into `name.N` part nodes, port
/// markers stripped and edges reattached to the referenced part; plain
/// endpoints of a split node reattach to its first part; the failing
/// constructs (nameless `subgraph`, malformed HTML-like label, unresolvable
/// port reference) produce the oracle's typed errors and no partial graph.
#[test]
fn p5() {
    // record label WITHOUT shape=record: no split — the label stays an attr
    let g = dot::parse("digraph G { a [label=\"A|B\"] a -> b }\n").must("must parse");
    assert_eq!(g.nodes.len(), 2, "no autosplit without shape=record");
    assert_eq!(g.nodes[node(&g, "a")].attributes.get("label"), Some("A|B"));

    // shape=record without a pipe: no split either
    let g = dot::parse("digraph G { a [shape=record label=\"AB\"] }\n").must("must parse");
    assert_eq!(g.nodes.len(), 1, "no autosplit without a vertical bar");
    assert_eq!(g.nodes[0].name, "a");

    // record autosplit: parts a.0/a.1, the original becomes the label text
    // with `basename` set (observed: `[ A|B ] { basename: a; }`), and the
    // plain endpoint reattaches to the first part
    let g = dot::parse("digraph G { a [shape=record label=\"A|B\"] a -> b }\n").must("must parse");
    node(&g, "a.0");
    node(&g, "a.1");
    node(&g, "b");
    assert_eq!(
        g.edges[0].from,
        node(&g, "a.0"),
        "plain endpoint reattaches to the first part"
    );
    assert_eq!(g.edges[0].to, node(&g, "b"));
    let orig = g
        .nodes
        .iter()
        .position(|n| n.name == "A|B")
        .must("renamed original");
    assert_eq!(
        g.nodes[orig].attributes.get("basename"),
        Some("a"),
        "record split pins basename"
    );

    // explicit ports resolve to their parts; compass ports resolve to part 0
    let g = dot::parse("digraph G { a [shape=record label=\"A|B\"] a:0 -> b c -> a:1 }\n")
        .must("must parse");
    assert_eq!(
        (g.edges[0].from, g.edges[0].to),
        (node(&g, "a.0"), node(&g, "b"))
    );
    assert_eq!(
        (g.edges[1].from, g.edges[1].to),
        (node(&g, "c"), node(&g, "a.1"))
    );
    let g =
        dot::parse("digraph G { a [shape=record label=\"A|B\"] a:n -> b }\n").must("must parse");
    assert_eq!(
        g.edges[0].from,
        node(&g, "a.0"),
        "compass port resolves to the first part"
    );

    html_table_autosplits();
    failing_constructs_are_typed_errors();
}
