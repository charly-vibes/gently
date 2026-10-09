//! Purpose: the boxart renderer's Unicode glyph tables (ge.boxart_render
//! c1/c3/c4) — ported from the pinned oracle's `As_ascii.pm`
//! `$border_styles[1]`, `$edge_styles[1]`, `$rounded_edges` and
//! `$point_shapes[1]`, and pinned against observed bytes in
//! tests/repro/claims/boxart-render-tables.observed (gently-css).
//! Responsibilities: resolve a node's border style to its corner/edge/
//! side glyphs (with the per-style rounded-shape overlay), an edge's
//! style to its horizontal pattern, vertical glyph and corner glyphs,
//! and text to display width (shared with the ascii renderer's rule).
//! Rationale: upstream renders boxart through the same code path as ascii
//! with tables selected by `_ascii_style`; the tables here are the
//! `_ascii_style = 1` column, byte-pinned by the probe round.

/// The glyph set of one node border style (Unicode row of the upstream
/// `$border_styles` table). The horizontal pattern is stored per top and
/// bottom border and the side glyphs per side: `broad` is the one style
/// whose four edges do not share glyphs (probed: `▛▀▀▀▜ / ▌ x ▐ / ▙▄▄▄▟`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Border {
    pub tl: char,
    pub tr: char,
    pub br: char,
    pub bl: char,
    /// Horizontal pattern of the top border (repeated across the interior).
    pub h_top: &'static str,
    /// Horizontal pattern of the bottom border.
    pub h_bottom: &'static str,
    pub side_l: char,
    pub side_r: char,
}

/// The Unicode border-style table. Unknown styles render solid (upstream
/// dies on unknown style strings at attribute-validation time; gently's
/// model stores values verbatim, so the renderer falls back the same way
/// the ascii renderer does).
pub(super) fn border(style: &str) -> Border {
    match style {
        "dotted" => Border { tl: '┌', tr: '┐', br: '┘', bl: '└', h_top: "⋯", h_bottom: "⋯", side_l: '⋮', side_r: '⋮' },
        "dashed" => Border { tl: '┌', tr: '┐', br: '┘', bl: '└', h_top: "−", h_bottom: "−", side_l: '╎', side_r: '╎' },
        "double" => Border { tl: '╔', tr: '╗', br: '╝', bl: '╚', h_top: "═", h_bottom: "═", side_l: '║', side_r: '║' },
        "dot-dash" => Border { tl: '┌', tr: '┐', br: '┘', bl: '└', h_top: "·-", h_bottom: "·-", side_l: '!', side_r: '!' },
        "dot-dot-dash" => Border {
            tl: '┌', tr: '┐', br: '┘', bl: '└', h_top: "··-", h_bottom: "··-", side_l: '│', side_r: '│',
        },
        "bold" => Border { tl: '┏', tr: '┓', br: '┛', bl: '┗', h_top: "━", h_bottom: "━", side_l: '┃', side_r: '┃' },
        "bold-dash" => Border {
            tl: '┏', tr: '┓', br: '┛', bl: '┗', h_top: "━ ", h_bottom: "━ ", side_l: '╻', side_r: '╻',
        },
        "double-dash" => Border {
            tl: '╔', tr: '╗', br: '╝', bl: '╚', h_top: "═ ", h_bottom: "═ ", side_l: '∥', side_r: '∥',
        },
        "wave" => Border { tl: '┌', tr: '┐', br: '┘', bl: '└', h_top: "∼", h_bottom: "∼", side_l: '≀', side_r: '≀' },
        "broad" => Border { tl: '▛', tr: '▜', br: '▟', bl: '▙', h_top: "▀", h_bottom: "▄", side_l: '▌', side_r: '▐' },
        "wide" => Border { tl: '█', tr: '█', br: '█', bl: '█', h_top: "█", h_bottom: "█", side_l: '█', side_r: '█' },
        "none" => Border { tl: ' ', tr: ' ', br: ' ', bl: ' ', h_top: " ", h_bottom: " ", side_l: ' ', side_r: ' ' },
        _ => Border { tl: '┌', tr: '┐', br: '┘', bl: '└', h_top: "─", h_bottom: "─", side_l: '│', side_r: '│' },
    }
}

