use super::*;

/// ge.layout.p1 (c1): the layout is a pure function of the model and
/// options — identical inputs produce byte-identical grid outputs, across
/// shapes and configured flows.
#[test]
fn p1() {
    let mut cases: Vec<Graph> = vec![
        Graph::tracer(),
        chain_with_flow(None),
        labelled_edge_graph(),
        quad_parallel_graph(),
        selfloop_graph(),
        dense_grid_graph(12),
        fanout_with_flow("down"),
        chain_with_flow(Some("left")),
        chain_with_flow(Some("up")),
    ];
    let mut cyclic = chain_with_flow(None);
    let a = cyclic.add_node("z0");
    let b = cyclic.add_node("z1");
    cyclic.add_edge(b, a, true).must("live endpoints");
    cases.push(cyclic);
    for g in &cases {
        assert_eq!(
            layout::layout(g),
            layout::layout(g),
            "layout must be deterministic"
        );
    }
}

/// ge.layout.p2 (c2): no two node cells overlap and every node is fully
/// contained inside the grid — on dense and wide inputs.
#[test]
fn p2() {
    let mut wide = Graph::default();
    wide.add_node("n0");
    for i in 1..8 {
        let prev = i - 1;
        let node = wide.add_node(&format!("n{i}"));
        wide.add_edge(prev, node, true).must("live endpoints");
    }
    let dense = dense_grid_graph(10);
    for g in [&wide, &dense] {
        let l = layout::layout(g);
        assert_disjoint_cells(&l);
        assert_inside(&l.node_cells, l.width, l.height);
        assert_eq!(l.node_cells.len(), g.nodes.len(), "every node placed");
    }
}

/// ge.layout.p2 (c2, gently-0is): the raw-scale case — the specodelic
/// whole-corpus DOT projection (1311 nodes, 1634 edges, tier-3 per
/// [[ge.oracle]] c7: gently-only invariants, no oracle pin). At corpus
/// scale the fan-in ring-ride resolved four bands onto two grid rows:
/// two nodes shared the hub's cell and the render died with 'edge 70
/// enters its box through an unsupported side'. c2 must hold at any
/// scale, and the render that consumes the grid must succeed.
#[test]
fn p2_corpus_scale_no_overlap() {
    let dot = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/corpus/specodelic-corpus.dot"),
    )
    .must("fixture tests/fixtures/corpus/specodelic-corpus.dot");
    let g = gently_core::parse::dot::parse(&dot).must("corpus projection must parse");
    let l = layout::layout(&g);
    assert_disjoint_cells(&l);
    assert_inside(&l.node_cells, l.width, l.height);
    assert_eq!(
        l.node_cells.len(),
        g.nodes.len(),
        "corpus: every node placed"
    );
    assert!(l.width > 0 && l.height > 0, "corpus grid is non-empty");
    gently_core::render::ascii::render(&g, &l)
        .must("the corpus render that consumes the grid must succeed");
}
