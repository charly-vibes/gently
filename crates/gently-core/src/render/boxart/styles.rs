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
use super::edge_table::{DEFAULT_EDGE, EDGE_TABLE};

pub(super) fn border(style: &str) -> Border {
    BORDER_TABLE
        .iter()
        .find(|(name, _)| *name == style)
        .map(|(_, b)| *b)
        .unwrap_or(DEFAULT_BORDER)
}

/// Per-style border glyph sets (name, glyphs), in upstream's match order.
const BORDER_TABLE: &[(&str, Border)] = &[
    (
        "dotted",
        Border {
            tl: '┌',
            tr: '┐',
            br: '┘',
            bl: '└',
            h_top: "⋯",
            h_bottom: "⋯",
            side_l: '⋮',
            side_r: '⋮',
        },
    ),
    (
        "dashed",
        Border {
            tl: '┌',
            tr: '┐',
            br: '┘',
            bl: '└',
            h_top: "−",
            h_bottom: "−",
            side_l: '╎',
            side_r: '╎',
        },
    ),
    (
        "double",
        Border {
            tl: '╔',
            tr: '╗',
            br: '╝',
            bl: '╚',
            h_top: "═",
            h_bottom: "═",
            side_l: '║',
            side_r: '║',
        },
    ),
    (
        "dot-dash",
        Border {
            tl: '┌',
            tr: '┐',
            br: '┘',
            bl: '└',
            h_top: "·-",
            h_bottom: "·-",
            side_l: '!',
            side_r: '!',
        },
    ),
    (
        "dot-dot-dash",
        Border {
            tl: '┌',
            tr: '┐',
            br: '┘',
            bl: '└',
            h_top: "··-",
            h_bottom: "··-",
            side_l: '│',
            side_r: '│',
        },
    ),
    (
        "bold",
        Border {
            tl: '┏',
            tr: '┓',
            br: '┛',
            bl: '┗',
            h_top: "━",
            h_bottom: "━",
            side_l: '┃',
            side_r: '┃',
        },
    ),
    (
        "bold-dash",
        Border {
            tl: '┏',
            tr: '┓',
            br: '┛',
            bl: '┗',
            h_top: "━ ",
            h_bottom: "━ ",
            side_l: '╻',
            side_r: '╻',
        },
    ),
    (
        "double-dash",
        Border {
            tl: '╔',
            tr: '╗',
            br: '╝',
            bl: '╚',
            h_top: "═ ",
            h_bottom: "═ ",
            side_l: '∥',
            side_r: '∥',
        },
    ),
    (
        "wave",
        Border {
            tl: '┌',
            tr: '┐',
            br: '┘',
            bl: '└',
            h_top: "∼",
            h_bottom: "∼",
            side_l: '≀',
            side_r: '≀',
        },
    ),
    (
        "broad",
        Border {
            tl: '▛',
            tr: '▜',
            br: '▟',
            bl: '▙',
            h_top: "▀",
            h_bottom: "▄",
            side_l: '▌',
            side_r: '▐',
        },
    ),
    (
        "wide",
        Border {
            tl: '█',
            tr: '█',
            br: '█',
            bl: '█',
            h_top: "█",
            h_bottom: "█",
            side_l: '█',
            side_r: '█',
        },
    ),
    (
        "none",
        Border {
            tl: ' ',
            tr: ' ',
            br: ' ',
            bl: ' ',
            h_top: " ",
            h_bottom: " ",
            side_l: ' ',
            side_r: ' ',
        },
    ),
];

/// The plain solid border used for unknown styles.
const DEFAULT_BORDER: Border = Border {
    tl: '┌',
    tr: '┐',
    br: '┘',
    bl: '└',
    h_top: "─",
    h_bottom: "─",
    side_l: '│',
    side_r: '│',
};

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
    let corners: [char; 4] = if matches!(
        style,
        "solid" | "dotted" | "dashed" | "dot-dash" | "dot-dot-dash"
    ) {
        ['╭', '╮', '╯', '╰']
    } else if matches!(
        style,
        "bold" | "wide" | "broad" | "double" | "double-dash" | "bold-dash"
    ) {
        [' ', ' ', ' ', ' ']
    } else {
        let b = border(style);
        [b.tl, b.tr, b.br, b.bl]
    };
    Border {
        tl: corners[0],
        tr: corners[1],
        br: corners[2],
        bl: corners[3],
        ..border(style)
    }
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
    EDGE_TABLE
        .iter()
        .find(|(names, _)| names.contains(&style))
        .map(|(_, e)| *e)
        .unwrap_or(DEFAULT_EDGE)
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
#[path = "styles_tests.rs"]
mod tests;
