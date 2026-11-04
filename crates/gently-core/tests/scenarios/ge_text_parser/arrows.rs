use super::*;

/// Every directed operator maps to its documented style (solid edges carry
/// no style attribute — solid is the default).
pub(crate) fn directed_ops_map_to_styles() {
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
pub(crate) fn bidirectional_ops_mark_both_ends() {
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
        assert!(
            e.directed && e.arrows.start && e.arrows.end,
            "{op:?} bidirectional"
        );
    }
}

/// A lone `<`, a missing left node and a missing right node are errors.
pub(crate) fn broken_edges_are_errors() {
    for src in ["[ a ] <--\n", "--> [ b ]\n", "[ a ] -->\n"] {
        let err = text::parse(src).expect_err("must be a parse error");
        assert!(err.line >= 1, "{src:?} must name a line");
    }
}

/// bold/wide/broad have no operator spelling; `{ style: X; }` works.
pub(crate) fn style_only_styles_come_via_the_attribute() {
    let g = text::parse("[ a ] --> { style: bold; } [ b ]\n").must("must parse");
    assert_eq!(style(&g, 0), Some(String::from("bold")));
    for src in ["[ a ] bold [ b ]\n", "[ a ] wide [ b ]\n"] {
        assert!(
            text::parse(src).is_err(),
            "{src:?} has no operator spelling"
        );
    }
}

/// Directed: mixed/repeated units accepted, style = last unit.
pub(crate) fn directed_style_follows_the_last_unit() {
    let cases = [
        ("..-..-..>", Some("dotted")),
        (".-..-..>", Some("dotted")),
        (".--->", None),
        ("--.>", Some("dotted")),
        (".->", Some("dot-dash")),
        ("- >", Some("dashed")),
        ("= >", Some("double-dash")),
    ];
    for (op, want) in cases {
        let src = format!("[ a ] {op} [ b ]\n");
        let g = text::parse(&src).unwrap_or_else(|e| panic!("{op:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator {op:?}");
    }
}

/// Arrow-less: single `.-`/`..-` valid; mixed dot-units inherit (solid).
pub(crate) fn arrowless_dot_singles_are_valid() {
    let cases = [
        (".-", Some("dot-dash")),
        ("..-", Some("dot-dot-dash")),
        (".-.-", Some("dot-dash")),
        ("..", Some("dotted")),
        ("..-.-", None),
    ];
    for (op, want) in cases {
        let src = format!("[ a ] {op} [ b ]\n");
        let g = text::parse(&src).unwrap_or_else(|e| panic!("{op:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator {op:?}");
        assert!(!g.edges[0].directed, "{op:?} is arrow-less (undirected)");
    }
}

/// Arrow-less plain units: homogeneous reps name the style; mixed inherit.
pub(crate) fn arrowless_plain_units() {
    let cases = [
        ("[ a ] == [ b ]", None),
        ("[ a ] ==[ b ]", Some("double")),
        ("[ a ] ~~ [ b ]", Some("wave")),
        ("[ a ] -- [ b ]", None),
        ("[ a ] - -[ b ]", Some("dot-dot-dash")),
        ("[ a ] - - [ b ]", Some("dashed")),
        ("[ a ] = = [ b ]", Some("double-dash")),
    ];
    for (src, want) in cases {
        let g = text::parse(src).unwrap_or_else(|e| panic!("{src:?} must parse: {e}"));
        assert_eq!(style(&g, 0), want.map(String::from), "operator in {src:?}");
        assert!(!g.edges[0].directed, "{src:?} is arrow-less (undirected)");
    }
}

/// Single-character arrow-less patterns are parse errors.
pub(crate) fn plain_singles_are_errors() {
    for src in [
        "[ a ] . [ b ]\n",
        "[ a ] = [ b ]\n",
        "[ a ] - [ b ]\n",
        "[ a ] ~ [ b ]\n",
    ] {
        let err = text::parse(src).expect_err("single plain unit must error");
        assert!(err.line >= 1);
    }
}
