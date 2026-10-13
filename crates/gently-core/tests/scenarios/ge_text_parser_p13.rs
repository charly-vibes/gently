//! ge.text_parser.p13 (c13) scenario — a trailing attribute block after a
//! node name applies to that node (never silently dropped), and the node's
//! label value is its display label in rendered output, falling back to
//! the name. Evidence: tests/repro/claims/bg-trailing-node-attr.observed
//! (v0.69 @ ededa3d7: the oracle renders Big as Huge).

use super::node;
use gently_core::graph::Scope;
use gently_core::parse::text;

/// The canonical txt form of a parsed graph (ge.txt_render pipeline).
fn render_txt(src: &str) -> String {
    let g = text::parse(src).expect("render_txt: must parse");
    gently_core::render::txt::render(&g)
}

/// The ascii render of a parsed graph (parse → layout → render pipeline).
fn render_ascii(src: &str) -> String {
    let g = text::parse(src).expect("render_ascii: must parse");
    let lay = gently_core::layout::layout(&g);
    gently_core::render::ascii::render(&g, &lay).expect("render_ascii: must render")
}

/// The trailing block lands on the node; the canonical txt form keeps
/// name + attribute (oracle as_txt bytes).
#[test]
fn trailing_block_lands_and_txt_keeps_the_name() {
    let src = "[ Big ] { label: \"Huge\"; }\n";
    let g = text::parse(src).expect("must parse");
    assert_eq!(
        g.get_attr(Scope::Node(node(&g, "Big")), "label"),
        Some("Huge"),
        "the trailing block lands on the node"
    );
    assert_eq!(render_txt(src), "[ Big ] { label: Huge; }\n\n");
}

/// The ascii render displays the label value, not the name; a node
/// without a label attribute falls back to its name.
#[test]
fn ascii_displays_the_label_value() {
    assert_eq!(
        render_ascii("[ Big ] { label: \"Huge\"; }\n"),
        "+------+\n| Huge |\n+------+\n"
    );
    assert_eq!(render_ascii("[ Big ]\n"), "+-----+\n| Big |\n+-----+\n");
}

/// The pinned quoting-section input renders Huge in the box
/// (bg-trailing-node-attr.observed bytes).
#[test]
fn pinned_quoting_input_bytes() {
    let art =
        render_ascii("[ \"Monitor Size\" ] --> { label: 21\"; } [ Big ] { label: \"Huge\"; }\n");
    assert_eq!(
        art,
        "+----------------+  21\"   +------+\n| \"Monitor Size\" | -----> | Huge |\n+----------------+        +------+\n"
    );
}
