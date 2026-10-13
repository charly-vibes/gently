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
/// for a centered `*`; `invisible` draws nothing; `edge` renders the
/// node as an edge cell — a full-width dash line on the band's middle
/// row with the label riding above it (upstream `Node::as_ascii` builds
/// an `EDGE_HOR | EDGE_LABEL_CELL` cell for it, gently-cbk). (`box` is
/// NOT a valid upstream shape — the default is `rect`; a `shape: box`
/// attribute is upstream-rejected, recorded for gently-dcp.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shape {
    Box,
    Rounded,
    Point,
    Invisible,
    Edge,
}

/// Resolve a node's `shape` attribute to its render class.
pub(super) fn shape(attr: Option<&str>) -> Shape {
    match attr {
        Some("rounded") => Shape::Rounded,
        Some("point") => Shape::Point,
        Some("invisible") => Shape::Invisible,
        Some("edge") => Shape::Edge,
        _ => Shape::Box,
    }
}

/// The style's repeating unit token (upstream As_ascii `$edge_styles`
/// row-0 patterns / Parser.pm `_edge_style` classes). `wide` and `broad`
/// carry rows byte-identical to `bold` ('##') — they alias it (gently-51d,
/// pinned by tests/repro/claims/pod-style-family.observed and
/// edge-style-wide-undirected.observed); upstream `_correct_size` does NOT
/// widen their gap (only dot-dot-dash).
pub(super) fn edge_pattern(style: &str) -> &'static str {
    match style {
        "double" => "==",
        "double-dash" => "= ",
        "dotted" => ".",
        "dot-dash" => ".-",
        "dot-dot-dash" => "..-",
        "dashed" => "- ",
        "wave" => "~",
        "bold" | "wide" | "broad" => "##",
        _ => "-",
    }
}

/// The style's glyph run over a straight short-edge cell of gap width `w`
/// whose left edge sits at absolute column `rx`, directed (`arrowed`) or
/// not — the upstream `Edge/Cell.pm _draw_hor` rule, pinned byte-for-byte
/// by tests/repro/claims/edge-style-short-runs.observed (gently-oy2):
/// fill the width with the pattern, undirected cells shift the fill by
/// `rx % len` (upstream's ofs removal — directed SHORT cells skip it),
/// truncate to `w`, then drop one char per START_W and START_E/END_E
/// flag (both always present on a short cell); a directed run REPLACES
/// the remaining last char with the arrow head (upstream END_E: chop,
/// then `substr($line,-1,1) = arrow`).
pub(super) fn edge_run_at(style: &str, arrowed: bool, w: usize, rx: usize) -> String {
    let pat = edge_pattern(style);
    let len = pat.len();
    let mut line = pat.repeat(2 + w / len);
    if !arrowed {
        let ofs = rx % len;
        if ofs != 0 {
            line.replace_range(..ofs, "");
        }
    }
    line.truncate(w);
    line.pop(); // START_W: the line starts one cell column in (x = 1)
    line.pop(); // START_E (undirected) / END_E chop (directed)
    if arrowed {
        line.pop(); // END_E replace: the char the head takes over
        line.push('>');
    }
    line
}

/// The straight short-edge run of a BIDIRECTIONAL edge over a gap cell of
/// width `w` whose left edge sits at absolute column `rx` — the upstream
/// `_draw_hor` rule for an EDGE_SHORT_BD_EW cell (gently-7qz, pinned by
/// tests/repro/claims/pod-bidir.observed and pod-bidir-label-straight
/// .observed): the undirected ofs shift applies (the BD type is neither
/// EDGE_SHORT_E nor EDGE_SHORT_W), then END_E chops the last char and
/// replaces the new last char with `>`, and END_W replaces the first two
/// chars with ` <` — the run is `w-1` chars drawn at cell-x 0: ` <` + fill
/// plus a closing `>`. Upstream `_correct_size` widens the BD cell by one
/// column (column_geometry applies that; both arrowheads point outward).
pub(super) fn bd_run_at(style: &str, w: usize, rx: usize) -> String {
    let pat = edge_pattern(style);
    let len = pat.len();
    let mut line = pat.repeat(2 + w / len);
    let ofs = rx % len;
    if ofs != 0 {
        line.replace_range(..ofs, "");
    }
    line.truncate(w);
    line.pop(); // END_E chop
    line.pop(); // END_E: the char the head takes over
    line.push('>');
    line.replace_range(..2, " <"); // END_W: substr(0,2) = ' <'
    line
}

