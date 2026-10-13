//! ge.text_parser.p12 (c12) scenario — an edge operator after a closed
//! group links the group to the right node (upstream makes the closed
//! group the statement's node), so the POD anonymous-group example parses.
//! Group endpoints are not representable in the model, so no edge is
//! recorded — bounded divergence: even the pinned oracle's own layouter
//! drops the group-endpoint edge. Evidence:
//! tests/repro/claims/pod-anon-group.observed (v0.69 @ ededa3d7).

use super::{members, node};
use gently_core::graph::Scope;
use gently_core::parse::text;

/// The POD anonymous-group example parses; Bonn is Group #0's member and
/// Berlin exists unlinked.
#[test]
fn pod_anon_group_parses() {
    let g = text::parse("( [ Bonn ] ) -> [ Berlin ]\n")
        .expect("the POD anonymous-group example must parse");
    assert_eq!(g.groups.len(), 1);
    assert_eq!(g.groups[0].name, "Group #0", "anonymous group");
    assert_eq!(members(&g, 0), vec![node(&g, "Bonn")], "Bonn is the group's member");
    assert!(g.edges.is_empty(), "group endpoints record no edge");
    assert_eq!(g.nodes.len(), 2, "Bonn and Berlin both exist");
}

/// Named-group variant with an undirected operator: same linkage rule.
#[test]
fn named_group_links_the_same_way() {
    let g = text::parse("( G: [ a ] ) -- [ b ]\n").expect("must parse");
    assert_eq!(g.groups[0].name, "G:");
    assert_eq!(members(&g, 0), vec![node(&g, "a")]);
    assert!(g.edges.is_empty(), "group endpoints record no edge");
}

/// Attributes inside the group body still land on the node.
#[test]
fn group_body_attrs_still_land() {
    let g = text::parse("( [ Bonn ] { color: red; } ) -> [ Berlin ]\n").expect("must parse");
    let bonn = node(&g, "Bonn");
    assert_eq!(g.get_attr(Scope::Node(bonn), "color"), Some("red"));
    assert_eq!(members(&g, 0), vec![bonn]);
}