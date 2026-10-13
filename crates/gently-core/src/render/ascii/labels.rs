//! Purpose: label splitting and alignment for the ascii renderer — the
//! port of upstream `Node::_aligned_label` (gently-527, gently-y7i).
//! Responsibilities: split a label into trimmed lines at the `\n` `\r`
//! `\l` `\c` terminators, unescape `\|` and `\\` inside each part, give
//! every line its alignment (the PREVIOUS terminator's — the escape at
//! the end of one segment aligns the NEXT line; the initial alignment is
//! the object's `align` attribute, node default `center`, edge default
//! `left`), and measure the longest line in UTF-8 bytes (the oracle's
//! `length()` — gently-bvb).
//! Rationale: upstream splits labels once, in `_aligned_label`, and both
//! node cells (`_draw_label`) and edge label cells (`_insert_label`)
//! place the resulting (line, alignment) rows through `_printfb_aligned`
//! — gently mirrors that shape so the two surfaces can't diverge. Text
//! WRAPPING (`textwrap: auto`/width, upstream `_wrapped_label`) is NOT
//! here — that is gently-emd's surface and stays out of this slice.

use super::styles::byte_width;
use crate::graph::{Graph, ObjectKind, Scope};
use crate::render::ascii::geometry::object_attr;

/// The per-line horizontal alignment of a label row (upstream `l`/`r`/`c`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Align {
    Left,
    Center,
    Right,
}

/// One label line plus its alignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LabelLine {
    pub text: String,
    pub align: Align,
}

/// The alignment a node/edge label defaults to: nodes center, edges
/// left (upstream Attributes.pm: `align` defaults `center` for nodes,
/// `left` for edges). An object's own `align` attribute wins.
pub(super) fn default_align(graph: &Graph, scope: Scope, kind: ObjectKind) -> Align {
    let default = match kind {
        ObjectKind::Node => Align::Center,
        _ => Align::Left,
    };
    align_of(object_attr(graph, scope.clone(), kind, "align"), default)
}

/// The alignment named by an `align` attribute value: its first letter
/// (`l`/`r`/`c`), falling back to `default` for anything else.
pub(super) fn align_of(attr: Option<&str>, default: Align) -> Align {
    match attr.and_then(|a| a.chars().next()) {
        Some('l') => Align::Left,
        Some('r') => Align::Right,
        Some('c') => Align::Center,
        _ => default,
    }
}

/// Split `label` the way upstream `_aligned_label` does: each `\n` `\r`
/// `\l` `\c` terminator ENDS a segment; the segment is unescaped (`\|`
/// → `|`, `\\` → `\`) and trimmed; the segment's alignment is the
/// PREVIOUS terminator's (initially `default`), and a `\n` terminator
/// means "the default" — so `foo\l bar` renders `foo` in the default
/// alignment and `bar` left. A trailing terminator adds no line (the
/// loop stops at the empty remainder).
pub(super) fn aligned_label(label: &str, default: Align) -> Vec<LabelLine> {
    let mut lines = Vec::new();
    let mut last_align = default;
    let mut part = String::new();
    let mut chars = label.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            part.push(c);
            continue;
        }
        match chars.next() {
            // the escape ends this segment and aligns the next one;
            // \n means "the object's default" (upstream: `$a = $al if $a eq 'n'`)
            Some(esc @ ('n' | 'r' | 'l' | 'c')) => {
                lines.push(LabelLine { text: finish(part), align: last_align });
                part = String::new();
                last_align = match esc {
                    'l' => Align::Left,
                    'r' => Align::Right,
                    'c' => Align::Center,
                    _ => default,
                };
            }
            Some('\\') => part.push('\\'),
            Some(other) => {
                part.push('\\');
                part.push(other);
            }
            None => part.push('\\'),
        }
    }
    // the remainder after the last terminator — never for an empty
    // remainder (upstream: `push @lines, $label if $label ne ''`), and
    // an escape-free empty label yields NO lines at all
    if !part.is_empty() {
        lines.push(LabelLine { text: finish(part), align: last_align });
    }
    lines
}

/// Unescape (`\|` → `|`, `\\` → `\`) and trim one label part
/// (upstream's per-part `s///` chain).
fn finish(part: String) -> String {
    let unescaped = part.replace("\\|", "|").replace("\\\\", "\\");
    unescaped.trim().to_string()
}

/// The longest line's width in UTF-8 bytes — the cell-width rule
/// upstream `dimensions()` feeds `_correct_size` with.
pub(super) fn max_line_width(lines: &[LabelLine]) -> usize {
    lines.iter().map(|l| byte_width(&l.text)).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_newline_and_trims_parts() {
        let lines = aligned_label("put msg\\n via http", Align::Left);
        assert_eq!(
            lines,
            vec![
                LabelLine { text: "put msg".into(), align: Align::Left },
                LabelLine { text: "via http".into(), align: Align::Left },
            ]
        );
    }

    #[test]
    fn escape_aligns_the_following_line() {
        // \l terminates "Long Node Label" and left-aligns "left"
        let lines = aligned_label("Long Node Label\\l left\\r right\\c center", Align::Center);
        let aligns: Vec<Align> = lines.iter().map(|l| l.align).collect();
        assert_eq!(aligns, vec![Align::Center, Align::Left, Align::Right, Align::Center]);
        let texts: Vec<&str> = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, vec!["Long Node Label", "left", "right", "center"]);
    }

    #[test]
    fn newline_terminator_falls_back_to_the_default() {
        // \n means "default alignment" for the next line — the \r before
        // it does NOT carry past it
        let lines = aligned_label("A\\r long\\n edge label", Align::Left);
        let aligns: Vec<Align> = lines.iter().map(|l| l.align).collect();
        assert_eq!(aligns, vec![Align::Left, Align::Right, Align::Left]);
    }

    #[test]
    fn trailing_terminator_adds_no_line() {
        assert_eq!(aligned_label("a\\n", Align::Center).len(), 1);
    }

    #[test]
    fn empty_middle_lines_are_kept() {
        let lines = aligned_label("a\\n\\nb", Align::Center);
        let texts: Vec<&str> = lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, vec!["a", "", "b"]);
    }

    #[test]
    fn escaped_chars_pass_through() {
        let lines = aligned_label("a\\\\b\\|c", Align::Left);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "a\\b|c");
    }

    #[test]
    fn max_line_width_counts_bytes() {
        let lines = aligned_label("Köln", Align::Left);
        assert_eq!(max_line_width(&lines), 5);
    }
}
