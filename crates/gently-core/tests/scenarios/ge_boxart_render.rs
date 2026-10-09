//! ge.boxart_render (gently-css): the Unicode boxart renderer contract —
//! each property row of specs/ge-boxart_render.md is one test below, named
//! for its id. Expected bytes are the probed pinned oracle (Graph::Easy
//! v0.69 @ ededa3d7; tests/repro/claims/boxart-render-tables.observed),
//! captured via `as_boxart` (the same render path as `as_ascii` with
//! `_ascii_style = 1`).
//!
//! Scope notes (probed):
//! - Junction/neighbourhood cells do NOT combine glyphs (c2): a border
//!   cell keeps its own glyph where an edge attaches, and a polyline
//!   corner is the edge style's own corner glyph for its neighbourhood —
//!   p2 pins the observed junction bytes.
//! - A bend's vertical pieces attach two chars into the node's column
//!   (upstream draws corner cells at fixed cell-x 2), which for standard
//!   boxes is the label column.
//! - Westward single-headed edges are not expressible in the upstream
//!   text format (`<--` alone is a parse error; only bidirectional `<->`
//!   exists), so westward runs stay out of p3's generator, as in
//!   ge.ascii_render.
//! - `box` is not a valid upstream shape (parse-rejected; recorded in
//!   shape-outline-collapse.observed), so p4's generator uses `rect` for
//!   the plain box.
//! - The oracle hangs on multiline labels; label wrapping stays out by
//!   design (same exclusion as ge.ascii_render).

use gently_core::graph::{Graph, Scope};
use gently_core::layout;
use gently_core::render::boxart;

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

/// The mixed-style junction chain x →(dashed) a →(double) b →(dotted) c.
fn mixed_chain() -> Graph {
    let mut g = Graph::default();
    let x = g.add_node("x");
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let e1 = g.add_edge(x, a, true).expect("live endpoints");
    let e2 = g.add_edge(a, b, true).expect("live endpoints");
    let e3 = g.add_edge(b, c, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e1), "style", "dashed");
    g.set_attr(Scope::Edge(e2), "style", "double");
    g.set_attr(Scope::Edge(e3), "style", "dotted");
    g
}

/// The cycle a → b → a, first edge `style_a`, return edge `style_b`.
fn cycle(style_a: &str, style_b: &str) -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let e1 = g.add_edge(a, b, true).expect("live endpoints");
    let e2 = g.add_edge(b, a, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e1), "style", style_a);
    g.set_attr(Scope::Edge(e2), "style", style_b);
    g
}

/// The two-node graph `[ a ] <--> [ b ]` (arrowheads at both ends).
fn bidirectional() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let e = g.add_edge(a, b, true).expect("live endpoints");
    g.edges[e].arrows.start = true;
    g
}

fn render(g: &Graph) -> String {
    boxart::render(g, &layout::layout(g)).expect("must render")
}

/// p1 (c1): each node border style yields its Unicode glyph set, one
/// character per cell — including the shifted repeat units of the dash
/// styles over a wide label.
#[test]
fn p1() {
    for (style, want) in [
        ("solid", "┌───┐\n│ x │\n└───┘\n"),
        ("dotted", "┌⋯⋯⋯┐\n⋮ x ⋮\n└⋯⋯⋯┘\n"),
        ("dashed", "┌−−−┐\n╎ x ╎\n└−−−┘\n"),
        ("double", "╔═══╗\n║ x ║\n╚═══╝\n"),
        ("wave", "┌∼∼∼┐\n≀ x ≀\n└∼∼∼┘\n"),
        ("bold", "┏━━━┓\n┃ x ┃\n┗━━━┛\n"),
        ("wide", "█████\n█ x █\n█████\n"),
        ("broad", "▛▀▀▀▜\n▌ x ▐\n▙▄▄▄▟\n"),
        ("dot-dash", "┌-·-┐\n! x !\n└-·-┘\n"),
        ("dot-dot-dash", "┌·-·┐\n│ x │\n└·-·┘\n"),
        ("double-dash", "╔ ═ ╗\n∥ x ∥\n╚ ═ ╝\n"),
        ("bold-dash", "┏ ━ ┓\n╻ x ╻\n┗ ━ ┛\n"),
        ("none", "\n x\n\n"),
    ] {
        assert_eq!(render(&bordered_node(style)), want, "border {style}");
    }
    // repeat units over a wide label: the horizontal pattern is cycled
    // from offset 1 across the interior (upstream h_run shift rule).
    for (style, want) in [
        ("solid", "┌────────────┐\n│ xxxxxxxxxx │\n└────────────┘\n"),
        ("dot-dash", "┌-·-·-·-·-·-·┐\n! xxxxxxxxxx !\n└-·-·-·-·-·-·┘\n"),
        ("dot-dot-dash", "┌·-··-··-··-·┐\n│ xxxxxxxxxx │\n└·-··-··-··-·┘\n"),
        ("double-dash", "╔ ═ ═ ═ ═ ═ ═╗\n∥ xxxxxxxxxx ∥\n╚ ═ ═ ═ ═ ═ ═╝\n"),
        ("bold-dash", "┏ ━ ━ ━ ━ ━ ━┓\n╻ xxxxxxxxxx ╻\n┗ ━ ━ ━ ━ ━ ━┛\n"),
    ] {
        let mut g = bordered_node(style);
        let n = g.nodes.len() - 1;
        g.nodes[n].name = "xxxxxxxxxx".to_string();
        assert_eq!(render(&g), want, "border {style} wide label");
    }
}

