use super::*;

/// ge.ascii_render.p7 (c7, gently-b0z): anonymous nodes render boxless —
/// the oracle's `node.anon` class carries borderstyle none and the label
/// `' '`, so the node's own 3×3 cell renders blank and the edges attach
/// to the empty space. Expected bytes: tests/repro/claims/pod-anon-nodes
/// .observed (seed 0, Parser.pm POD anonymous-nodes example).
#[test]
fn p7() {
    let g = text::parse("[ ] --> [ Bonn ]\n[ Bonn ] --> [ ]\n").must("anon chain must parse");
    let art = ascii::render(&g, &layout::layout(&g)).must("anon chain must render");
    assert_eq!(
        art,
        "        +------+\n    --> | Bonn | -->\n        +------+\n"
    );
}

/// ge.ascii_render.p8 (c8, gently-7rx bonus): a node group renders the
/// dashed border frame (`+ - - +` corners with `'` sides — the dashed
/// style's border row-0 glyphs) around its members with the `Name:`
/// title band above; the title cell shares the leftmost member's grid
/// column and its `label+2` minimum widens it. Expected bytes:
/// tests/repro/claims/pod-group.observed (seed 0, Parser.pm POD groups
/// example).
#[test]
fn p8() {
    let g = text::parse("( Cities:\n  [ Berlin ]\n  [ Bonn ]\n)\n\n[ Bonn ] --> [ Berlin ]\n")
        .must("group example must parse");
    let art = ascii::render(&g, &layout::layout(&g)).must("group example must render");
    assert_eq!(
        art,
        concat!(
            "+ - - - - - - - - - - - - -+\n",
            "' Cities:                  '\n",
            "'                          '\n",
            "' +-------+     +--------+ '\n",
            "' | Bonn  | --> | Berlin | '\n",
            "' +-------+     +--------+ '\n",
            "'                          '\n",
            "+ - - - - - - - - - - - - -+\n",
        )
    );
}

/// ge.ascii_render.p6 (c6): each shape's outline matches the probed
/// upstream ASCII table — the special shapes (rounded, point, invisible)
/// diverge, every other vocabulary shape collapses to the plain box for
/// small nodes; color attributes leave the output byte-identical.
#[test]
fn p6() {
    let cases = [
        ("rounded", " ---\n| x |\n ---\n"),
        ("point", "\n  *\n\n"),
        ("invisible", "\n\n\n"),
        ("img", "+---+\n| x |\n+---+\n"),
        ("circle", "+---+\n| x |\n+---+\n"),
        ("ellipse", "+---+\n| x |\n+---+\n"),
        ("diamond", "+---+\n| x |\n+---+\n"),
        ("triangle", "+---+\n| x |\n+---+\n"),
        ("pentagon", "+---+\n| x |\n+---+\n"),
        ("hexagon", "+---+\n| x |\n+---+\n"),
        ("octagon", "+---+\n| x |\n+---+\n"),
        ("parallelogram", "+---+\n| x |\n+---+\n"),
        ("house", "+---+\n| x |\n+---+\n"),
    ];
    for (shape, want) in cases {
        let g = shaped_node(shape);
        let art = ascii::render(&g, &layout::layout(&g)).must("shaped node must render");
        assert_eq!(art, want, "shape {shape}");
    }
    // color attributes do not affect ASCII output
    let mut g = Graph::default();
    let n = g.add_node("x");
    g.set_attr(Scope::Node(n), "color", "red");
    let art = ascii::render(&g, &layout::layout(&g)).must("colored node must render");
    assert_eq!(
        art, "+---+\n| x |\n+---+\n",
        "color must not change ascii bytes"
    );
}
