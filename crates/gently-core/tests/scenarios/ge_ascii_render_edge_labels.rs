//! ge.ascii_render p4 additions (gently-527): a straight edge's label
//! splits into per-line rows — pinned from the recorded oracle bytes.

use super::{ascii, layout, text};

/// ge.ascii_render.p4 (gently-527): a literal `\n` inside an edge label
/// splits the label into LEFT-aligned rows (the edge `align` default) —
/// two rows ride the band's top two rows, the arrow stays on the grown
/// band's `H-2` line, and the label cell sizes its row band (both boxes
/// grow one blank interior row; upstream `dimensions`/`_correct_size` +
/// `_prepare_layout`'s row growth). The gap is the widest LINE's byte
/// length + 5 (gently-bvb), not the raw label's. Expected bytes:
/// tests/repro/claims/so-flowchart-size.observed, disentangled from the
/// fixture's `{ size: 5,10; }` attrs and 2x2 layout (out of scope).
#[test]
fn p4_newline_edge_label_splits_into_left_aligned_rows() {
    let g =
        text::parse("[ A ] -- put msg\\n via http --> [ B ]\n").expect("the \\n label must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("the \\n label must render");
    assert_eq!(
        art,
        concat!(
            "+---+  put msg    +---+\n",
            "| A |  via http   | B |\n",
            "|   | ----------> |   |\n",
            "+---+             +---+\n",
        ),
        "the \\n escape splits the edge label over two rows"
    );
}

/// ge.ascii_render.p4 (gently-y7i): `\r` / `\n` inside an edge label
/// align the FOLLOWING line — `A\r long\n edge label` renders `A`
/// (default/left), `long` right-aligned in the `w-5` field, and
/// `edge label` back at the default — with the label cell growing its
/// row band to lines+2. Expected bytes: the edge half of
/// tests/repro/claims/bg-multiline-align.observed (the node half is
/// pinned under p5_alignment_escapes_align_node_label_rows).
#[test]
fn p4_alignment_escapes_align_edge_label_rows() {
    let g = text::parse("[ A ] -- A\\r long\\n edge label --> [ B ]\n")
        .expect("the aligned label must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("the aligned label must render");
    assert_eq!(
        art,
        concat!(
            "+---+  A            +---+\n",
            "|   |        long   |   |\n",
            "| A |  edge label   | B |\n",
            "|   | ------------> |   |\n",
            "+---+               +---+\n",
        ),
        "\\r and \\n align the edge label rows per the oracle"
    );
}
