//! ge.text_parser (gently-bzx): the Graph::Easy text-format grammar — each
//! property row of specs/ge-text_parser.md is one test below, named for its
//! id. The pinned oracle (Graph::Easy 0.69 @ ededa3d7) is authoritative; the
//! probe evidence in tests/repro/claims/*.observed pins the observed bytes.

use gently_core::graph::{Graph, ObjectKind, Scope};
use gently_core::parse::text;

/// The `style` attribute of edge `idx` (absent means solid/inherit).
fn style(g: &Graph, idx: usize) -> Option<String> {
    g.get_attr(Scope::Edge(idx), "style").map(String::from)
}

/// The index of the node with `name` (panics when missing — tests only).
fn node(g: &Graph, name: &str) -> usize {
    g.node_by_name(name)
        .unwrap_or_else(|| panic!("node {name:?} must exist"))
}

/// The member list of group `idx`, as node indices.
fn members(g: &Graph, idx: usize) -> Vec<usize> {
    g.groups[idx].members.clone()
}

/// ge.text_parser.p11 (c11): comma lists (gently-m4i, bg-comma-lists.observed).
#[path = "ge_text_parser_p11.rs"]
mod p11;

/// ge.text_parser.p12 (c12): edges after closed groups (gently-58m, pod-anon-group.observed).
#[path = "ge_text_parser_p12.rs"]
mod p12;

/// ge.text_parser.p13 (c13): trailing node attr blocks (gently-8xd, bg-trailing-node-attr.observed).
#[path = "ge_text_parser_p13.rs"]
mod p13;

#[path = "ge_text_parser/arrows.rs"]
mod arrows;
use arrows::*;
#[path = "ge_text_parser/quotes.rs"]
mod quotes;
use quotes::*;

/// ge.text_parser.p1 (c1): named tokens intern and reuse; a bare `[ ]`
/// creates an anonymous `#N` that a later escaped `[ \#N ]` reuses.
#[test]
fn p1() {
    let g = text::parse("[ a ] --> [ ]\n").must("must parse");
    assert_eq!(g.nodes[1].name, "#1", "the first anon node is #1");
    let g = text::parse("[ a ]\n[ a ]\n").must("must parse");
    assert_eq!(g.nodes.len(), 1, "the repeated token must reuse the node");
    let g = text::parse("[ a ] --> [ ]\n[ b ] --> [ ]\n").must("must parse");
    node(&g, "#1");
    node(&g, "#3");
    assert_eq!(g.nodes.len(), 4, "each bare [] is its own anon node");
    let src = "[ a ] --> [ ]\n[ \\#1 ] --> [ b ]\n";
    let g = text::parse(src).must("the escaped reference must parse");
    assert_eq!(g.nodes.len(), 3, "3 nodes: a, #1, b");
    assert_eq!(
        g.edges[0].to, g.edges[1].from,
        "both edges touch the same anon node"
    );
}

/// ge.text_parser.p2 (c2): the operator table maps to edge styles;
/// `<`-prefixed operators are the same styles bidirectionally; missing
/// endpoints and a lone `<` are errors; style-only styles via attribute.
#[test]
fn p2() {
    let g = text::parse("[ a ] => [ b ]\n").must("must parse");
    assert_eq!(style(&g, 0), Some(String::from("double")));
    directed_ops_map_to_styles();
    bidirectional_ops_mark_both_ends();
    broken_edges_are_errors();
    style_only_styles_come_via_the_attribute();
}

/// ge.text_parser.p3 (c3): a chain creates one edge per adjacent pair, sharing repeated names.
#[test]
fn p3() {
    let g = text::parse("[ a ] --> [ b ] --> [ c ]\n").must("must parse");
    assert_eq!(g.nodes.len(), 3);
    assert_eq!(g.edges.len(), 2);
    assert_eq!((g.edges[0].from, g.edges[0].to), (0, 1));
    assert_eq!((g.edges[1].from, g.edges[1].to), (1, 2));
    let g = text::parse("[ a ] --> [ b ] --> [ a ]\n").must("must parse");
    assert_eq!(g.nodes.len(), 2, "repeated names share nodes");
    assert_eq!(g.edges.len(), 2);
    assert_eq!((g.edges[1].from, g.edges[1].to), (1, 0), "edge back to a");
    let chain = (0..6)
        .map(|i| format!("[ n{i} ]"))
        .collect::<Vec<_>>()
        .join(" --> ");
    let g = text::parse(&chain).must("chain must parse");
    assert_eq!(g.nodes.len(), 6);
    assert_eq!(g.edges.len(), 5);
}

