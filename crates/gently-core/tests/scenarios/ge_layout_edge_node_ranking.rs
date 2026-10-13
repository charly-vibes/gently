//! ge.layout p4 additions (gently-pua): `shape: edge` node graphs rank the
//! box chain INLINE and drop side-entry leaves one band below — the
//! chain-based placement order (upstream Layout.pm's chain tracking:
//! chains from roots absorb already-chained suffixes and are laid out
//! root-chain first, longest first; a chain whose members were absorbed
//! re-ranks as a leftover leaf attaching to its placed successor).
//!
//! RED scaffold: the oracle test pins the re-derived oracle node dump
//! (Graph::Easy v0.69 @ ededa3d7, seed 0 — freshly re-derived for this
//! run and byte-identical to the bead's numbers); the companion test pins
//! the CURRENT wrong cells as RED-state documentation, mirroring the
//! gently-cbk error-pin pattern. The GREEN run replaces the current-wrong
//! pin's expectation with the oracle (fixing ranking, never suppressing).

use super::{layout, text};
use gently_core::layout::Layout;

/// The bloodgate tutorial_edges.html graph (gently-cbk's probe): two
/// `shape: edge` nodes (car, train) each reached by a labeled edge from
/// an anon node and embedded in the box chain.
///
/// Parse order (node index): car=0, train=1, rent-anon=2, buy-anon=3,
/// Bonn=4, Ulm=5, Hof=6. The two anon nodes share the empty name — they
/// are addressed by index (the model's only unique handle for them).
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

/// Node cells of `g` by node NAME for the uniquely-named nodes; the two
/// anon nodes are asserted by index with their identity proven via the
/// named neighbors they attach to.
fn cells_by_name(g: &gently_core::graph::Graph, l: &Layout) -> Vec<(String, (usize, usize))> {
    g.nodes()
        .iter()
        .enumerate()
        .map(|(i, n)| (n.name.clone(), l.node_cells[i]))
        .collect()
}

/// ge.layout.p4 (gently-pua, ORACLE): the chain Bonn–train–Ulm lands
/// inline on one row (Bonn west of the edge cell via the undirected `--`,
/// Ulm east via the directed `-->` on the SAME row) and the anon buy node
/// ranks one band BELOW train. Expected: the freshly re-derived oracle
/// dump (Graph::Easy v0.69 @ ededa3d7): rent-anon(0,0), car(2,0),
/// Hof(4,0), Bonn(0,2), train(2,2), Ulm(4,2), buy-anon(2,4).
#[test]
fn p4_edge_node_ranks_box_chain_inline_and_leaf_below() {
    let g = edge_node_graph();
    let l = layout::layout(&g);
    let cells = cells_by_name(&g, &l);
    let mut named: Vec<(String, (usize, usize))> = cells
        .iter()
        .filter(|(name, _)| !name.starts_with('#'))
        .cloned()
        .collect();
    named.sort();
    assert_eq!(
        named,
        vec![
            ("Bonn".to_string(), (0, 2)),
            ("Hof".to_string(), (4, 0)),
            ("Ulm".to_string(), (4, 2)),
            ("car".to_string(), (2, 0)),
            ("train".to_string(), (2, 2)),
        ],
        "named nodes must match the oracle dump"
    );
    // the anon pair, by index (rent-anon=2, buy-anon=3): rent west of car
    // on Hof's row, buy one band below train.
    assert_eq!(l.node_cells[2], (0, 0), "rent-anon west of car");
    assert_eq!(l.node_cells[3], (2, 4), "buy-anon below train");
}