/// p2 (c2): junction cells — border cells keep their own glyph where an
/// edge attaches (no glyph combining is observed), bend corners are the
/// edge style's own corner glyphs for their neighbourhood, and the
/// straight/mixed/bend/selfloop/bidirectional neighbourhoods render the
/// probed combined bytes.
#[test]
fn p2() {
    // sanity: the junction generator is the mixed chain it claims to be
    // (the neighbourhood assertions live in the helpers below).
    assert_eq!(mixed_chain().edges.len(), 3);
    assert_mixed_style_chain();
    assert_border_attachments_keep_their_glyphs();
    assert_none_border_box();
    assert_bend_corners();
    assert_selfloop_elbows();
    assert_bidirectional_arrowheads();
}

/// The mixed-style chain: every edge cell carries its own style's glyphs.
fn assert_mixed_style_chain() {
    assert_eq!(
        render(&mixed_chain()),
        "┌───┐     ┌───┐     ┌───┐     ┌───┐\n\
         │ x │ ╴╴> │ a │ ══> │ b │ ··> │ c │\n\
         └───┘     └───┘     └───┘     └───┘\n"
    );
}

/// Border attachments keep the border's own glyphs (dashed ╎ / bold ┃);
/// a none-border box shows no glyphs at all.
fn assert_border_attachments_keep_their_glyphs() {
    for (border, want) in [
        (
            "dashed",
            "┌−−−┐     ┌───┐\n╎ a ╎ ──> │ b │\n└−−−┘     └───┘\n",
        ),
        (
            "bold",
            "┏━━━┓     ┌───┐\n┃ a ┃ ──> │ b │\n┗━━━┛     └───┘\n",
        ),
    ] {
        let mut g = Graph::default();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.add_edge(a, b, true).expect("live endpoints");
        g.set_attr(Scope::Node(a), "border", border);
        assert_eq!(render(&g), want, "border {border} + solid edge");
    }
}

/// The none-border box with a straight edge: label column intact.
fn assert_none_border_box() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    g.add_edge(a, b, true).expect("live endpoints");
    g.set_attr(Scope::Node(a), "border", "none");
    assert_eq!(render(&g), "        ┌───┐\n a  ──> │ b │\n        └───┘\n");
}

/// Bend (cycle) neighbourhoods: corners are the edge style's SE/SW
/// corner glyphs, verticals the ver glyphs, arrowheads '∨'.
fn assert_bend_corners() {
    assert_eq!(
        render(&cycle("solid", "solid")),
        "\n  ┌─────────┐\n  ∨         │\n┌───┐     ┌───┐\n│ a │ ──> │ b │\n└───┘     └───┘\n"
    );
    assert_eq!(
        render(&cycle("dashed", "double")),
        "\n  ╔═════════╗\n  ∨         ║\n┌───┐     ┌───┐\n│ a │ ╴╴> │ b │\n└───┘     └───┘\n"
    );
}

