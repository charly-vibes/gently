//! ge.text_parser (gently-bzx): the Graph::Easy text-format grammar — each
//! property row of specs/ge-text_parser.md is one test below, named for its
//! id. The pinned upstream oracle (Graph::Easy 0.69 Parser.pm @
//! ededa3d787ad89ac532c578c06390e8a7b270499) is authoritative; the probe
//! evidence in tests/repro/claims/*.observed pins the binding observed
//! behavior (never the POD, which diverges on `..-..-..>` and `.-`).

use gently_core::graph::{Graph, ObjectKind, Scope};
use gently_core::parse::text;

/// The `style` attribute of edge `idx` (absent means solid/inherit).
fn style(g: &Graph, idx: usize) -> Option<String> {
    g.get_attr(Scope::Edge(idx), "style").map(String::from)
}

/// The index of the node with `name` (panics when missing — tests only).
fn node(g: &Graph, name: &str) -> usize {
    g.node_by_name(name).unwrap_or_else(|| panic!("node {name:?} must exist"))
}

/// The member list of group `idx`, as node indices.
fn members(g: &Graph, idx: usize) -> Vec<usize> {
    g.groups[idx].members.clone()
}

/// ge.text_parser.p1 (c1): named tokens intern and reuse; a bare `[ ]`
/// creates an anonymous node named `#N` (odd counter) that a later escaped
/// `[ \#N ]` reference reuses as the same node.
#[test]
fn p1() {
    let g = text::parse("[ a ] --> [ ]\n").expect("must parse");
    assert_eq!(g.nodes[1].name, "#1", "the first anon node is #1");
    let g = text::parse("[ a ]\n[ a ]\n").expect("must parse");
    assert_eq!(g.nodes.len(), 1, "the repeated token must reuse the node");
    let g = text::parse("[ a ] --> [ ]\n[ b ] --> [ ]\n").expect("must parse");
    node(&g, "#1");
    node(&g, "#3");
    assert_eq!(g.nodes.len(), 4, "each bare [] is its own anon node");
    let src = "[ a ] --> [ ]\n[ \\#1 ] --> [ b ]\n";
    let g = text::parse(src).expect("the escaped reference must parse");
    assert_eq!(g.nodes.len(), 3, "3 nodes: a, #1, b");
    assert_eq!(g.edges[0].to, g.edges[1].from, "both edges touch the same anon node");
}

/// ge.text_parser.p2 (c2): the operator table maps to edge styles;
/// `<`-prefixed operators are the same styles with bidirectional arrows;
/// missing endpoints and a lone `<` are parse errors; style-only styles
/// are settable via the attribute only.
#[test]
fn p2() {
    let g = text::parse("[ a ] => [ b ]\n").expect("must parse");
    assert_eq!(style(&g, 0), Some(String::from("double")));
    directed_ops_map_to_styles();
    bidirectional_ops_mark_both_ends();
    broken_edges_are_errors();
    style_only_styles_come_via_the_attribute();
}

