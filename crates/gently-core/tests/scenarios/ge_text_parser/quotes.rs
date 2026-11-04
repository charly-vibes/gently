use super::*;

/// Double-, single-, mixed-end, greedy, and mid-value quote handling.
pub(crate) fn quote_pairs_strip() {
    let g = text::parse("[ a ] { label: \"hello world\"; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("hello world"));
    // mixed ends strip (greedy first-and-last, either quote char)
    let g = text::parse("[ a ] { label: \"mixed'; } --> [ b ]\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Node(0), "label"),
        Some("mixed"),
        "mismatched ends strip"
    );
    // greedy: only the first and last quote go
    let g = text::parse("[ a ] { label: \"a\" \"b\"; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a\" \"b"));
    // mid-value quotes are kept verbatim
    let g = text::parse("[ a ] { label: x\"y\"z; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("x\"y\"z"));
}

/// The store-layer unescape set: `\"`, `\'`, `\;`, `\\` — and the
/// unterminated-quote and empty-value forms.
pub(crate) fn store_unescape_set() {
    let g = text::parse("[ a ] { label: \"a\\\"b\"; } --> [ b ]\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Node(0), "label"),
        Some("a\"b"),
        "escaped quote inside quotes"
    );
    let g = text::parse("[ a ] { label: a\\'b; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a'b"));
    let g = text::parse("[ a ] { label: a\\;b; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a;b"));
    let g = text::parse("[ a ] { label: a\\\\b; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("a\\b"));
    // unterminated quote: accepted verbatim, `;` still terminates
    let g = text::parse("[ a ] { label: \"abc; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("\"abc"));
    // empty quoted value
    let g = text::parse("[ a ] { label: \"\"; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some(""));
}

/// The store-layer %XX layer (vs the oracle's unquote_attribute): printable
/// band decodes, exploit band strips, bare `%` with no hex-pair tail stays.
pub(crate) fn percent_entity_decode() {
    let cases = [
        ("%41", "A"),
        ("a%41b", "aAb"),
        ("%20", " "),
        ("%2B", "+"),
        ("%7f", ""),
        ("%00", ""),
        ("%9f", ""),
        ("%A1", ""),
        ("%1A", ""),
        ("100%", "100%"),
        ("%zz", "%zz"),
        ("%7F", "\u{7f}"),
    ];
    for (src, want) in cases {
        let text = format!("[ a ] {{ label: {src}; }} --> [ b ]\n");
        let g = text::parse(&text).unwrap_or_else(|e| panic!("{src:?} must parse: {e}"));
        assert_eq!(
            g.get_attr(Scope::Node(0), "label"),
            Some(want),
            "value {src:?}"
        );
    }
}

/// The quoted-branch terminator: a closing quote must be followed by `;`
/// or the end of the block — otherwise the unquoted branch takes the
/// whole run; a properly closed value shields `;` from splitting.
pub(crate) fn quote_split_rules() {
    let g = text::parse("[ a ] { label: \"a\" x; } --> [ b ]\n").must("must parse");
    assert_eq!(
        g.get_attr(Scope::Node(0), "label"),
        Some("\"a\" x"),
        "no early quote close"
    );
    let g = text::parse("[ a ] { label: \"x; y\"; color: red; } --> [ b ]\n").must("must parse");
    assert_eq!(g.get_attr(Scope::Node(0), "label"), Some("x; y"));
    assert_eq!(g.get_attr(Scope::Node(0), "color"), Some("red"));
}
