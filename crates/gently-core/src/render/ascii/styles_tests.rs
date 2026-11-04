use super::*;

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
