//! ge.text_parser.p11 (c11) scenario — comma-separated node lists: fan-out
//! reuses the operator's remembered left side for each follower, fan-in
//! links every list element, attribute blocks share across the list, the
//! statement stack survives line breaks, and a new bare node statement
//! resets the remembered side. Pinned bytes:
//! tests/repro/claims/bg-comma-lists.observed (Graph::Easy v0.69 @ ededa3d7).

use super::node;
use gently_core::graph::Scope;
use gently_core::parse::text;

/// Every edge as its `(from, to)` node-index pair, in declaration order.
fn edge_pairs(g: &gently_core::graph::Graph) -> Vec<(usize, usize)> {
    g.edges.iter().map(|e| (e.from, e.to)).collect()
}

/// `[ A ] -> [ B ], [ C ], [ D ]` fans out from A (oracle as_txt:
/// A->B, A->C, A->D — three edges over four shared nodes).
#[test]
fn fan_out_reuses_the_remembered_left_side() {
    let g = text::parse("[ A ] -> [ B ], [ C ], [ D ]\n").expect("must parse");
    assert_eq!(g.nodes.len(), 4);
    let (a, b, c, d) = (node(&g, "A"), node(&g, "B"), node(&g, "C"), node(&g, "D"));
    assert_eq!(edge_pairs(&g), vec![(a, b), (a, c), (a, d)], "fan-out from A");
}

/// `[ A ], [ B ], [ C ] --> [ D ]` links every list element to D (fan-in).
#[test]
fn fan_in_links_every_list_element() {
    let g = text::parse("[ A ], [ B ], [ C ] --> [ D ]\n").expect("must parse");
    let (a, b, c, d) = (node(&g, "A"), node(&g, "B"), node(&g, "C"), node(&g, "D"));
    assert_eq!(edge_pairs(&g), vec![(a, d), (b, d), (c, d)], "fan-in to D");
}

/// A trailing attribute block on a list element applies to every node
/// listed so far and itself; a bare attribute block after a list applies
/// to the whole stack (upstream _new_node's stack pass / stack-top attrs).
#[test]
fn attribute_blocks_share_across_the_list() {
    for src in ["[ A ], [ B ] { color: red; }\n", "[ A ], [ B ]\n{ color: red; }\n"] {
        let g = text::parse(src).expect("must parse");
        assert_eq!(g.get_attr(Scope::Node(node(&g, "A")), "color"), Some("red"), "{src:?}: A shares");
        assert_eq!(g.get_attr(Scope::Node(node(&g, "B")), "color"), Some("red"), "{src:?}: B shares");
    }
}

/// The statement stack and remembered operator survive line breaks; a new
/// bare node statement resets the remembered left side.
#[test]
fn lists_continue_across_lines_until_reset() {
    let g = text::parse("[ A ] --> [ B ]\n, [ C ]\n").expect("must parse");
    let (a, b, c) = (node(&g, "A"), node(&g, "B"), node(&g, "C"));
    assert_eq!(edge_pairs(&g), vec![(a, b), (a, c)], "the comma continues the fan across lines");
    let g = text::parse("[ A ] --> [ B ]\n[ C ], [ D ]\n").expect("must parse");
    let (a, b) = (node(&g, "A"), node(&g, "B"));
    assert_eq!(edge_pairs(&g), vec![(a, b)], "[ C ] resets; C, D stay unlinked");
    assert_eq!(g.nodes.len(), 4);
}

/// A bidirectional operator fans out with both arrows on every edge.
#[test]
fn bidirectional_fan_keeps_both_arrows() {
    let g = text::parse("[ A ] <-> [ B ], [ C ]\n").expect("must parse");
    assert_eq!(g.edges.len(), 2);
    for e in &g.edges {
        assert!(e.arrows.start && e.arrows.end, "bidirectional fan edge");
    }
}