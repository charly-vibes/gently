//! The Unicode edge-style table (upstream `$edge_styles`), extracted to
//! keep `styles.rs` under the file-size gate.

use super::styles::Edge;

/// Per-style Unicode edge glyphs (style names, glyphs), in upstream's
/// match order.
pub(super) const EDGE_TABLE: &[(&[&str], Edge)] = &[
    (
        &["double"],
        Edge {
            hor: "═",
            ver: '║',
            corners: ['╔', '╗', '╚', '╝'],
        },
    ),
    (
        &["double-dash"],
        Edge {
            hor: "═ ",
            ver: '∥',
            corners: ['╔', '╗', '╚', '╝'],
        },
    ),
    (
        &["dotted"],
        Edge {
            hor: "·",
            ver: ':',
            corners: ['┌', '┐', '└', '┘'],
        },
    ),
    (
        &["dashed"],
        Edge {
            hor: "╴",
            ver: '╵',
            corners: ['┌', '┐', '╵', '┘'],
        },
    ),
    (
        &["dot-dash"],
        Edge {
            hor: "·-",
            ver: '!',
            corners: ['┌', '┐', '└', '┘'],
        },
    ),
    (
        &["dot-dot-dash"],
        Edge {
            hor: "··-",
            ver: '!',
            corners: ['┌', '┐', '└', '┘'],
        },
    ),
    (
        &["wave"],
        Edge {
            hor: "∼",
            ver: '≀',
            corners: ['┌', '┐', '└', '┘'],
        },
    ),
    (
        &["bold"],
        Edge {
            hor: "━",
            ver: '┃',
            corners: ['┏', '┓', '┗', '┛'],
        },
    ),
    (
        &["bold-dash"],
        Edge {
            hor: "━ ",
            ver: '╻',
            corners: ['┏', '┓', '┗', '┛'],
        },
    ),
    (
        &["broad"],
        Edge {
            hor: "▬",
            ver: '▮',
            corners: ['█', '█', '█', '█'],
        },
    ),
    (
        &["wide"],
        Edge {
            hor: "█",
            ver: '█',
            corners: ['█', '█', '█', '█'],
        },
    ),
];

/// The solid edge glyphs used for unknown styles.
pub(super) const DEFAULT_EDGE: Edge = Edge {
    hor: "─",
    ver: '│',
    corners: ['┌', '┐', '└', '┘'],
};