/// The minimum straight-edge gap width of a style: upstream's default 5
/// cell columns, widened by one for dot-dot-dash (`_correct_size`'s
/// `w++ if $style eq 'dot-dot-dash'` — the ` ..-> ` gap; probed).
pub(super) fn edge_gap_width(style: &str) -> usize {
    if style == "dot-dot-dash" {
        6
    } else {
        5
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
        "bold" | "wide" | "broad" => "##".to_string(),
        "dot-dash" => ".-".to_string(),
        "dot-dot-dash" => "..-".to_string(),
        "double-dash" => "= ".to_string(),
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

/// Width of `s` in UTF-8 BYTES — the ascii renderer's label arithmetic
/// follows upstream's byte-string `length()` (gently-bvb: the oracle
/// draws `[ Köln ]` as a 7-dash box because `Köln` is 5 bytes; probed
/// byte rule, re-derived from the display-width letter — gently-527).
/// The boxart and html renderers keep `display_width` (their contracts
/// pin display-width columns).
pub(in crate::render) fn byte_width(s: &str) -> usize {
    s.len()
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
    use super::{border, display_width, edge_run_at, h_run, shape, Shape};

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

    /// The full straight-edge run rule (gently-oy2): upstream
    /// `_draw_hor` pinned by tests/repro/claims/edge-style-short-runs
    /// .observed — arrowed runs at the minimum gap, undirected runs
    /// shifted by the cell's absolute x (ofs removal), dot-dot-dash's
    /// 6-column minimum gap, and the position-dependent undirected
    /// dot-dot-dash variants (chain gap 2/3 = x 15/25).
    #[test]
    fn edge_runs_at_match_oracle_rule() {
        for (style, arrowed, w, rx, want) in [
            // arrowed: no ofs shift, run = w-2 with the head
            ("dot-dash", true, 5, 5, ".->"),
            ("dot-dot-dash", true, 6, 5, "..->"),
            ("double-dash", true, 5, 5, "= >"),
            // undirected: ofs removal by rx % len, no head
            ("dot-dash", false, 5, 5, "-.-"),
            ("dot-dot-dash", false, 6, 5, "-..-"),
            ("double-dash", false, 5, 5, " = "),
            // position-dependence of the undirected shift (chain gaps)
            ("dot-dot-dash", false, 6, 15, "..-."),
            ("dot-dot-dash", false, 6, 25, ".-.."),
            // pre-existing styles obey the same rule at rx=5
            ("solid", true, 5, 5, "-->"),
            ("solid", false, 5, 5, "---"),
            ("dashed", true, 5, 5, "- >"),
            ("dashed", false, 5, 5, " - "),
            ("dotted", true, 5, 5, "..>"),
            ("dotted", false, 5, 5, "..."),
            ("double", true, 5, 5, "==>"),
            ("double", false, 5, 5, "==="),
        ] {
            assert_eq!(
                edge_run_at(style, arrowed, w, rx),
                want,
                "{style} arrowed={arrowed} w={w} rx={rx}"
            );
        }
    }

    #[test]
    fn edge_gap_width_widens_for_dot_dot_dash() {
        for (style, want) in [
            ("solid", 5),
            ("dashed", 5),
            ("dot-dash", 5),
            ("dot-dot-dash", 6),
            ("double-dash", 5),
        ] {
            assert_eq!(super::edge_gap_width(style), want, "{style} gap");
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
