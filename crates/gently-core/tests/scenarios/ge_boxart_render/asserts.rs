use super::*;

/// The mixed-style chain: every edge cell carries its own style's glyphs.
pub(crate) fn assert_mixed_style_chain() {
    assert_eq!(
        render(&mixed_chain()),
        "┌───┐     ┌───┐     ┌───┐     ┌───┐\n\
         │ x │ ╴╴> │ a │ ══> │ b │ ··> │ c │\n\
         └───┘     └───┘     └───┘     └───┘\n"
    );
}

/// Border attachments keep the border's own glyphs (dashed ╎ / bold ┃);
/// a none-border box shows no glyphs at all.
pub(crate) fn assert_border_attachments_keep_their_glyphs() {
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
        let g = built(|g| {
            let a = g.add_node("a");
            let b = g.add_node("b");
            g.add_edge(a, b, true).must("live endpoints");
            g.set_attr(Scope::Node(a), "border", border);
        });
        assert_eq!(render(&g), want, "border {border} + solid edge");
    }
}

/// The none-border box with a straight edge: label column intact.
pub(crate) fn assert_none_border_box() {
    let g = built(|g| {
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.add_edge(a, b, true).must("live endpoints");
        g.set_attr(Scope::Node(a), "border", "none");
    });
    assert_eq!(render(&g), "        ┌───┐\n a  ──> │ b │\n        └───┘\n");
}

/// Bend (cycle) neighbourhoods: corners are the edge style's SE/SW
/// corner glyphs, verticals the ver glyphs, arrowheads '∨'.
pub(crate) fn assert_bend_corners() {
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
pub(crate) fn assert_selfloop_elbows() {
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
        let e = g.add_edge(a, a, arrowed).must("live endpoints");
        g.set_attr(Scope::Edge(e), "style", style);
        if !arrowed {
            g.edges[e].arrows.end = false;
        }
        assert_eq!(render(&g), want, "selfloop {style} arrowed={arrowed}");
    }
}

/// Bidirectional: arrowheads at both ends, gap one wider.
pub(crate) fn assert_bidirectional_arrowheads() {
    assert_eq!(
        render(&bidirectional()),
        "┌───┐      ┌───┐\n│ a │ <──> │ b │\n└───┘      └───┘\n"
    );
}

/// Build a `Graph` by applying `f` to a fresh default graph — keeps the
/// call sites' bindings immutable.
pub(crate) fn built(f: impl FnOnce(&mut Graph)) -> Graph {
    let mut g = Graph::default();
    f(&mut g);
    g
}

/// Same, from an existing graph.
pub(crate) fn with(mut g: Graph, f: impl FnOnce(&mut Graph)) -> Graph {
    f(&mut g);
    g
}