/// The interior horizontal run of a border: the pattern shifted by one
/// position (the upstream h_run rule, probed: dot-dash `┌-·-┐`,
/// dot-dot-dash `┌·-·┐`, double-dash `╔ ═ ╗`, bold-dash `┏ ━ ┓` — all
/// `pattern[(i+1) mod len]`).
pub(super) fn h_run(b: &Border, inner: usize) -> String {
    shifted_run(b.h_top, inner)
}

/// `pattern` cycled from offset 1 across `inner` chars.
pub(super) fn shifted_run(pattern: &str, inner: usize) -> String {
    let pat: Vec<char> = pattern.chars().collect();
    let len = pat.len();
    (0..inner).map(|i| pat[(i + 1) % len]).collect()
}

/// How the `rounded` shape resolves for a border style (probed bytes):
/// the `╭╮╯╰` corner set overlays the style's own corners for
/// solid|dotted|dashed|dot-dash|dot-dot-dash; the corners blank for
/// bold|wide|broad|double|double-dash|bold-dash; and wave — listed in
/// neither upstream branch — KEEPS its corners (observed quirk).
pub(super) fn rounded(style: &str) -> Border {
    let mut b = border(style);
    if matches!(style, "solid" | "dotted" | "dashed" | "dot-dash" | "dot-dot-dash") {
        b.tl = '╭';
        b.tr = '╮';
        b.br = '╯';
        b.bl = '╰';
    } else if matches!(
        style,
        "bold" | "wide" | "broad" | "double" | "double-dash" | "bold-dash"
    ) {
        b.tl = ' ';
        b.tr = ' ';
        b.br = ' ';
        b.bl = ' ';
    }
    b
}

/// One edge style's Unicode glyph set (row 1 of the upstream
/// `$edge_styles` table): the horizontal pattern, the vertical glyph, and
/// the four corner glyphs in upstream's (SE, SW, NE, NW) order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Edge {
    pub hor: &'static str,
    pub ver: char,
    /// Corners (SE, SW, NE, NW) — the glyph whose arms match the
    /// neighbourhood (down+right, down+left, up+right, up+left).
    pub corners: [char; 4],
}

/// The Unicode edge-style table. Unknown styles render solid.
pub(super) fn edge(style: &str) -> Edge {
    match style {
        "double" => Edge { hor: "═", ver: '║', corners: ['╔', '╗', '╚', '╝'] },
        "double-dash" => Edge { hor: "═ ", ver: '∥', corners: ['╔', '╗', '╚', '╝'] },
        "dotted" => Edge { hor: "·", ver: ':', corners: ['┌', '┐', '└', '┘'] },
        "dashed" => Edge { hor: "╴", ver: '╵', corners: ['┌', '┐', '╵', '┘'] },
        "dot-dash" => Edge { hor: "·-", ver: '!', corners: ['┌', '┐', '└', '┘'] },
        "dot-dot-dash" => Edge { hor: "··-", ver: '!', corners: ['┌', '┐', '└', '┘'] },
        "wave" => Edge { hor: "∼", ver: '≀', corners: ['┌', '┐', '└', '┘'] },
        "bold" => Edge { hor: "━", ver: '┃', corners: ['┏', '┓', '┗', '┛'] },
        "bold-dash" => Edge { hor: "━ ", ver: '╻', corners: ['┏', '┓', '┗', '┛'] },
        "broad" => Edge { hor: "▬", ver: '▮', corners: ['█', '█', '█', '█'] },
        "wide" => Edge { hor: "█", ver: '█', corners: ['█', '█', '█', '█'] },
        _ => Edge { hor: "─", ver: '│', corners: ['┌', '┐', '└', '┘'] },
    }
}

