//! Unit tests for the txt emitter — oracle-captured expected forms.

use crate::graph::{Edge, Graph, Node, Scope};

#[test]
fn edge_less_node_emits_bare_node_line() {
    let g = Graph {
        nodes: vec![Node::named("a"), Node::named("b")],
        edges: vec![Edge::directed(0, 1), Edge::directed(0, 1)],
        ..Graph::default()
    };
    let mut g2 = g.clone();
    g2.nodes.push(Node::named("c"));
    assert_eq!(
        super::render(&g2),
        "[ a ] --> [ b ]\n[ a ] --> [ b ]\n[ c ]\n"
    );
}

#[test]
fn empty_graph_emits_empty_output() {
    assert_eq!(super::render(&Graph::default()), "");
}

/// Oracle: undirected styles with a trailing space double the operator
/// (`- - ` for dashed, `= = ` for double-dash).
#[test]
fn undirected_trailing_space_styles_double() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let e = g.add_edge(a, b, false).expect("live");
    g.edges[e].arrows = Default::default();
    g.set_attr(Scope::Edge(e), "style", "dashed");
    assert_eq!(super::render(&g), "[ a ] - -  [ b ]\n");
    g.set_attr(Scope::Edge(e), "style", "double-dash");
    assert_eq!(super::render(&g), "[ a ] = =  [ b ]\n");
    g.set_attr(Scope::Edge(e), "style", "solid");
    assert_eq!(super::render(&g), "[ a ] -- [ b ]\n");
}

/// Oracle: node names escape `[`, `]`, `|`, `{`, `}`, `#`.
#[test]
fn node_names_escape_brackets_and_friends() {
    let mut g = Graph::default();
    g.add_node("a|b");
    assert_eq!(super::render(&g), "[ a\\|b ]\n");
}

/// Oracle: a named group section renders as `( name )` plus its
/// instance attributes, followed by a blank line.
#[test]
fn group_sections_render_sorted_with_attributes() {
    let mut g = Graph::default();
    g.add_group("B");
    let a = g.add_group("A");
    g.set_attr(Scope::Group(a), "fill", "#ffccaa");
    assert_eq!(super::render(&g), "( A ) { fill: #ffccaa; }\n\n( B )\n\n");
}

/// A member-bearing group emits the upstream section shape: the header
/// `( name`, member node declarations (attributes included) indented
/// two spaces, then `)` — membership round-trips (gently-8jf).
#[test]
fn group_members_render_inside_the_section() {
    let mut g = Graph::default();
    let x = g.add_node("x");
    let y = g.add_node("y");
    let a = g.add_group("A");
    g.set_node_group(x, a);
    g.set_node_group(y, a);
    g.set_attr(Scope::Node(x), "fill", "red");
    assert_eq!(
        super::render(&g),
        "( A\n  [ x ] { fill: red; }\n  [ y ]\n)\n\n"
    );
}
