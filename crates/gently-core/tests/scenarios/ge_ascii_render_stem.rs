//! ge.ascii_render p2 addition (gently-5vf): the vertical stem column —
//! pinned from the recorded oracle bytes
//! (tests/repro/claims/so-graph-convert.observed).

use super::{ascii, layout, text};

/// ge.ascii_render.p2 (gently-5vf, SO 71869025): a branch's vertical
/// stem hangs from its source box at the box's left+2 — the upstream
/// `_draw_cross` vertical offset (gently-klf's attachment rule) — NOT
/// at the box's center column: the `/` box is 6 wide and its `/c`
/// branch renders `|`, `|`, `v` at column 2, byte-identical to the
/// answerer's posted bytes. Expected bytes:
/// tests/repro/claims/so-graph-convert.observed (both seeds).
#[test]
fn p2_vertical_stem_attaches_at_left_plus_two() {
    let g = text::parse("[ / ] --> [ /a ]\n[ / ] --> [ /c ]\n[ /a ] --> [ /b ]\n")
        .expect("the URI tree must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("the URI tree must render");
    assert_eq!(
        art,
        concat!(
            "+----+     +----+     +----+\n",
            "| /  | --> | /a | --> | /b |\n",
            "+----+     +----+     +----+\n",
            "  |\n",
            "  |\n",
            "  v\n",
            "+----+\n",
            "| /c |\n",
            "+----+\n",
        ),
        "the /c stem hangs at its source box's left+2 column"
    );
}