/// The arrowhead glyph for one direction of travel, boxart row of the
/// upstream `$arrow_styles` table (open triangle): east `>`, west `<`,
/// north `∧`, south `∨` (probed: south `∨` on bends, east `>` on runs).
pub(super) fn arrow(dir: super::polyline::Dir) -> char {
    match dir {
        super::polyline::Dir::East => '>',
        super::polyline::Dir::West => '<',
        super::polyline::Dir::North => '∧',
        super::polyline::Dir::South => '∨',
    }
}

/// The point-shape glyph (upstream `$point_shapes[1]`, filled star).
pub(super) const POINT: char = '★';

/// The +1 width bonus every edge cell of a `dot-dot-dash` edge carries
/// (upstream `Edge::Cell::_correct_size`: "make the edge to display
/// ' ..-> ' instead of ' ..> '"), and the +1 an arrowed bidirectional
/// horizontal cell carries.
pub(super) fn cell_bonus(style: &str, bidirectional_arrowed: bool) -> usize {
    let mut bonus = 0;
    if style == "dot-dot-dash" {
        bonus += 1;
    }
    if bidirectional_arrowed {
        bonus += 1;
    }
    bonus
}

#[cfg(test)]
mod tests {
    use super::super::polyline::Dir;
    use super::{arrow, border, edge, h_run, rounded, shifted_run, Edge, POINT};

    #[test]
    fn solid_and_double_border_glyphs() {
        let b = border("solid");
        assert_eq!(h_run(&b, 3), "───");
        assert_eq!(b.side_l, '│');
        let d = border("double");
        assert_eq!(h_run(&d, 3), "═══");
        assert_eq!((d.tl, d.tr, d.br, d.bl), ('╔', '╗', '╝', '╚'));
    }

    #[test]
    fn shifted_h_runs_match_probe() {
        assert_eq!(h_run(&border("dot-dash"), 3), "-·-");
        assert_eq!(h_run(&border("dot-dot-dash"), 3), "·-·");
        assert_eq!(h_run(&border("double-dash"), 3), " ═ ");
        assert_eq!(h_run(&border("bold-dash"), 3), " ━ ");
        assert_eq!(shifted_run("··-", 12), "·-··-··-··-·");
    }

    #[test]
    fn broad_border_has_distinct_edges() {
        let b = border("broad");
        assert_eq!((b.h_top, b.h_bottom, b.side_l, b.side_r), ("▀", "▄", '▌', '▐'));
    }

    #[test]
    fn rounded_overlays_blanks_or_keeps_per_style() {
        assert_eq!(rounded("solid").tl, '╭');
        assert_eq!(rounded("dotted").tl, '╭');
        assert_eq!(rounded("dot-dot-dash").tl, '╭');
        assert_eq!(rounded("double").tl, ' ');
        assert_eq!(rounded("bold").tl, ' ');
        assert_eq!(rounded("broad").tl, ' ');
        // upstream quirk: wave is in neither branch and keeps its corners
        assert_eq!(rounded("wave").tl, '┌');
    }

    #[test]
    fn edge_glyph_sets_match_probe() {
        let e = edge("double");
        assert_eq!(e, Edge { hor: "═", ver: '║', corners: ['╔', '╗', '╚', '╝'] });
        assert_eq!(edge("dashed").ver, '╵');
        assert_eq!(edge("dotted").ver, ':');
        assert_eq!(edge("bold").corners, ['┏', '┓', '┗', '┛']);
    }

    #[test]
    fn arrows_and_point_match_probe() {
        assert_eq!((arrow(Dir::East), arrow(Dir::South)), ('>', '∨'));
        assert_eq!(arrow(Dir::North), '∧');
        assert_eq!(POINT, '★');
    }
}