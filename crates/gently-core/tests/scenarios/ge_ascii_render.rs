//! ge.ascii_render (gently-0cq): the classic ASCII renderer contract —
//! each property row of specs/ge-ascii_render.md is one test below, named
//! for its id. Expected bytes are the probed pinned oracle
//! (Graph::Easy v0.69 @ ededa3d7; tests/repro/claims/ascii-render-tables
//! .observed), except where the oracle is display-broken: the wide-glyph
//! box follows c5's display-width letter (the oracle aligns CJK by UTF-8
//! byte length — recorded as a divergence for gently-dcp).
//!
//! Scope notes (probed): upstream hangs on multiline labels
//! (tests/repro/probes.pl note), so label *wrapping* is unverifiable
//! against the oracle and stays out of p5's generator; `box` is not a
//! valid upstream shape (the default is `rect`); westward runs are
//! probed for solid only.

use gently_core::graph::{Graph, Scope};
use gently_core::layout;
use gently_core::render::ascii;

/// `[ x ] { border: <style>; }`.
fn bordered_node(style: &str) -> Graph {
    let mut g = Graph::default();
    let n = g.add_node("x");
    g.set_attr(Scope::Node(n), "border", style);
    g
}

/// `[ a ] --> { style: <style>; } [ b ]`, arrowless when `arrowed` is false.
fn styled_edge(style: &str, arrowed: bool) -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let e = g.add_edge(a, b, arrowed).expect("live endpoints");
    g.set_attr(Scope::Edge(e), "style", style);
    if !arrowed {
        g.edges[e].arrows.end = false;
    }
    g
}

/// `[ x ] { shape: <shape>; }`.
fn shaped_node(shape: &str) -> Graph {
    let mut g = Graph::default();
    let n = g.add_node("x");
    g.set_attr(Scope::Node(n), "shape", shape);
    g
}

/// The chain a -> b -> c with edge labels `xy` and `pq`.
fn labelled_chain() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let e1 = g.add_edge(a, b, true).expect("live endpoints");
    let e2 = g.add_edge(b, c, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e1), "label", "xy");
    g.set_attr(Scope::Edge(e2), "label", "pq");
    g
}

/// ge.ascii_render.p1 (c1): each rendered box border matches its declared
/// style (probed oracle bytes for every ASCII border style).
#[test]
fn p1() {
    let cases = [
        ("solid", "+---+\n| x |\n+---+\n"),
        ("dotted", ".....\n: x :\n:...:\n"),
        ("dashed", "+ - +\n' x '\n+ - +\n"),
        ("double", "#===#\nH x H\n#===#\n"),
        ("wave", "+~~~+\n{ x {\n+~~~+\n"),
        ("bold", "#####\n# x #\n#####\n"),
        ("wide", "#####\n# x #\n#####\n"),
        ("broad", "#####\n# x #\n#####\n"),
        ("dot-dash", "+-.-+\n! x !\n+-.-+\n"),
        ("dot-dot-dash", "+.-.+\n| x |\n+.-.+\n"),
        ("double-dash", "# = #\n\" x \"\n# = #\n"),
        ("none", "\n x\n\n"),
    ];
    for (style, want) in cases {
        let art = ascii::render(&bordered_node(style), &layout::layout(&bordered_node(style)))
            .expect("bordered node must render");
        assert_eq!(art, want, "border {style}");
    }
}

/// ge.ascii_render.p2 (c2): each edge's glyph run matches the style table
/// byte-for-byte, arrowed and arrow-less (probed oracle bytes).
#[test]
fn p2() {
    let cases = [
        ("solid", true, "+---+     +---+\n| a | --> | b |\n+---+     +---+\n"),
        ("solid", false, "+---+     +---+\n| a | --- | b |\n+---+     +---+\n"),
        ("double", true, "+---+     +---+\n| a | ==> | b |\n+---+     +---+\n"),
        ("double", false, "+---+     +---+\n| a | === | b |\n+---+     +---+\n"),
        ("dotted", true, "+---+     +---+\n| a | ..> | b |\n+---+     +---+\n"),
        ("dotted", false, "+---+     +---+\n| a | ... | b |\n+---+     +---+\n"),
        ("wave", true, "+---+     +---+\n| a | ~~> | b |\n+---+     +---+\n"),
        ("wave", false, "+---+     +---+\n| a | ~~~ | b |\n+---+     +---+\n"),
        ("dashed", true, "+---+     +---+\n| a | - > | b |\n+---+     +---+\n"),
        ("dashed", false, "+---+     +---+\n| a |  -  | b |\n+---+     +---+\n"),
        ("bold", true, "+---+     +---+\n| a | ##> | b |\n+---+     +---+\n"),
        ("bold", false, "+---+     +---+\n| a | ### | b |\n+---+     +---+\n"),
    ];
    for (style, arrowed, want) in cases {
        let g = styled_edge(style, arrowed);
        let art = ascii::render(&g, &layout::layout(&g)).expect("styled edge must render");
        assert_eq!(art, want, "edge {style} arrowed={arrowed}");
    }
}

