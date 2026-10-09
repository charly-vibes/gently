//! Unit tests for the ge.text_parser grammar internals (gently-bzx);
//! the capability contract lives in tests/scenarios/ge_text_parser.rs.

use super::parse;
use crate::graph::{Edge, Graph, Node};

#[test]
fn tracer_line_yields_tracer_graph() {
    let g = parse("[ a ] --> [ b ]\n").expect("must parse");
    assert_eq!(g, Graph::tracer());
}

#[test]
fn duplicate_edges_share_interned_nodes() {
    let g = parse("[ a ] --> [ b ]\n[ a ] --> [ b ]\n").expect("must parse");
    assert_eq!(g.nodes.len(), 2);
    assert_eq!(g.edges, vec![Edge::directed(0, 1), Edge::directed(0, 1),]);
}

#[test]
fn errors_carry_one_based_lines() {
    let e = parse("[ a ]\nnope\n").expect_err("line 2 is junk");
    assert_eq!(e.line, 2);
    let e = parse("[ a ]\n[ b ]\n[ c -->\n").expect_err("line 3 is junk");
    assert_eq!(e.line, 3);
}

#[test]
fn missing_right_node_is_an_error() {
    assert!(parse("[ a ] -->\n").is_err());
    // ge.text_parser c1 (gently-bzx): the bare [ ] is an anonymous node
    let g = parse("[ a ] --> [ ]\n").expect("bare [] is an anon node");
    assert_eq!(g.nodes[1].name, "#1");
}

#[test]
fn bracket_soup_does_not_panic() {
    assert!(parse("]]][[[\n").is_err());
    assert!(parse("[\n").is_err());
    assert!(parse("]\n").is_err());
    assert!(parse("-->\n").is_err());
}

#[test]
fn blank_lines_and_lone_nodes_parse() {
    let g = parse("\n[ a ]\n\n  \n[ b ]\n").expect("must parse");
    assert_eq!(g.nodes, vec![Node::named("a"), Node::named("b")]);
    assert!(g.edges.is_empty());
}