/// Every directed operator maps to its documented style (solid edges carry
/// no style attribute — solid is the default).
fn directed_ops_map_to_styles() {
    let cases = [
        ("->", None),
        (".>", Some("dotted")),
        ("~>", Some("wave")),
        ("- >", Some("dashed")),
        (".->", Some("dot-dash")),
        ("..->", Some("dot-dot-dash")),
        ("= >", Some("double-dash")),
    ];
    for (op, want) in cases {
        let src = format!("[ a ] {op} [ b ]\n");
        let g = text::parse(&src).unwrap_or_else(|e| panic!("{op:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator {op:?}");
    }
}

/// `<`-prefixed operators keep the style and mark both arrow ends.
fn bidirectional_ops_mark_both_ends() {
    let cases = [
        ("<->", None),
        ("<=>", Some("double")),
        ("<.>", Some("dotted")),
        ("<~>", Some("wave")),
        ("<- >", Some("dashed")),
        ("<.->", Some("dot-dash")),
        ("<..->", Some("dot-dot-dash")),
        ("<= >", Some("double-dash")),
    ];
    for (op, want) in cases {
        let src = format!("[ a ] {op} [ b ]\n");
        let g = text::parse(&src).unwrap_or_else(|e| panic!("{op:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator {op:?}");
        let e = &g.edges[0];
        assert!(e.directed && e.arrows.start && e.arrows.end, "{op:?} bidirectional");
    }
}

/// A lone `<`, a missing left node and a missing right node are errors.
fn broken_edges_are_errors() {
    for src in ["[ a ] <--\n", "--> [ b ]\n", "[ a ] -->\n"] {
        let err = text::parse(src).expect_err("must be a parse error");
        assert!(err.line >= 1, "{src:?} must name a line");
    }
}

/// bold/wide/broad have no operator spelling; `{ style: X; }` works.
fn style_only_styles_come_via_the_attribute() {
    let g = text::parse("[ a ] --> { style: bold; } [ b ]\n").expect("must parse");
    assert_eq!(style(&g, 0), Some(String::from("bold")));
    for src in ["[ a ] bold [ b ]\n", "[ a ] wide [ b ]\n"] {
        assert!(text::parse(src).is_err(), "{src:?} has no operator spelling");
    }
}

/// ge.text_parser.p3 (c3): a chain creates exactly one edge per adjacent
/// pair, sharing node objects for repeated names.
#[test]
fn p3() {
    let g = text::parse("[ a ] --> [ b ] --> [ c ]\n").expect("must parse");
    assert_eq!(g.nodes.len(), 3);
    assert_eq!(g.edges.len(), 2);
    assert_eq!((g.edges[0].from, g.edges[0].to), (0, 1));
    assert_eq!((g.edges[1].from, g.edges[1].to), (1, 2));
    let g = text::parse("[ a ] --> [ b ] --> [ a ]\n").expect("must parse");
    assert_eq!(g.nodes.len(), 2, "repeated names share nodes");
    assert_eq!(g.edges.len(), 2);
    assert_eq!((g.edges[1].from, g.edges[1].to), (1, 0), "edge back to a");
    let src: Vec<String> = (0..6).map(|i| format!("[ n{i} ]")).collect();
    let g = text::parse(&src.join(" --> ")).expect("chain must parse");
    assert_eq!(g.nodes.len(), 6);
    assert_eq!(g.edges.len(), 5);
}

/// ge.text_parser.p4 (c4): an inline edge label sets the label only when
/// both flanking patterns match and the edge has an arrow; mismatched
/// flanks and arrow-less inline labels are parse errors.
#[test]
fn p4() {
    let g = text::parse("[ a ] -- go --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("go"));
    assert_eq!(style(&g, 0), None, "solid flanks");
    let g = text::parse("[ a ] -  go - > [ b ]\n").expect("the rendered dashed form must parse");
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("go"));
    assert_eq!(style(&g, 0), Some(String::from("dashed")));
    let g = text::parse("[ a ] <.. lbl ..> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("lbl"));
    let e = &g.edges[0];
    assert!(e.directed && e.arrows.start && e.arrows.end, "bidirectional labeled");
    for src in ["[ a ] -- go -> [ b ]\n", "[ a ] .. go .. [ b ]\n"] {
        let err = text::parse(src).expect_err("must be a parse error");
        assert!(err.line >= 1);
    }
}

/// ge.text_parser.p5 (c5): attribute blocks apply to the nearest
/// preceding object; class sections apply to the whole class.
#[test]
fn p5() {
    let g = text::parse("[ a ] { color: red; } [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "color"), Some("red"));
    assert_eq!(g.get_attr(Scope::Node(1), "color"), None, "nearest only");
    let g = text::parse("[ a ]\n{ color: red; }\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "color"), Some("red"));
    let src = "[ a ] -->\n{ style: bold; }\n[ b ]\n";
    let g = text::parse(src).expect("must parse");
    assert_eq!(style(&g, 0), Some(String::from("bold")), "edge attrs across lines");
    // a block after a completed edge line targets the edge's right node
    // (the oracle's stack-top; verified under the pinned oracle)
    let src = "[ a ] --> [ b ]\n{ color: red; }\n";
    let g = text::parse(src).expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(1), "color"), Some("red"), "last node target");
    let g = text::parse("( G: [ a ] ) { color: red; } [ c ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Group(0), "color"), Some("red"));
    let src = "graph { x: 1; }\nnode { y: 2; }\nedge { z: 3; }\ngroup { w: 4; }\n[ a ]\n";
    let g = text::parse(src).expect("must parse");
    assert_eq!(g.get_attr(Scope::Graph, "x"), Some("1"));
    assert_eq!(g.get_attr(Scope::Class(ObjectKind::Node, String::new()), "y"), Some("2"));
    assert_eq!(g.get_attr(Scope::Class(ObjectKind::Edge, String::new()), "z"), Some("3"));
    assert_eq!(g.get_attr(Scope::Class(ObjectKind::Group, String::new()), "w"), Some("4"));
}

/// ge.text_parser.p6 (c6): group blocks — colon names, one-group
/// membership with move-on-redeclare, nested inner-only containment,
/// anonymous `Group #N`.
#[test]
fn p6() {
    let g = text::parse("( G: [ a ] )\n").expect("must parse");
    assert_eq!(g.groups[0].name, "G:", "the colon joins the name");
    assert_eq!(members(&g, 0), vec![0]);
    let g = text::parse("( G: [ a ] --> [ b ] )\n").expect("must parse");
    assert_eq!(members(&g, 0), vec![0, 1]);
    let g = text::parse("( A [ a ] [ c ] )\n").expect("must parse");
    assert_eq!(g.groups[0].name, "A", "no colon, no colon in the name");
    assert_eq!(members(&g, 0), vec![0, 1]);
    let g = text::parse("( A: [ a ] )\n( B: [ a ] )\n").expect("must parse");
    assert_eq!(members(&g, 0), Vec::<usize>::new(), "the earlier group empties");
    assert_eq!(members(&g, 1), vec![0], "the later group owns the node");
    let src = "( A [ a ] ( B: [ b ] ) [ c ] )\n";
    let g = text::parse(src).expect("must parse");
    assert_eq!(g.groups.len(), 2);
    let (a, b) = (node(&g, "a"), node(&g, "b"));
    assert_eq!(members(&g, 0), vec![a, node(&g, "c")], "outer keeps direct nodes");
    assert_eq!(members(&g, 1), vec![b], "inner-only membership");
    let g = text::parse("( [ a ] )\n").expect("must parse");
    assert_eq!(g.groups[0].name, "Group #0", "anonymous group");
    assert_eq!(members(&g, 0), vec![0]);
}

/// ge.text_parser.p7 (c7): an unescaped `#` truncates everywhere (even
/// inside quotes); `\#` escapes it; hex colour tokens after an attribute
/// separator are auto-escaped and accepted.
#[test]
fn p7() {
    let g = text::parse("[ a ] # junk\n").expect("must parse");
    assert_eq!(g.nodes.len(), 1, "mid-line comments truncate");
    let g = text::parse("# junk\n[ a ]\n   # indented junk\n").expect("must parse");
    assert_eq!(g.nodes.len(), 1, "comment-only lines are dropped");
    let err = text::parse("[ a ] { label: \"x # y\"; }\n").expect_err("truncated in-string");
    assert!(err.line >= 1);
    let g = text::parse("[ \\#a ] --> [ b ]\n").expect("must parse");
    assert_eq!(node(&g, "#a"), 0, "the escape collapses to #");
    let g = text::parse("[ a ] { label: x \\# y; } [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("x # y"));
    let cases = [("[ a ] { color: #ff0000; }\n", "#ff0000"), ("[ a ] { color: #f00; }\n", "#f00")];
    for (src, want) in cases {
        let g = text::parse(src).expect("hex colour must be accepted");
        assert_eq!(g.get_attr(Scope::Node(0), "color"), Some(want));
    }
    for src in ["[ a ] { color: red #ff0000; }\n", "[ #1 ] --> [ b ]\n"] {
        let err = text::parse(src).expect_err("non-separator hash truncates");
        assert!(err.line >= 1);
    }
}

/// ge.text_parser.p8 (c8): a malformed input line produces a typed parse
/// error naming the 1-based line and a reason, aborting without a graph.
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
/// the last unit token; arrow-less `.-`/`..-` are valid singles; plain
/// units need at least two repetitions; single-character patterns error.
/// All expected styles were verified against the pinned oracle.
#[test]
fn p9() {
    let g = text::parse("[ a ] ..-..-..-> [ b ]\n").expect("must parse");
    assert_eq!(style(&g, 0), Some(String::from("dot-dot-dash")));
    directed_style_follows_the_last_unit();
    arrowless_dot_singles_are_valid();
    arrowless_plain_units();
    plain_singles_are_errors();
}

/// Directed: mixed/repeated units accepted, style = last unit.
fn directed_style_follows_the_last_unit() {
    let cases = [
        ("..-..-..>", Some("dotted")), (".-..-..>", Some("dotted")),
        (".--->", None), ("--.>", Some("dotted")), (".->", Some("dot-dash")),
        ("- >", Some("dashed")), ("= >", Some("double-dash")),
    ];
    for (op, want) in cases {
        let src = format!("[ a ] {op} [ b ]\n");
        let g = text::parse(&src).unwrap_or_else(|e| panic!("{op:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator {op:?}");
    }
}

/// Arrow-less: single `.-`/`..-` valid; mixed dot-units inherit (solid).
fn arrowless_dot_singles_are_valid() {
    let cases = [
        (".-", Some("dot-dash")), ("..-", Some("dot-dot-dash")),
        (".-.-", Some("dot-dash")), ("..", Some("dotted")), ("..-.-", None),
    ];
    for (op, want) in cases {
        let src = format!("[ a ] {op} [ b ]\n");
        let g = text::parse(&src).unwrap_or_else(|e| panic!("{op:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator {op:?}");
        assert!(!g.edges[0].directed, "{op:?} is arrow-less (undirected)");
    }
}

/// Arrow-less plain units: homogeneous reps name the style; mixed inherit.
fn arrowless_plain_units() {
    let cases = [
        ("[ a ] == [ b ]", None), ("[ a ] ==[ b ]", Some("double")),
        ("[ a ] ~~ [ b ]", Some("wave")), ("[ a ] -- [ b ]", None),
        ("[ a ] - -[ b ]", Some("dot-dot-dash")),
        ("[ a ] - - [ b ]", Some("dashed")), ("[ a ] = = [ b ]", Some("double-dash")),
    ];
    for (src, want) in cases {
        let g = text::parse(src).unwrap_or_else(|e| panic!("{src:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator in {src:?}");
        assert!(!g.edges[0].directed, "{src:?} is arrow-less (undirected)");
    }
}

/// Single-character arrow-less patterns are parse errors.
fn plain_singles_are_errors() {
    for src in ["[ a ] . [ b ]\n", "[ a ] = [ b ]\n", "[ a ] - [ b ]\n", "[ a ] ~ [ b ]\n"] {
        let err = text::parse(src).expect_err("single plain unit must error");
        assert!(err.line >= 1);
    }
}

/// ge.text_parser.p10 (c10): attribute-value unquoting — the upstream two-layer
/// composite, pinned byte-for-byte by tests/repro/claims/attr-quote-value.observed.
#[test]
fn p10() {
    // headline: single-quoted values lose their quotes (the bug report)
    let g = text::parse("[ a ] { label: 'hello'; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("hello"));
    quote_pairs_strip();
    store_unescape_set();
    percent_entity_decode();
    quote_split_rules();
    names_and_labels_are_parser_layer_only();
}

/// Double-, single-, mixed-end, greedy, and mid-value quote handling.
fn quote_pairs_strip() {
    let g = text::parse("[ a ] { label: \"hello world\"; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("hello world"));
    // mixed ends strip (greedy first-and-last, either quote char)
    let g = text::parse("[ a ] { label: \"mixed'; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("mixed"), "mismatched ends strip");
    // greedy: only the first and last quote go
    let g = text::parse("[ a ] { label: \"a\" \"b\"; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a\" \"b"));
    // mid-value quotes are kept verbatim
    let g = text::parse("[ a ] { label: x\"y\"z; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("x\"y\"z"));
}

/// The store-layer unescape set: `\"`, `\'`, `\;`, `\\` — and the
/// unterminated-quote and empty-value forms.
fn store_unescape_set() {
    let g = text::parse("[ a ] { label: \"a\\\"b\"; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a\"b"), "escaped quote inside quotes");
    let g = text::parse("[ a ] { label: a\\'b; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a'b"));
    let g = text::parse("[ a ] { label: a\\;b; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a;b"));
    let g = text::parse("[ a ] { label: a\\\\b; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a\\b"));
    // unterminated quote: accepted verbatim, `;` still terminates
    let g = text::parse("[ a ] { label: \"abc; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("\"abc"));
    // empty quoted value
    let g = text::parse("[ a ] { label: \"\"; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some(""));
}

/// The store-layer %XX layer (vs the oracle's unquote_attribute): printable
/// band decodes, exploit band strips, bare `%` with no hex-pair tail stays.
fn percent_entity_decode() {
    let cases = [
        ("%41", "A"), ("a%41b", "aAb"), ("%20", " "), ("%2B", "+"),
        ("%7f", ""), ("%00", ""), ("%9f", ""), ("%A1", ""), ("%1A", ""),
        ("100%", "100%"), ("%zz", "%zz"), ("%7F", "\u{7f}"),
    ];
    for (src, want) in cases {
        let text = format!("[ a ] {{ label: {src}; }} --> [ b ]\n");
        let g = text::parse(&text).unwrap_or_else(|e| panic!("{src:?} must parse: {e}"));
        assert_eq!(g.get_attr(Scope::Node(0), "label"), Some(want), "value {src:?}");
    }
}

/// The quoted-branch terminator: a closing quote must be followed by `;`
/// or the end of the block — otherwise the unquoted branch takes the
/// whole run; a properly closed value shields `;` from splitting.
fn quote_split_rules() {
    let g = text::parse("[ a ] { label: \"a\" x; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("\"a\" x"), "no early quote close");
    let g = text::parse("[ a ] { label: \"x; y\"; color: red; } --> [ b ]\n").expect("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("x; y"));
    assert_eq!(g.get_attr(Scope::Node(0), "color"), Some("red"));
}

/// Node names and edge labels are parser-layer only — no store unquoting.
fn names_and_labels_are_parser_layer_only() {
    let g = text::parse("[ \"n\" ] -- \"l\" --> [ b ]\n").expect("must parse");
    assert_eq!(g.nodes[node(&g, "\"n\"")].name, "\"n\"", "names keep their quotes");
    assert_eq!(g.get_attr(Scope::Edge(0), "label"), Some("\"l\""));
}