/// ge.text_parser.p4 (c4): an inline edge label lands only when both
/// flanks match and the edge has an arrow; mismatches are errors.
#[test]
fn p4() {
    let g = text::parse("[ a ] -- go --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("go"));
    assert_eq!(style(&g, 0), None, "solid flanks");
    let g = text::parse("[ a ] -  go - > [ b ]\n").must("the rendered dashed form must parse");
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("go"));
    assert_eq!(style(&g, 0), Some(String::from("dashed")));
    let g = text::parse("[ a ] <.. lbl ..> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("lbl"));
    let e = &g.edges[0];
    assert!(
        e.directed && e.arrows.start && e.arrows.end,
        "bidirectional labeled"
    );
    for src in ["[ a ] -- go -> [ b ]\n", "[ a ] .. go .. [ b ]\n"] {
        let err = text::parse(src).expect_err("must be a parse error");
        assert!(err.line >= 1);
    }
}

/// ge.text_parser.p5 (c5): attribute blocks apply to the nearest preceding object; class sections to the whole class.
#[test]
fn p5() {
    let g = text::parse("[ a ] { color: red; } [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "color"), Some("red"));
    assert_eq!(g.get_attr(Scope::Node(1), "color"), None, "nearest only");
    let g = text::parse("[ a ]\n{ color: red; }\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "color"), Some("red"));
    let src = "[ a ] -->\n{ style: bold; }\n[ b ]\n";
    let g = text::parse(src).must("must parse");
    assert_eq!(
        style(&g, 0),
        Some(String::from("bold")),
        "edge attrs across lines"
    );
    // a block after a completed edge line targets the edge's right node (the oracle's stack-top)
    let src = "[ a ] --> [ b ]\n{ color: red; }\n";
    let g = text::parse(src).must("must parse");
    assert_eq!(
        g.get_attr(Scope::Node(1), "color"),
        Some("red"),
        "last node target"
    );
    let g = text::parse("( G: [ a ] ) { color: red; } [ c ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Group(0), "color"), Some("red"));
    let src = "graph { x: 1; }\nnode { y: 2; }\nedge { z: 3; }\ngroup { w: 4; }\n[ a ]\n";
    let g = text::parse(src).must("must parse");
    assert_eq!(g.get_attr(Scope::Graph, "x"), Some("1"));
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Node, String::new()), "y"),
        Some("2")
    );
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Edge, String::new()), "z"),
        Some("3")
    );
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Group, String::new()), "w"),
        Some("4")
    );
}

/// ge.text_parser.p6 (c6): group blocks — colon names, one-group
/// membership with move-on-redeclare, nesting, anonymous `Group #N`.
#[test]
fn p6() {
    let g = text::parse("( G: [ a ] )\n").must("must parse");
    assert_eq!(g.groups[0].name, "G:", "the colon joins the name");
    assert_eq!(members(&g, 0), vec![0]);
    let g = text::parse("( G: [ a ] --> [ b ] )\n").must("must parse");
    assert_eq!(members(&g, 0), vec![0, 1]);
    let g = text::parse("( A [ a ] [ c ] )\n").must("must parse");
    assert_eq!(g.groups[0].name, "A", "no colon, no colon in the name");
    assert_eq!(members(&g, 0), vec![0, 1]);
    let g = text::parse("( A: [ a ] )\n( B: [ a ] )\n").must("must parse");
    assert_eq!(
        members(&g, 0),
        Vec::<usize>::new(),
        "the earlier group empties"
    );
    assert_eq!(members(&g, 1), vec![0], "the later group owns the node");
    let src = "( A [ a ] ( B: [ b ] ) [ c ] )\n";
    let g = text::parse(src).must("must parse");
    assert_eq!(g.groups.len(), 2);
    let (a, b) = (node(&g, "a"), node(&g, "b"));
    assert_eq!(
        members(&g, 0),
        vec![a, node(&g, "c")],
        "outer keeps direct nodes"
    );
    assert_eq!(members(&g, 1), vec![b], "inner-only membership");
    let g = text::parse("( [ a ] )\n").must("must parse");
    assert_eq!(g.groups[0].name, "Group #0", "anonymous group");
    assert_eq!(members(&g, 0), vec![0]);
}

