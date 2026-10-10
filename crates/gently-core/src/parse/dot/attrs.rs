//! Purpose: value-unquoting for the ge.dot_parser grammar (gently-89p).
//! Responsibilities: unescape a raw quoted-string content exactly once —
//! `\"` becomes `"`, `\\` becomes `\`, any other escape keeps its
//! backslash — so the model's attribute tables receive the final verbatim
//! value with no second unquote pass (the r22 store-layer finding: the
//! model stores what it is given, the parser owns unescaping). Rationale:
//! isolated here so the once-only rule has one auditable home, mirroring
//! the `text/` module's attrs split.

/// Unquote raw quoted-string content: escapes resolved exactly once.
pub(super) fn unquote(raw: &str) -> String {
    let mut out = String::new();
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::unquote;

    #[test]
    fn unquotes_exactly_once() {
        assert_eq!(unquote("x \\\"y\\\\"), "x \"y\\");
        assert_eq!(unquote("plain"), "plain");
        assert_eq!(unquote("a\\nb"), "a\\nb", "unknown escapes keep the backslash");
        assert_eq!(unquote("trailing\\"), "trailing\\");
    }
}
