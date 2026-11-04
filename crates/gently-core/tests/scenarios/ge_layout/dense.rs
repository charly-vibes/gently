use super::*;

#[test]
fn p4_dense_spec_projection_routes_and_renders() {
    let g = gently_core::parse::dot::parse(GENTLY_HPD_DOT).must("dense projection must parse");
    // 36 edge statements today — the ticket's "33 edges" was filed against
    // the corpus before three later spec rows landed; the failing error
    // (edge 5 path cells (5, 3) -> (5, 5)) is byte-identical to the filed
    // one, so the dense-set shape is the same.
    assert_eq!(
        g.edges.len(),
        36,
        "the full dense set — ticket is 1-minimal"
    );
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
    gently_core::render::ascii::render(&g, &l).must("dense spec projection must render");
}
