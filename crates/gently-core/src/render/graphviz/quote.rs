//! Purpose: DOT identifier/value quoting — the safely-quoted names of
//! ge.graphviz_render.c1.
//! Responsibilities: decide bare vs quoted form for a DOT identifier and
//! escape backslashes and quotes inside quoted values exactly once (the
//! ge.dot_parser contract unescapes exactly once, so the round trip c4
//! holds).
//! Rationale: dot accepts bare identifiers matching `[A-Za-z_][A-Za-z_0-9]*`
//! (numerals included); everything else — spaces, unicode, embedded
//! quotes, the `#N` anonymous ids — needs quoting.

/// Emit `value` as a DOT identifier: bare when it is a plain identifier
/// or numeral, otherwise quoted with `\` and `"` escaped.
pub fn quote(value: &str) -> String {
    let bare = !value.is_empty()
        && value.chars().enumerate().all(|(i, c)| {
            c.is_ascii_alphanumeric() && (i > 0 || !c.is_ascii_digit())
                || c == '_'
                || c == '.'
        });
    if bare {
        return value.to_string();
    }
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '\\' | '"' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::quote;

    #[test]
    fn bare_identifiers_stay_bare() {
        assert_eq!(quote("a"), "a");
        assert_eq!(quote("box"), "box");
        assert_eq!(quote("#000000"), "\"#000000\"", "leading # quotes");
        assert_eq!(quote("2"), "\"2\"", "numeral quotes (leading digit)");
        assert_eq!(quote(""), "\"\"", "empty quotes");
    }

    #[test]
    fn specials_are_escaped_once() {
        assert_eq!(quote("a b"), "\"a b\"");
        assert_eq!(quote("q\"x"), "\"q\\\"x\"");
        assert_eq!(quote("b\\c"), "\"b\\\\c\"");
        assert_eq!(quote("üñî"), "\"üñî\"");
    }
}