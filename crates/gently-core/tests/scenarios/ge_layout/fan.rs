use super::*;

/// ge.layout.p4 (c4, gently-4ga): fan placements stack nodes in one
/// column, and the deeper stacked edge's route must jog around the
/// shallower node through the adjacent gap column — contiguous and
/// grid-adjacent end to end, never jumping the occupied band row, and
/// the ascii consumer renders the fans without error.
#[test]
fn p4_fan_stack_routes_jog_not_jump() {
    for g in [&fanout_stack_graph(), &fanin_star_graph()] {
        let l = layout::layout(g);
        assert_all_routed(&l, g.edges.len());
        // Endpoint adjacency too: the render walks source cell -> path ->
        // target cell, so the first/last path cell must abut its node.
        for (ei, e) in g.edges.iter().enumerate() {
            let path = &l.edge_paths[ei];
            let (su, sv) = (l.node_cells[e.from], l.node_cells[e.to]);
            assert_adjacent(path[0], su, l.spans[e.from], "edge {ei} first path cell");
            assert_adjacent(
                path[path.len() - 1],
                sv,
                l.spans[e.to],
                "edge {ei} last path cell",
            );
        }
        assert_eq!(l.edge_paths.len(), g.edges.len(), "every edge has a route");
        gently_core::render::ascii::render(g, &l).must("fan must render");
    }
}

/// ge.layout.p4 (c4, gently-a6g): the fan-in star's placement matches the
/// pinned oracle (tests/repro/claims/fan-layout.observed, example 2 —
/// hash-stable across 12 seeds, tier-1): the hub GROWS to a 3-row span
/// (four same-side attachments), the first attaching source takes the
/// east ring slot at the hub's top row, the second slots two rows south
/// (the adjacent row is cleared against the first attachment), the third
/// hangs below the hub on its own band, and the chain head stays west.
#[test]
fn p4_fanin_star_places_east_ring_slots_on_grown_hub() {
    let g = fanin_star_graph();
    let l = layout::layout(&g);
    // n1=0, n2=1, n3=2, n4=3, hub=4 — the oracle's absolute grid cells:
    // hub at (2,0) grown to 3 rows, n1 west, n4/n2 on the east column at
    // the hub's top/bottom rows, n3 below the hub's bottom row + 1 gap.
    assert_eq!(l.node_cells[0], (0, 0), "n1 west of the hub");
    assert_eq!(l.node_cells[1], (4, 2), "n2 east ring slot at row 2");
    assert_eq!(l.node_cells[2], (2, 4), "n3 below the hub band");
    assert_eq!(l.node_cells[3], (4, 0), "n4 east ring slot at row 0");
    assert_eq!(l.node_cells[4], (2, 0), "hub chained east of n1");
    assert_eq!(l.spans[4], (1, 3), "hub grown to a 3-row span");
    assert_all_routed(&l, g.edges.len());
    let art = gently_core::render::ascii::render(&g, &l).must("star must render");
    assert_eq!(
        art,
        concat!(
            "+----+     +-----+     +----+\n",
            "| n1 | --> |     | <-- | n4 |\n",
            "+----+     | hub |     +----+\n",
            "           |     |     +----+\n",
            "           |     | <-- | n2 |\n",
            "           +-----+     +----+\n",
            "             ^\n",
            "             |\n",
            "             |\n",
            "           +-----+\n",
            "           | n3  |\n",
            "           +-----+\n",
        ),
        "the star's oracle bytes (fan-layout.observed example 2)"
    );
}

/// A fan-in star with `n` sources -> hub, each edge optionally labelled
/// (gently-n0d / gently-206 repro family).
fn fanin_star_n(n: usize, label: Option<&str>) -> Graph {
    let mut g = Graph::default();
    let hub = g.add_node("hub");
    for i in 0..n {
        let s = g.add_node(&format!("s{}", i + 1));
        let e = g.add_edge(s, hub, true).must("live endpoints");
        if let Some(l) = label {
            g.set_attr(Scope::Edge(e), "label", l);
        }
    }
    g
}

/// ge.layout.p4 (c4, gently-n0d): a labelled fan-in FIVE star must not
/// panic — the hub's growth gives the fourth attacher a below-slot at
/// the hub's bottom span row in the SAME column (band-adjacent), which
/// multiband_path realized as a ZERO-cell path; label_cells then
/// underflowed indexing `path[(path.len()-1)/2]` (layout.rs:468).
/// Every edge must be routed (non-empty, orthogonal) and the ascii
/// consumer must render without error — a labelled edge whose routing
/// degenerates may not crash the layout, it must degrade like the
/// unlabelled case (the renderer's own 'edge N has an empty path'
/// guard was the only thing standing between this shape and the panic).
#[test]
fn p4_fanin5_labelled_star_routes_and_renders() {
    let g = fanin_star_n(5, Some("x"));
    let l = layout::layout(&g);
    assert_all_routed(&l, g.edges.len());
    for (ei, e) in g.edges.iter().enumerate() {
        let path = &l.edge_paths[ei];
        let (su, sv) = (l.node_cells[e.from], l.node_cells[e.to]);
        assert_adjacent(path[0], su, l.spans[e.from], "edge {ei} first path cell");
        assert_adjacent(
            path[path.len() - 1],
            sv,
            l.spans[e.to],
            "edge {ei} last path cell",
        );
    }
    gently_core::render::ascii::render(&g, &l).must("labelled fan-in 5 must render");
    assert_eq!(l.node_cells.len(), g.nodes.len(), "every star node placed");
}

/// ge.layout.p4 (c4, gently-206): the fan-in EIGHT star (the minimized
/// specodelic wiring-hub shape — 16 labelled edges converging on one
/// file node) must lay out and render without error. The >=5 attacher
/// ring previously under-routed: the below-slot edge realized as a
/// zero-cell path and the later ring attachers as long non-axis
/// turning walks the renderer rejected ('enters its box through an
/// unsupported side' / 'turns without changing axis').
#[test]
fn p4_fanin8_star_routes_and_renders() {
    for label in [None, Some("x")] {
        let g = fanin_star_n(8, label);
        let l = layout::layout(&g);
        assert_all_routed(&l, g.edges.len());
        for (ei, e) in g.edges.iter().enumerate() {
            let path = &l.edge_paths[ei];
            let (su, sv) = (l.node_cells[e.from], l.node_cells[e.to]);
            assert_adjacent(path[0], su, l.spans[e.from], "edge {ei} first path cell");
            assert_adjacent(
                path[path.len() - 1],
                sv,
                l.spans[e.to],
                "edge {ei} last path cell",
            );
        }
        gently_core::render::ascii::render(&g, &l).must("fan-in 8 must render");
        assert_eq!(l.node_cells.len(), g.nodes.len(), "every star node placed");
    }
}