/// ge.ascii_render.p3 (c3): the output is a function of the grid alone —
/// one graph built via two different construction orders (nodes-then-
/// edges vs interleaved) renders byte-identically.
#[test]
fn p3() {
    let mut first = Graph::default();
    let a = first.add_node("a");
    let b = first.add_node("b");
    let c = first.add_node("c");
    first.add_edge(a, b, true).expect("live endpoints");
    first.add_edge(b, c, true).expect("live endpoints");

    let mut second = Graph::default();
    let a2 = second.add_node("a");
    let b2 = second.add_node("b");
    second.add_edge(a2, b2, true).expect("live endpoints");
    let c2 = second.add_node("c");
    second.add_edge(b2, c2, true).expect("live endpoints");

    let l1 = layout::layout(&first);
    let l2 = layout::layout(&second);
    let art1 = ascii::render(&first, &l1).expect("must render");
    let art2 = ascii::render(&second, &l2).expect("must render");
    assert_eq!(art1, art2, "construction order must not change the bytes");
}

/// ge.ascii_render.p4 (c4): edge labels occupy cells on their edge's
/// routed path, never a node cell or another label's cell — and the
/// rendered bytes are the probed oracle's (label above the arrow, the
/// run widened to a pattern fill plus head).
#[test]
fn p4() {
    let g = labelled_chain();
    let l = layout::layout(&g);
    // the label cells: on the routed path, off every node cell, distinct
    for ei in 0..2 {
        let label = l.label_cells[ei].expect("labelled edge must publish a label cell");
        assert!(
            l.edge_paths[ei].contains(&label),
            "label cell {label:?} off edge {ei}'s path"
        );
        for (ni, &n) in l.node_cells.iter().enumerate() {
            assert_ne!(label, n, "edge {ei}'s label sits on node {ni}");
        }
    }
    assert_ne!(l.label_cells[0], l.label_cells[1], "labels collide");
    let art = ascii::render(&g, &l).expect("labelled chain must render");
    let want = concat!(
        "+---+  xy   +---+  pq   +---+\n",
        "| a | ----> | b | ----> | c |\n",
        "+---+       +---+       +---+\n",
    );
    assert_eq!(art, want);
}

/// ge.ascii_render.p5 (c5): each glyph occupies its display width —
/// double-width glyphs take two columns (diverging from the oracle's
/// byte-length alignment, see the module notes) — and a long label keeps
/// one interior row (probed: upstream grows the box, no wrap).
#[test]
fn p5() {
    let mut g = Graph::default();
    g.add_node("中");
    let l = layout::layout(&g);
    let art = ascii::render(&g, &l).expect("wide glyph must render");
    let want = "+----+\n| 中 |\n+----+\n";
    assert_eq!(art, want, "中 occupies two columns");

    let long = "this is a very long node label that must wrap somewhere";
    let mut g = Graph::default();
    g.add_node(long);
    let l = layout::layout(&g);
    let art = ascii::render(&g, &l).expect("long label must render");
    let want = format!(
        "+{}+\n| {} |\n+{}+\n",
        "-".repeat(long.len() + 2),
        long,
        "-".repeat(long.len() + 2)
    );
    assert_eq!(art, want, "long label stays one interior row");
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
        let art = ascii::render(&g, &layout::layout(&g)).expect("shaped node must render");
        assert_eq!(art, want, "shape {shape}");
    }
    // color attributes do not affect ASCII output
    let mut g = Graph::default();
    let n = g.add_node("x");
    g.set_attr(Scope::Node(n), "color", "red");
    let art = ascii::render(&g, &layout::layout(&g)).expect("colored node must render");
    assert_eq!(art, "+---+\n| x |\n+---+\n", "color must not change ascii bytes");
}
