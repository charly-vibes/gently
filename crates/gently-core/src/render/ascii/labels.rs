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
//! — gently mirrors that shape so the two surfaces can't diverge. When a
//! `textwrap` attribute is set (gently-emd), upstream `_aligned_label`
//! dispatches to `_wrapped_label` instead: wrapping REPLACES the escape
//! split (each `\n`/`\r`/`\l`/`\c` becomes a space, whitespace
//! collapses, lines break at word boundaries), so both surfaces wrap
//! through the same entry point here too.

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
                lines.push(LabelLine {
                    text: finish(part),
                    align: last_align,
                });
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
        lines.push(LabelLine {
            text: finish(part),
            align: last_align,
        });
    }
    lines
}

/// Unescape (`\|` → `|`, `\\` → `\`) and trim one label part
/// (upstream's per-part `s///` chain).
fn finish(part: String) -> String {
    let unescaped = part.replace("\\|", "|").replace("\\\\", "\\");
    unescaped.trim().to_string()
}

/// The textwrap mode of an object (upstream Attributes.pm `textwrap`:
/// object default `inherit`, graph default `none` — so the effective
/// mode is the object's own attribute, else its class's, else the
/// graph's). `auto` picks the width from the label's byte length; a
/// number wraps after that many characters; anything else (the `none`
/// default) leaves the escape-split surface alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextWrap {
    None,
    Auto,
    At(usize),
}

/// The effective textwrap mode of the object at `scope` (gently-emd):
/// per-object attribute over class attribute over the graph attribute
/// (upstream's `inherit` chain), falling back to `None`.
pub(super) fn textwrap_of(graph: &Graph, scope: Scope, kind: ObjectKind) -> TextWrap {
    let value = object_attr(graph, scope, kind, "textwrap")
        .or_else(|| graph.get_attr(Scope::Graph, "textwrap"));
    match value {
        Some("auto") => TextWrap::Auto,
        Some(v) if !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()) => {
            TextWrap::At(v.parse().unwrap_or(0))
        }
        _ => TextWrap::None,
    }
}

/// The label lines of the object at `scope`: the escape split when its
/// effective textwrap mode is `none` (the 527 surface), upstream
/// `_wrapped_label` otherwise (gently-emd).
pub(super) fn object_label_lines(
    graph: &Graph,
    scope: Scope,
    kind: ObjectKind,
    label: &str,
    default: Align,
) -> Vec<LabelLine> {
    match textwrap_of(graph, scope, kind) {
        TextWrap::None => aligned_label(label, default),
        wrap => wrapped_label(label, default, wrap),
    }
}

/// Upstream `Node::_wrapped_label` (gently-emd): wrapping REPLACES the
/// escape split — every `\n`/`\r`/`\l`/`\c` becomes a space and
/// whitespace collapses to single spaces — then the text wraps once a
/// line reaches `wrap` columns, at the LAST space (dropped) or the last
/// hyphen (kept on the line) whichever comes later; `auto` picks
/// `int(sqrt(byte_length) * 1.4)`, floored at 2. A line with no break
/// point never wraps (the overflow stays on one row, upstream's
/// `last_space != 0 || last_hyphen != 0` guard), and every row carries
/// the object's default alignment — the escapes' alignments are lost.
pub(super) fn wrapped_label(label: &str, default: Align, wrap: TextWrap) -> Vec<LabelLine> {
    let collapsed = collapse_spaces(flatten_escapes(label));
    let lines = wrap_lines(&collapsed, wrap_width(collapsed.len(), wrap));
    lines
        .into_iter()
        .map(|text| LabelLine {
            text,
            align: default,
        })
        .collect()
}

/// The wrap width of a mode (upstream `_wrapped_label`'s head): a
/// number is taken literally, `auto` picks `int(sqrt(byte_length) *
/// 1.4)`, and both are floored at 2.
fn wrap_width(byte_len: usize, wrap: TextWrap) -> usize {
    let w = match wrap {
        TextWrap::At(n) => n,
        TextWrap::Auto => ((byte_len as f64).sqrt() * 1.4).floor() as usize,
        TextWrap::None => unreachable!("wrap_width is only called for a set mode"),
    };
    w.max(2)
}

/// Every `\n`/`\r`/`\l`/`\c` escape becomes a space; other escapes pass
/// through (upstream `s/\\(n|r|l|c)/ /g`).
fn flatten_escapes(label: &str) -> String {
    let mut flattened = String::with_capacity(label.len());
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && matches!(chars.peek(), Some('n' | 'r' | 'l' | 'c')) {
            chars.next();
            flattened.push(' ');
        } else {
            flattened.push(c);
        }
    }
    flattened
}

/// Collapse whitespace runs to single spaces, no trimming (upstream
/// `s/\s+/ /g`).
fn collapse_spaces(text: String) -> String {
    let mut collapsed = String::with_capacity(text.len());
    let mut spaces = 0usize;
    for c in text.chars() {
        if c.is_whitespace() {
            spaces += 1;
            if spaces == 1 {
                collapsed.push(' ');
            }
        } else {
            spaces = 0;
            collapsed.push(c);
        }
    }
    collapsed
}

/// Wrap `text` once each line reaches `wrap_at` bytes, at the LAST
/// space (dropped) or last hyphen (kept on the line) whichever comes
/// later (upstream's loop, in bytes — the break points are ASCII, so
/// byte splits stay on char boundaries). A line with no break point
/// never wraps (the overflow stays on one row, upstream's
/// `last_space != 0 || last_hyphen != 0` guard); the leftover after the
/// last break is the final line.
fn wrap_lines(text: &str, wrap_at: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some((w, keep_hyphen)) = wrap_point(rest, wrap_at) {
        let mut line = rest[..w].to_string();
        if keep_hyphen {
            line.push('-');
        }
        lines.push(line);
        rest = &rest[w + 1..];
    }
    if !rest.is_empty() {
        lines.push(rest.to_string());
    }
    lines
}

/// The next wrap point of `text`: the byte offset whose line first
/// reaches `wrap_at`, split at the last space or hyphen (`true` = keep
/// the hyphen on the line) — `None` when no line break point exists.
fn wrap_point(text: &str, wrap_at: usize) -> Option<(usize, bool)> {
    let bytes = text.as_bytes();
    let (mut i, mut line_len, mut last_space, mut last_hyphen) = (0usize, 0usize, 0usize, 0usize);
    while i < bytes.len() {
        let c = bytes[i];
        if c == b' ' {
            last_space = i;
        }
        if c == b'-' {
            last_hyphen = i;
        }
        line_len += 1;
        if line_len >= wrap_at && (last_space != 0 || last_hyphen != 0) {
            return Some(if last_hyphen > last_space {
                (last_hyphen, true)
            } else {
                (last_space, false)
            });
        }
        i += 1;
    }
    None
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
                LabelLine {
                    text: "put msg".into(),
                    align: Align::Left
                },
                LabelLine {
                    text: "via http".into(),
                    align: Align::Left
                },
            ]
        );
    }

    #[test]
    fn escape_aligns_the_following_line() {
        // \l terminates "Long Node Label" and left-aligns "left"
        let lines = aligned_label("Long Node Label\\l left\\r right\\c center", Align::Center);
        let aligns: Vec<Align> = lines.iter().map(|l| l.align).collect();
        assert_eq!(
            aligns,
            vec![Align::Center, Align::Left, Align::Right, Align::Center]
        );
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
