//! Purpose: the ASCII renderer's glyph tables and shape resolution
//! (ge.ascii_render.c1/c2/c5/c6) — ported from the pinned oracle's
//! `As_ascii.pm` (`$border_styles` first row) and pinned against observed
//! bytes in tests/repro/claims/ascii-render-tables.observed (gently-0cq).
//! Responsibilities: resolve a node's border style to its corner/edge/
//! side glyphs, an edge's style to its 3-cell arrow run, a node's shape
//! to its render class, and text to display width (double-width glyphs
//! count two columns).

/// The glyph set of one node border style (ASCII row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Border {
    pub tl: char,
    pub tr: char,
    pub br: char,
    pub bl: char,
    /// Horizontal edge pattern (repeated across the interior).
    pub h: &'static str,
    pub side: char,
}

/// The ASCII border-style table: per style, corners, horizontal
/// pattern, and side glyph. Unknown styles render solid (upstream
/// falls back the same way for unrecognized style strings).
pub(super) fn border(style: &str) -> Border {
    match style {
        "dotted" => Border { tl: '.', tr: '.', br: ':', bl: ':', h: ".", side: ':' },
        "dashed" => Border { tl: '+', tr: '+', br: '+', bl: '+', h: "- ", side: '\'' },
        "dot-dash" => Border { tl: '+', tr: '+', br: '+', bl: '+', h: ".-", side: '!' },
        "dot-dot-dash" => Border { tl: '+', tr: '+', br: '+', bl: '+', h: "..-", side: '|' },
        "double" => Border { tl: '#', tr: '#', br: '#', bl: '#', h: "=", side: 'H' },
        "double-dash" => Border { tl: '#', tr: '#', br: '#', bl: '#', h: "= ", side: '"' },
        "wave" => Border { tl: '+', tr: '+', br: '+', bl: '+', h: "~", side: '{' },
        "bold" | "broad" | "wide" => Border { tl: '#', tr: '#', br: '#', bl: '#', h: "#", side: '#' },
        "none" => Border { tl: ' ', tr: ' ', br: ' ', bl: ' ', h: " ", side: ' ' },
        _ => Border { tl: '+', tr: '+', br: '+', bl: '+', h: "-", side: '|' },
    }
}

/// The interior horizontal run of a border: the pattern shifted by one
/// position (oracle: dashed `+ - +`, dot-dash `+-.-+`, dot-dot-dash
/// `+.-.+`, double-dash `# = #` — all pattern[(i+1) mod len]).
pub(super) fn h_run(b: &Border, inner: usize) -> String {
    let pat = b.h.as_bytes();
    let pat_len = pat.len();
    (0..inner)
        .map(|i| pat[(i + 1) % pat_len] as char)
        .collect()
}

/// How a node shape renders (ge.ascii_render.c6): the probed oracle
/// collapses every non-special shape to the plain box for small nodes —
/// `circle`/`ellipse`/`diamond`/… all render `+---+`-style; `rounded`
/// blanks the corners; `point` blanks the borders and swaps the label
/// for a centered `*`; `invisible` draws nothing. (`box` is NOT a valid
/// upstream shape — the default is `rect`; a `shape: box` attribute is
/// upstream-rejected, recorded for gently-dcp.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shape {
    Box,
    Rounded,
    Point,
    Invisible,
}

/// Resolve a node's `shape` attribute to its render class.
pub(super) fn shape(attr: Option<&str>) -> Shape {
    match attr {
        Some("rounded") => Shape::Rounded,
        Some("point") => Shape::Point,
        Some("invisible") => Shape::Invisible,
        _ => Shape::Box,
    }
}

