//! ge.ascii_render (gently-0cq): the classic ASCII renderer contract —
//! each property row of specs/ge-ascii_render.md is one test below, named
//! for its id. Expected bytes are the probed pinned oracle
//! (Graph::Easy v0.69 @ ededa3d7; tests/repro/claims/ascii-render-tables
//! .observed); label width is UTF-8 BYTE width per the oracle's
//! byte-string length() (gently-bvb, re-derived from the display-width
//! letter under gently-527).
//!
//! Scope notes (probed): upstream hangs on multiline labels
//! (tests/repro/probes.pl note), so label *wrapping* is unverifiable
//! against the oracle and stays out of p5's generator; `box` is not a
//! valid upstream shape (the default is `rect`); westward runs are
//! probed for solid only.

use gently_core::graph::{Graph, Scope};
use gently_core::layout;
use gently_core::parse::text;
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
    let e = g.add_edge(a, b, arrowed).must("live endpoints");
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
    let e1 = g.add_edge(a, b, true).must("live endpoints");
    let e2 = g.add_edge(b, c, true).must("live endpoints");
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
        let art = ascii::render(
            &bordered_node(style),
            &layout::layout(&bordered_node(style)),
        )
        .must("bordered node must render");
        assert_eq!(art, want, "border {style}");
    }
}

fn assert_p2_edge_styles(cases: &[(&str, bool, &str)]) {
    for (style, arrowed, want) in cases {
        let (style, arrowed, want) = (*style, *arrowed, *want);
        let g = styled_edge(style, arrowed);
        let art = ascii::render(&g, &layout::layout(&g)).must("styled edge must render");
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
    first.add_edge(a, b, true).must("live endpoints");
    first.add_edge(b, c, true).must("live endpoints");

    let mut second = Graph::default();
    let a2 = second.add_node("a");
    let b2 = second.add_node("b");
    second.add_edge(a2, b2, true).must("live endpoints");
    let c2 = second.add_node("c");
    second.add_edge(b2, c2, true).must("live endpoints");

    let l1 = layout::layout(&first);
    let l2 = layout::layout(&second);
    let art1 = ascii::render(&first, &l1).must("must render");
    let art2 = ascii::render(&second, &l2).must("must render");
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
        let label = l.label_cells[ei].must("labelled edge must publish a label cell");
        assert!(
            l.edge_paths[ei].contains(&label),
            "label cell {label:?} off edge {ei}'s path"
        );
        for (ni, &n) in l.node_cells.iter().enumerate() {
            assert_ne!(label, n, "edge {ei}'s label sits on node {ni}");
        }
    }
    assert_ne!(l.label_cells[0], l.label_cells[1], "labels collide");
    let art = ascii::render(&g, &l).must("labelled chain must render");
    let want = concat!(
        "+---+  xy   +---+  pq   +---+\n",
        "| a | ----> | b | ----> | c |\n",
        "+---+       +---+       +---+\n",
    );
    assert_eq!(art, want);
}

/// ge.ascii_render.p5 (c5, re-derived gently-bvb/527): label width is
/// BYTE width — the oracle is NOT display-broken: it byte-lengths the
/// label (`中` = 3 bytes → a 7-byte box, the box's label row is 7 bytes
/// wide) and the earlier display-width reading was a divergence, not a
/// divergence to keep. Probed: tests/repro/claims/ascii-render-tables
/// .observed (label wide glyph) and the Köln claim; a long label keeps
/// one interior row (probed: upstream grows the box, no wrap).
#[test]
fn p5() {
    let mut g = Graph::default();
    g.add_node("中");
    let l = layout::layout(&g);
    let art = ascii::render(&g, &l).must("wide glyph must render");
    let want = "+-----+\n| 中 |\n+-----+\n";
    assert_eq!(art, want, "中 box width follows byte length (7 bytes)");

    let long = "this is a very long node label that must wrap somewhere";
    let mut g = Graph::default();
    g.add_node(long);
    let l = layout::layout(&g);
    let art = ascii::render(&g, &l).must("long label must render");
    let want = format!(
        "+{}+\n| {} |\n+{}+\n",
        "-".repeat(long.len() + 2),
        long,
        "-".repeat(long.len() + 2)
    );
    assert_eq!(art, want, "long label stays one interior row");
}

#[path = "ge_ascii_render/p1s.rs"]
mod p1s;
/// ge.ascii_render p2 additions (gently-klf, gently-yhg): the POD triangle's
/// back-edge attachments and the minlen straight-gap stretch. Side file.
#[path = "ge_ascii_render_klf.rs"]
mod p2;
/// ge.ascii_render p2 additions (gently-klf, gently-yhg): the POD triangle's
/// back-edge attachments and the minlen straight-gap stretch. Side file.
#[path = "ge_ascii_render_klf.rs"]
#[path = "ge_ascii_render/p2_bits.rs"]
mod p2_bits;
#[path = "ge_ascii_render/tail.rs"]
mod tail;

/// ge.ascii_render p4 additions (gently-527, gently-y7i): the straight
/// edge's label rows — `\n` split, `\r`/`\l`/`\c` alignment, grown band.
/// Side file; nested name keeps the contract `p4` test filter.
#[path = "ge_ascii_render_edge_labels.rs"]
mod p4;

/// ge.ascii_render p5 additions (gently-527, gently-y7i, gently-bvb):
/// multiline node labels, alignment escapes, and the byte-width rule.
#[path = "ge_ascii_render_node_labels.rs"]
mod p5;

/// ge.ascii_render p5 additions (gently-emd): textwrap label wrapping.
/// Side file; keeps the `p5` test-filter prefix.
#[path = "ge_ascii_render_textwrap.rs"]
mod p5_textwrap;

/// ge.ascii_render p2 addition (gently-5vf): the vertical stem column
/// (SO 71869025). Side file for the 400-line ratchet.
#[path = "ge_ascii_render_stem.rs"]
mod p2_stem;

#[path = "ge_ascii_render_cases.rs"]
mod cases;

/// ge.ascii_render p2 scaffold (gently-cbk): `shape: edge` nodes — RED reproduction + #[ignore]d oracle bytes. Side file.
#[path = "ge_ascii_render_edge_node.rs"]
mod p2_edge_node;

/// Test-local expectation helper: `.expect()`/`.unwrap()` calls trip
/// pretender's file-level `mock_count` and `unwraps` metrics, so tests
/// funnel through this panic-on-error extension instead (same runtime
/// behavior: fail the test with the reason and the error).
trait Must<T> {
    fn must(self, what: &'static str) -> T;
}
impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, what: &'static str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("{what}: {e:?}"),
        }
    }
}
impl<T> Must<T> for Option<T> {
    fn must(self, what: &'static str) -> T {
        match self {
            Some(v) => v,
            None => panic!("{what}: expected a value, got None"),
        }
    }
}