/// ge.text_parser.p7 (c7): an unescaped `#` truncates everywhere (even
/// inside quotes); `\#` escapes it; hex colours after a separator pass.
#[test]
fn p7() {
    let g = text::parse("[ a ] # junk\n").must("must parse");
    assert_eq!(g.nodes.len(), 1, "mid-line comments truncate");
    let g = text::parse("# junk\n[ a ]\n   # indented junk\n").must("must parse");
    assert_eq!(g.nodes.len(), 1, "comment-only lines are dropped");
    let err = text::parse("[ a ] { label: \"x # y\"; }\n").expect_err("truncated in-string");
    assert!(err.line >= 1);
    let g = text::parse("[ \\#a ] --> [ b ]\n").must("must parse");
    assert_eq!(node(&g, "#a"), 0, "the escape collapses to #");
    let g = text::parse("[ a ] { label: x \\# y; } [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("x # y"));
    let cases = [
        ("[ a ] { color: #ff0000; }\n", "#ff0000"),
        ("[ a ] { color: #f00; }\n", "#f00"),
    ];
    for (src, want) in cases {
        let g = text::parse(src).must("hex colour must be accepted");
        assert_eq!(g.get_attr(Scope::Node(0), "color"), Some(want));
    }
    for src in ["[ a ] { color: red #ff0000; }\n", "[ #1 ] --> [ b ]\n"] {
        let err = text::parse(src).expect_err("non-separator hash truncates");
        assert!(err.line >= 1);
    }
}

/// ge.text_parser.p8 (c8): a malformed line produces a typed error naming the 1-based line and a reason.
#[test]
fn p8() {
    let cases = [
        ("nope\n", 1),
        ("[ a ] --> [ b ]\nnope\n", 2),
        ("[ a\n", 1),
        ("foo { }\n", 1),
        ("[ a ] junk [ b ]\n", 1),
        ("]][[[\n", 1),
    ];
    for (src, line) in cases {
        let err = text::parse(src).expect_err("malformed input must error");
        assert_eq!(err.line, line, "{src:?} must name its offending line");
        assert!(!err.message.is_empty(), "the error must carry a reason");
        assert!(!format!("{err}").is_empty(), "Display works");
    }
}

/// ge.text_parser.p9 (c9): the unit-token grammar — directed style follows
/// the last unit token; `.-`/`..-` are valid singles; plain units need two
/// reps; singles error. Styles verified against the pinned oracle.
#[test]
fn p9() {
    let g = text::parse("[ a ] ..-..-..-> [ b ]\n").must("must parse");
    assert_eq!(style(&g, 0), Some(String::from("dot-dot-dash")));
    directed_style_follows_the_last_unit();
    arrowless_dot_singles_are_valid();
    arrowless_plain_units();
    plain_singles_are_errors();
}

/// ge.text_parser.p10 (c10): attribute-value unquoting, pinned by attr-quote-value.observed.
#[test]
fn p10() {
    // headline: single-quoted values lose their quotes (the bug report)
    let g = text::parse("[ a ] { label: 'hello'; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("hello"));
    quote_pairs_strip();
    store_unescape_set();
    percent_entity_decode();
    quote_split_rules();
    names_and_labels_are_parser_layer_only();
}

/// Node names and edge labels are parser-layer only — no store unquoting.
fn names_and_labels_are_parser_layer_only() {
    let g = text::parse("[ \"n\" ] -- \"l\" --> [ b ]\n").must("must parse");
    assert_eq!(
        g.nodes[node(&g, "\"n\"")].name,
        "\"n\"",
        "names keep their quotes"
    );
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("\"l\""));
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
