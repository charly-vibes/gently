//! ge.ascii_render p2 additions (gently-cbk): `shape: edge` nodes render
//! as edge cells — a full-pipeline feature pinned from the recorded oracle
//! bytes. GREEN: the oracle-bytes test below pins the render byte-identical
//! against tests/repro/claims/bg-edge-node.observed (Graph::Easy v0.69 @
//! ededa3d7, seed 0; seeds 0 and 1 are byte-identical). The oracle ranks
//! the edge-shaped nodes INLINE on their edges' rows (Bonn, train, Ulm on
//! one row; car on Hof's row) and strips start/end flags: the dash run
//! continues THROUGH the edge cell (Bonn --> train --> Ulm draws ONE line
//! with a single `>` at Ulm), the node's label rides above the cell line
//! (`train`, `rent`, `car` — left-aligned on the fresh edge's `align`
//! default), and the anon cross node #3 ranks BELOW train with `buy`
//! entering train's south side (`^`).

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

/// ge.ascii_render.p2 (gently-cbk GREEN, placement level): the render
/// completes — the gently-pua ranking fix removed the orthogonality
/// geometry error and cbk's edge-cell render removed the byte divergence —
/// so this pin keeps the placement-level oracle truth (the chain
/// Bonn–train–Ulm inline, the buy leaf one band below train) as a cheap
/// diagnostic beside the byte-identical pin below.
#[test]
fn p2_edge_node_ranks_inline_with_the_buy_leaf_below() {
    let g = edge_node_graph();
    let l = layout::layout(&g);
    ascii::render(&g, &l).expect("the ranked layout must render — no geometry error");
    // placement truth (gently-pua's oracle re-derivation): the chain
    // Bonn–train–Ulm inline, the buy leaf one band below train.
    let by_name: Vec<(String, (usize, usize))> = g
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.name.starts_with('#'))
        .map(|(i, n)| (n.name.clone(), l.node_cells[i]))
        .collect();
    // the ge_layout oracle pin asserts in name order
    let by_name = { let mut v = by_name; v.sort(); v };
    assert_eq!(
        by_name,
        vec![
            ("Bonn".to_string(), (0, 2)),
            ("Hof".to_string(), (4, 0)),
            ("Ulm".to_string(), (4, 2)),
            ("car".to_string(), (2, 0)),
            ("train".to_string(), (2, 2)),
        ]
    );
}

/// ge.ascii_render.p2 (gently-cbk GREEN): the full-pipeline render —
/// edge-shaped nodes claim their cell as an EDGE_HOR + EDGE_LABEL_CELL
/// edge cell, the dash run continues through it (endpoint flags stripped
/// Scout.pm-style), the label rides above the cell line, and the anon
/// cross node ranks below. Expected bytes:
/// tests/repro/claims/bg-edge-node.observed (seed 0, hash-stable across
/// 2 seeds).
#[test]
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
