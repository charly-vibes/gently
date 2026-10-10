//! Purpose: record and HTML-like label autosplit for the ge.dot_parser
//! grammar (gently-89p, c5/p5). Responsibilities: the flat record split —
//! a `shape=record` label with a vertical bar becomes `name.N` part nodes
//! with braces escaped `\{`/`\}` per the oracle and the original renamed
//! to the label text plus a `basename` attribute — and the HTML-like table
//! split (one part per `<TD>` cell, original renamed to the raw label,
//! no `basename`); bare text outside a `<TD>` cell is malformed and fails
//! tokenization-style ("not recognized"). Rationale: the probe evidence in
//! tests/repro/claims/dot-records-ports.observed pins these shapes against
//! Graph::Easy v0.69 @ ededa3d7 — the flat split (not Graphviz's nesting)
//! is the oracle's own surviving behavior, so it is kept deliberately.

/// The record label with braces escaped, if any (oracle escaping).
pub(super) fn escape_record_braces(label: &str) -> String {
    label.replace('{', "\\{").replace('}', "\\}")
}

/// The `<TD>` cell texts of an HTML-like table label, validating tag-only
/// structure: bare text is legal only directly after an opening `<TD>`
/// tag; anything else is malformed. Fails unit-style — the caller wraps
/// the oracle's "not recognized" message around the raw label.
pub(super) fn html_parts(inner: &str) -> Result<Vec<String>, ()> {
    let chars: Vec<char> = inner.chars().collect();
    let mut parts = Vec::new();
    let mut i = 0usize;
    let mut open_td = false;
    while i < chars.len() {
        if chars[i] == '<' {
            i = tag_end(&chars, i)?;
            open_td = td_tag(&chars, i);
            continue;
        }
        let end = chars[i..].iter().position(|&c| c == '<').unwrap_or(chars.len());
        let text: String = chars[i..i + end].iter().collect();
        i += end;
        open_td = push_text(&mut parts, &text, open_td)?;
    }
    Ok(parts)
}

/// Consume one `<...>` tag through its `>`; malformed (no `>`) fails.
fn tag_end(chars: &[char], i: usize) -> Result<usize, ()> {
    let close = chars[i..].iter().position(|&c| c == '>').ok_or(())?;
    Ok(i + close + 1)
}

/// Whether the tag ending just before `i` was an opening `<TD ...>`.
fn td_tag(chars: &[char], i: usize) -> bool {
    // walk back to the tag's '<'
    let open = chars[..i].iter().rposition(|&c| c == '<').unwrap_or(i);
    let tag: String = chars[open + 1..i - 1].iter().collect();
    tag.len() >= 2 && &tag[..2] == "TD"
}

/// Push a text run: whitespace-only text is ignored; bare text outside a
/// `<TD>` cell is malformed. Returns the new TD state.
fn push_text(parts: &mut Vec<String>, text: &str, open_td: bool) -> Result<bool, ()> {
    if text.trim().is_empty() {
        return Ok(open_td);
    }
    if !open_td {
        return Err(());
    }
    parts.push(text.trim().to_string());
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::{escape_record_braces, html_parts};

    #[test]
    fn record_braces_escape() {
        assert_eq!(escape_record_braces("A|{B|C}"), "A|\\{B|C\\}");
    }

    #[test]
    fn html_parts_of_table() {
        let parts = html_parts("<TABLE><TR><TD>one</TD><TD>two</TD></TR></TABLE>").expect("well-formed");
        assert_eq!(parts, vec!["one", "two"]);
    }

    #[test]
    fn bare_text_outside_td_is_malformed() {
        assert!(html_parts("<table>...</table>").is_err());
        assert!(html_parts("<TABLE><TR><TD>ok</TD></TR></TABLE>").is_ok());
    }
}
