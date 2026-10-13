//! ge.ascii_render p2 additions (gently-cbk): `shape: edge` nodes render
//! as edge cells — a full-pipeline feature pinned from the recorded oracle
//! bytes. RED scaffold: the reproduction test below pins the CURRENT typed
//! error (the orthogonality geometry rejects the edge-node's merged path);
//! the oracle-bytes test is `#[ignore]`d until the GREEN run replaces the
//! error with the real render — never by suppressing the error.
//!
//! Expected bytes: tests/repro/claims/bg-edge-node.observed (Graph::Easy
//! v0.69 @ ededa3d7, seed 0; seeds 0 and 1 are byte-identical). The oracle
//! ranks the edge-shaped nodes INLINE on their edges' rows (Bonn, train,
//! Ulm on one row; car on Hof's row) and strips start/end flags: the dash
//! run continues THROUGH the edge cell (Bonn --> train --> Ulm draws ONE
//! line with a single `>` at Ulm), the node's label rides above the cell
//! line (`train`, `rent`, `car`), and the anon cross node #3 ranks BELOW
//! train with `buy` entering train's south side (`^`).

use super::{ascii, layout, text};

/// The bloodgate tutorial_edges.html graph: two `shape: edge` nodes (car,
/// train) each reached by a labeled edge from an anon node and embedded in
/// the box chain.
fn edge_node_graph() -> gently_core::graph::Graph {
    text::parse(concat!(
        "[ car ] { shape: edge; }\n",
        "[ train ] { shape: edge; }\n",
        "\n",
        "[ ] -- rent --> [ car ]\n",
        "[ ] -- buy --> [ train ]\n",
        "[ Bonn ] -- [ train ]\n",
        "[ train ] --> [ Ulm ]\n",
        "[ Ulm ] -- [ car ]\n",
        "[ car ] --> [ Hof ]\n",
    ))
    .expect("edge-node graph must parse")
}

/// ge.ascii_render.p2 (gently-cbk, RED state): the current renderer typed
/// error — the edge-node's cell merges into the edge path and the path
/// skips a row, so the orthogonality check rejects it. This test pins the
/// error's existence and shape so the GREEN run must REPLACE it with the
/// oracle render (fix the geometry, never suppress the error).
#[test]
fn p2_edge_node_currently_errors_on_path_orthogonality() {
    let g = edge_node_graph();
    let err = ascii::render(&g, &layout::layout(&g))
        .expect_err("shape:edge nodes are not implemented yet — RED state");
    assert!(
        err.to_string().contains("not orthogonally adjacent"),
        "the RED-state error must stay the orthogonality geometry error, got: {err}"
    );
}

/// ge.ascii_render.p2 (gently-cbk, GREEN target): the full-pipeline render
/// per the re-derivation — edge-shaped nodes claim their cell as an
/// EDGE_HOR + EDGE_LABEL_CELL edge cell, the dash run continues through
/// it (endpoint flags stripped Scout.pm-style), the label rides above the
/// cell line, and the anon cross node ranks below. Expected bytes:
/// tests/repro/claims/bg-edge-node.observed (seed 0, hash-stable across
/// 2 seeds). `#[ignore]`d while the feature is RED — un-ignore in the
/// GREEN commit together with the placement/render implementation.
#[test]
#[ignore = "gently-cbk RED scaffold: shape:edge is unimplemented (errors today)"]
fn p2_edge_node_renders_as_edge_cell_with_label_above() {
    let g = edge_node_graph();
    let art = ascii::render(&g, &layout::layout(&g)).expect("edge-node graph must render");
    assert_eq!(
        art,
        concat!(
            "          rent    car          +-----+\n",
            "         ------> ------------> | Hof |\n",
            "                               +-----+\n",
            "\n",
            "                 ----------------+\n",
            "                                 |\n",
            "+------+          train        +-----+\n",
            "| Bonn | --------------------> | Ulm |\n",
            "+------+                       +-----+\n",
            "                   ^\n",
            "                   | buy\n",
            "                   |\n",
        ),
        "shape:edge nodes render as edge cells with the label above"
    );
}