/// Selfloop elbows: edge-style corners/hor/ver glyphs above the box;
/// an arrowless loop attaches the ver glyph on both sides.
fn assert_selfloop_elbows() {
    for (style, arrowed, want) in [
        (
            "solid",
            true,
            "\n  ┌──┐\n  ∨  │\n┌──────┐\n│  a   │\n└──────┘\n",
        ),
        (
            "double",
            true,
            "\n  ╔══╗\n  ∨  ║\n┌──────┐\n│  a   │\n└──────┘\n",
        ),
        (
            "solid",
            false,
            "\n  ┌──┐\n  │  │\n┌──────┐\n│  a   │\n└──────┘\n",
        ),
    ] {
        let mut g = Graph::default();
        let a = g.add_node("a");
        let e = g.add_edge(a, a, arrowed).expect("live endpoints");
        g.set_attr(Scope::Edge(e), "style", style);
        if !arrowed {
            g.edges[e].arrows.end = false;
        }
        assert_eq!(render(&g), want, "selfloop {style} arrowed={arrowed}");
    }
}

/// Bidirectional: arrowheads at both ends, gap one wider.
fn assert_bidirectional_arrowheads() {
    assert_eq!(
        render(&bidirectional()),
        "┌───┐      ┌───┐\n│ a │ <──> │ b │\n└───┘      └───┘\n"
    );
}

/// p3 (c3): each edge style renders with the exact upstream Unicode
/// glyphs — arrowed and arrowless — with repeat units spanning the same
/// column counts as upstream (the dot-dot-dash gap is one wider, and
/// labelled runs cycle the horizontal pattern).
#[test]
fn p3() {
    for (style, arrowed, want) in [
        ("solid", true, "┌───┐     ┌───┐\n│ a │ ──> │ b │\n└───┘     └───┘\n"),
        ("solid", false, "┌───┐     ┌───┐\n│ a │ ─── │ b │\n└───┘     └───┘\n"),
        ("double", true, "┌───┐     ┌───┐\n│ a │ ══> │ b │\n└───┘     └───┘\n"),
        ("double", false, "┌───┐     ┌───┐\n│ a │ ═══ │ b │\n└───┘     └───┘\n"),
        ("dotted", true, "┌───┐     ┌───┐\n│ a │ ··> │ b │\n└───┘     └───┘\n"),
        ("dotted", false, "┌───┐     ┌───┐\n│ a │ ··· │ b │\n└───┘     └───┘\n"),
        ("wave", true, "┌───┐     ┌───┐\n│ a │ ∼∼> │ b │\n└───┘     └───┘\n"),
        ("wave", false, "┌───┐     ┌───┐\n│ a │ ∼∼∼ │ b │\n└───┘     └───┘\n"),
        ("dashed", true, "┌───┐     ┌───┐\n│ a │ ╴╴> │ b │\n└───┘     └───┘\n"),
        ("dashed", false, "┌───┐     ┌───┐\n│ a │ ╴╴╴ │ b │\n└───┘     └───┘\n"),
        ("bold", true, "┌───┐     ┌───┐\n│ a │ ━━> │ b │\n└───┘     └───┘\n"),
        ("bold", false, "┌───┐     ┌───┐\n│ a │ ━━━ │ b │\n└───┘     └───┘\n"),
        ("wide", true, "┌───┐     ┌───┐\n│ a │ ██> │ b │\n└───┘     └───┘\n"),
        ("wide", false, "┌───┐     ┌───┐\n│ a │ ███ │ b │\n└───┘     └───┘\n"),
        ("broad", true, "┌───┐     ┌───┐\n│ a │ ▬▬> │ b │\n└───┘     └───┘\n"),
        ("broad", false, "┌───┐     ┌───┐\n│ a │ ▬▬▬ │ b │\n└───┘     └───┘\n"),
        ("dot-dash", true, "┌───┐     ┌───┐\n│ a │ ·-> │ b │\n└───┘     └───┘\n"),
        ("dot-dash", false, "┌───┐     ┌───┐\n│ a │ -·- │ b │\n└───┘     └───┘\n"),
        // dot-dot-dash: the gap column is one char wider (repeat unit).
        ("dot-dot-dash", true, "┌───┐      ┌───┐\n│ a │ ··-> │ b │\n└───┘      └───┘\n"),
        ("dot-dot-dash", false, "┌───┐      ┌───┐\n│ a │ -··- │ b │\n└───┘      └───┘\n"),
        ("double-dash", true, "┌───┐     ┌───┐\n│ a │ ═ > │ b │\n└───┘     └───┘\n"),
        ("double-dash", false, "┌───┐     ┌───┐\n│ a │  ═  │ b │\n└───┘     └───┘\n"),
        ("bold-dash", true, "┌───┐     ┌───┐\n│ a │ ━ > │ b │\n└───┘     └───┘\n"),
        ("bold-dash", false, "┌───┐     ┌───┐\n│ a │  ━  │ b │\n└───┘     └───┘\n"),
    ] {
        assert_eq!(render(&styled_edge(style, arrowed)), want, "edge {style} arrowed={arrowed}");
    }
    // labelled straight edges: the label hosts on the box-top row, the
    // run cycles the style's horizontal pattern plus the '>' head.
    for (style, want) in [
        (
            "solid",
            "┌───┐  go   ┌───┐\n│ a │ ────> │ b │\n└───┘       └───┘\n",
        ),
        (
            "dotted",
            "┌───┐  go   ┌───┐\n│ a │ ····> │ b │\n└───┘       └───┘\n",
        ),
        (
            "dot-dot-dash",
            "┌───┐  go    ┌───┐\n│ a │ ··-··> │ b │\n└───┘        └───┘\n",
        ),
    ] {
        let mut g = styled_edge(style, true);
        let e = g.edges.len() - 1;
        g.set_attr(Scope::Edge(e), "label", "go");
        assert_eq!(render(&g), want, "labelled edge {style}");
    }
}

