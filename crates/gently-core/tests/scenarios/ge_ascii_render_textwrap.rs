//! ge.ascii_render p5 additions (gently-emd): `textwrap: auto` and a
//! numeric `textwrap: N` wrap label text — pinned from the recorded
//! oracle bytes (tests/repro/claims/bg-textwrap.observed).

use super::{ascii, layout, text};

/// ge.ascii_render.p5 (gently-emd): `graph { textwrap: auto; }` wraps
/// the node label at `int(sqrt(bytes) * 1.4)` (33 bytes → 8) and the
/// edge's own `textwrap: 10` overrides the inherited mode — wrapping
/// REPLACES the `\n`/`\r`/`\l`/`\c` escape split (each escape becomes a
/// space, whitespace collapses, lines break at word boundaries), and
/// every row carries the object's default alignment (node center, edge
/// left). The wrapped lines drive the box/gap sizing exactly as aligned
/// lines do: the node box grows to lines+2, the labelled straight gap
/// to the widest line + 5, and the multiline rows ride the grown bands.
/// Expected bytes: tests/repro/claims/bg-textwrap.observed (both seeds).
#[test]
fn p5_textwrap_auto_and_numeric_wrap_labels() {
    let g = text::parse(concat!(
        "graph { textwrap: auto; }\n",
        "\n",
        "[ Long Node Label left right center ]",
        " -- A long edge label --> { textwrap: 10; } [ B ]\n",
    ))
    .expect("the textwrap attrs must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("the wrapped labels must render");
    assert_eq!(
        art,
        concat!(
            "+--------+           +---+\n",
            "|  Long  |           |   |\n",
            "|  Node  |           |   |\n",
            "| Label  |  A long   | B |\n",
            "|  left  |  edge     |   |\n",
            "| right  |  label    |   |\n",
            "| center | --------> |   |\n",
            "+--------+           +---+\n",
        ),
        "textwrap wraps node and edge labels per the oracle"
    );
}

/// ge.ascii_render.p5 (gently-emd): the default textwrap mode stays
/// `none` — no wrap, the 527 escape-split surface byte-identical.
#[test]
fn p5_textwrap_defaults_to_none() {
    let g = text::parse("[ Long Node Label left right center ] --> [ B ]\n")
        .expect("the graph must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("must render");
    assert_eq!(
        art,
        concat!(
            "+-----------------------------------+     +---+\n",
            "| Long Node Label left right center | --> | B |\n",
            "+-----------------------------------+     +---+\n",
        ),
        "without textwrap the label renders unwrapped"
    );
}
