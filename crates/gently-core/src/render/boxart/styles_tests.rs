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
    assert_eq!(
        (b.h_top, b.h_bottom, b.side_l, b.side_r),
        ("▀", "▄", '▌', '▐')
    );
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
    assert_eq!(
        e,
        Edge {
            hor: "═",
            ver: '║',
            corners: ['╔', '╗', '╚', '╝']
        }
    );
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
