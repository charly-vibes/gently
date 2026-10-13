//! ge.ascii_render p5 additions (gently-527, gently-y7i, gently-bvb):
//! multiline node labels, the `\l`/`\r`/`\c` alignment escapes, and the
//! byte-width rule — pinned from the recorded oracle bytes.

use super::{ascii, layout, text};

/// ge.ascii_render.p5 (gently-527): a `\n` inside a node label splits it
/// into one interior row per line (upstream `_aligned_label`), the box
/// growing to lines+2 (`Node::_correct_size`). Expected bytes: probed
/// pinned oracle (seed-stable two-node render).
#[test]
fn p5_node_label_newline_splits_interior_rows() {
    let g = text::parse("[ a\\nb ]\n").expect("the \\n label must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("the \\n label must render");
    assert_eq!(
        art,
        "+---+\n| a |\n| b |\n+---+\n",
        "the \\n escape splits the node label over two interior rows"
    );
}

/// ge.ascii_render.p5 (gently-y7i): `\l` / `\r` / `\c` inside a label
/// align the FOLLOWING line left/right/center (upstream `_aligned_label`:
/// the escape terminates the previous segment and sets the next line's
/// alignment; `\n` falls back to the object's default) — in the node's
/// box (`w-4` field at `x0 = 2`) and the edge's gap (`w-5` field) alike,
/// with grown boxes centering their label rows vertically. Expected
/// bytes: tests/repro/claims/bg-multiline-align.observed.
#[test]
fn p5_alignment_escapes_align_label_rows() {
    let g = text::parse("[ Long Node Label\\l left\\r right\\c center ] -- A\\r long\\n edge label --> [ B ]\n")
        .expect("the aligned labels must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("the aligned labels must render");
    assert_eq!(
        art,
        concat!(
            "+-----------------+               +---+\n",
            "| Long Node Label |  A            |   |\n",
            "| left            |        long   | B |\n",
            "|           right |  edge label   |   |\n",
            "|     center      | ------------> |   |\n",
            "+-----------------+               +---+\n",
        ),
        "the \\l/\\r/\\c escapes align the node and edge label rows"
    );
}

/// ge.ascii_render.p5 (gently-bvb, re-derived from the display-width
/// letter): label width arithmetic counts UTF-8 BYTES (upstream's
/// byte-string `length()`), so `[ Köln ]` draws a 7-dash box (bytes 5 +
/// 2 per side) — and the box's rows are BYTE rows: the label row is 7
/// bytes wide even though `ö` occupies one display column, exactly the
/// oracle's byte-string framebuffer.
#[test]
fn p5_label_width_counts_utf8_bytes() {
    let g = text::parse("[ A ] --> [ Köln ]\n").expect("the umlaut label must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("the umlaut label must render");
    assert_eq!(
        art,
        "+---+     +-------+\n| A | --> | Köln |\n+---+     +-------+\n",
        "Köln's box is 7 dashes wide (5 label bytes + 2 per side)"
    );
}