/// The 3-cell arrow run of an edge style (ge.ascii_render.c2 — observed
/// bytes: solid `-->`/`---`, double `==>`/`===`, dotted `..>`/`...`,
/// wave `~~>`/`~~~`, dashed `- >`/single `-`, bold `##>`/`###`).
pub(super) fn edge_run(style: &str, arrowed: bool) -> String {
    match (style, arrowed) {
        ("dashed", true) => "- >".to_string(),
        ("double", true) => "==>".to_string(),
        ("dotted", true) => "..>".to_string(),
        ("wave", true) => "~~>".to_string(),
        ("bold", true) => "##>".to_string(),
        (s, true) => {
            let _ = s;
            "-->".to_string()
        }
        ("dashed", false) => "-".to_string(),
        ("double", false) => "===".to_string(),
        ("dotted", false) => "...".to_string(),
        ("wave", false) => "~~~".to_string(),
        ("bold", false) => "###".to_string(),
        (_, false) => "---".to_string(),
    }
}

/// The repeating edge-line fill pattern of a style (the run minus its
/// head char, used for labelled/wider runs).
pub(super) fn edge_fill(style: &str) -> String {
    match style {
        "double" => "==".to_string(),
        "dotted" => "..".to_string(),
        "wave" => "~~".to_string(),
        "dashed" => "- ".to_string(),
        "bold" => "##".to_string(),
        _ => "--".to_string(),
    }
}

/// Display width of `s`: double-width glyphs (East Asian Wide/Fullwidth)
/// count two columns, everything else one — alignment never follows byte
/// length (ge.ascii_render.c5; probed: 中 renders a 5-wide box). Shared
/// with the boxart renderer, whose boxes and labels follow the same rule.
pub(in crate::render) fn display_width(s: &str) -> usize {
    s.chars().map(|c| if is_wide(c) { 2 } else { 1 }).sum()
}

/// East Asian Wide/Fullwidth ranges (the double-width classes the
/// display-width contract covers).
fn is_wide(c: char) -> bool {
    let cp = c as u32;
    matches!(
        cp,
        0x1100..=0x115F
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE19
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x20000..=0x3FFFD
    )
}

#[cfg(test)]
mod tests {
    use super::{border, display_width, edge_run, h_run, shape, Shape};

    #[test]
    fn solid_border_glyphs() {
        let b = border("solid");
        assert_eq!(h_run(&b, 3), "---");
        assert_eq!(b.side, '|');
    }

    #[test]
    fn dotted_corners_differ_top_bottom() {
        let b = border("dotted");
        assert_eq!((b.tl, b.tr, b.br, b.bl), ('.', '.', ':', ':'));
        assert_eq!(h_run(&b, 3), "...");
    }

    #[test]
    fn shifted_h_runs_match_oracle() {
        assert_eq!(h_run(&border("dashed"), 3), " - ");
        assert_eq!(h_run(&border("dot-dash"), 3), "-.-");
        assert_eq!(h_run(&border("dot-dot-dash"), 3), ".-.");
        assert_eq!(h_run(&border("double-dash"), 3), " = ");
    }

    #[test]
    fn edge_runs_match_oracle_table() {
        for (style, arrowed, want) in [
            ("solid", true, "-->"),
            ("solid", false, "---"),
            ("double", true, "==>"),
            ("double", false, "==="),
            ("dotted", true, "..>"),
            ("dotted", false, "..."),
            ("wave", true, "~~>"),
            ("wave", false, "~~~"),
            ("dashed", true, "- >"),
            ("dashed", false, "-"),
            ("bold", true, "##>"),
            ("bold", false, "###"),
        ] {
            assert_eq!(edge_run(style, arrowed), want, "{style} arrowed={arrowed}");
        }
    }

    #[test]
    fn shapes_resolve_to_render_classes() {
        assert_eq!(shape(None), Shape::Box);
        assert_eq!(shape(Some("rect")), Shape::Box);
        assert_eq!(shape(Some("circle")), Shape::Box);
        assert_eq!(shape(Some("rounded")), Shape::Rounded);
        assert_eq!(shape(Some("point")), Shape::Point);
        assert_eq!(shape(Some("invisible")), Shape::Invisible);
    }

    #[test]
    fn display_width_counts_wide_glyphs_double() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("中"), 2);
        assert_eq!(display_width("a中b"), 4);
    }
}