/// p4 (c4): a node's shape attribute changes the outline per the upstream
/// Unicode shape table — `rounded` overlays the ╭╮╯╰ corner set for the
/// solid|dotted|dashed|dot-dash|dot-dot-dash border styles, blanks the
/// corners for bold|wide|broad|double|double-dash|bold-dash, and KEEPS
/// the corners for wave (upstream quirk); `point` swaps the label for a
/// centered ★; `invisible` draws nothing; every other vocabulary name
/// collapses to the plain box.
#[test]
fn p4() {
    for (shape, want) in [
        ("rect", "┌───┐\n│ x │\n└───┘\n"),
        ("rounded", "╭───╮\n│ x │\n╰───╯\n"),
        ("point", "\n  ★\n\n"),
        ("circle", "┌───┐\n│ x │\n└───┘\n"),
        ("ellipse", "┌───┐\n│ x │\n└───┘\n"),
        ("diamond", "┌───┐\n│ x │\n└───┘\n"),
        ("triangle", "┌───┐\n│ x │\n└───┘\n"),
        ("pentagon", "┌───┐\n│ x │\n└───┘\n"),
        ("hexagon", "┌───┐\n│ x │\n└───┘\n"),
        ("octagon", "┌───┐\n│ x │\n└───┘\n"),
        ("parallelogram", "┌───┐\n│ x │\n└───┘\n"),
        ("house", "┌───┐\n│ x │\n└───┘\n"),
        ("invisible", "\n\n\n"),
        ("img", "┌───┐\n│ x │\n└───┘\n"),
    ] {
        assert_eq!(render(&shaped_node(shape)), want, "shape {shape}");
    }
    // rounded × border style: overlay, blank, or keep (wave quirk).
    for (border, want) in [
        ("double", " ═══\n║ x ║\n ═══\n"),
        ("dotted", "╭⋯⋯⋯╮\n⋮ x ⋮\n╰⋯⋯⋯╯\n"),
        ("wave", "┌∼∼∼┐\n≀ x ≀\n└∼∼∼┘\n"),
        ("bold", " ━━━\n┃ x ┃\n ━━━\n"),
        ("dot-dot-dash", "╭·-·╮\n│ x │\n╰·-·╯\n"),
    ] {
        let mut g = bordered_node(border);
        let n = g.nodes.len() - 1;
        g.set_attr(Scope::Node(n), "shape", "rounded");
        assert_eq!(render(&g), want, "rounded + border {border}");
    }
}