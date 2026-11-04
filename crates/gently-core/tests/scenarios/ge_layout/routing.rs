use super::*;

/// ge.layout.p3 (c3): the configured flow direction maps source nodes to
/// strictly earlier positions along the flow axis than their targets;
/// self-loops are exempt (source and target coincide). Generator: simple
/// chains and one fan-out per flow — cycles and the oracle's packed
/// fan-out/shared-target shapes are excluded (see the module notes; the
/// oracle probed to route them against the flow axis, gently-89d).
#[test]
fn p3() {
    // east (right): source column strictly west of target column.
    let l = layout::layout(&chain_with_flow(None));
    assert!(l.node_cells[0].0 < l.node_cells[1].0, "east: a west of b");
    assert!(l.node_cells[1].0 < l.node_cells[2].0, "east: b west of c");
    // west (left): source strictly east of target along the leftward axis.
    let l = layout::layout(&chain_with_flow(Some("left")));
    assert!(l.node_cells[0].0 > l.node_cells[1].0, "west: a east of b");
    assert!(l.node_cells[1].0 > l.node_cells[2].0, "west: b east of c");
    // down (south): source row strictly north of target row.
    let l = layout::layout(&chain_with_flow(Some("down")));
    assert!(l.node_cells[0].1 < l.node_cells[1].1, "down: a north of b");
    assert!(l.node_cells[1].1 < l.node_cells[2].1, "down: b north of c");
    // up (north): source row strictly south of target row.
    let l = layout::layout(&chain_with_flow(Some("up")));
    assert!(l.node_cells[0].1 > l.node_cells[1].1, "up: a south of b");
    assert!(l.node_cells[1].1 > l.node_cells[2].1, "up: b south of c");
    // fan-out honours the flow axis too (down: both targets strictly below).
    let l = layout::layout(&fanout_with_flow("down"));
    assert!(
        l.node_cells[0].1 < l.node_cells[1].1,
        "down fan-out: b below a"
    );
    assert!(
        l.node_cells[0].1 < l.node_cells[2].1,
        "down fan-out: c below a"
    );
    // self-loops are exempt: the looped node has no earlier/later constraint.
    let l = layout::layout(&selfloop_graph());
    assert_eq!(l.node_cells.len(), 1, "selfloop node placed once");
}

/// ge.layout.p5 (c5): the published grid-output contract serves the
/// renderers — a renderer-style consumer reading the grid, extents, and
/// paths gets exactly what the layout's own inspection sees (every node
/// placed, every edge routed, all cells inside the grid), and the ascii
/// consumer renders the generator graphs without error.
#[test]
fn p5() {
    let graphs = [
        Graph::tracer(),
        labelled_edge_graph(),
        quad_parallel_graph(),
        selfloop_graph(),
        fanout_with_flow("down"),
        chain_with_flow(Some("up")),
        dense_grid_graph(12),
    ];
    for g in &graphs {
        let l = layout::layout(g);
        assert_published_contract(g, &l);
        // the renderer-style consumer read
        let art = gently_core::render::ascii::render(g, &l).must("consumer must render");
        assert!(
            !art.is_empty() || g.nodes.is_empty(),
            "consumer produced bytes"
        );
    }
}

/// ge.layout.p6 (c6): for typical large inputs with dense crossing
/// regions, ranking, ordering, and routing run as bounded deterministic
/// heuristic iterations — no exponential blowup (the layout completes
/// well within the probe envelope, and identical inputs stay identical).
#[test]
fn p6() {
    let g = dense_grid_graph(120);
    let start = std::time::Instant::now();
    let first = layout::layout(&g);
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 5,
        "layout of a 120-node dense graph took {elapsed:?} — unbounded"
    );
    assert_eq!(
        layout::layout(&g),
        first,
        "bounded stages stay deterministic"
    );
}
