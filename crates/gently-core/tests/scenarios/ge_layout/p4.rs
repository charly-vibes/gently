use super::*;

/// ge.layout.p4 (c4): every edge is routed along an orthogonal, connected
/// path of grid cells connecting its endpoints without entering a node
/// cell; the path passes through the label cell when the edge has a label;
/// parallel edges between the same node pair are routed through distinct
/// cells.
#[test]
fn p4() {
    // tracer: straight orthogonal route.
    let l = layout::layout(&Graph::tracer());
    assert_routed(&l, 0);

    // gently-awd: a size-attributed source's straight edge routes
    // through the single gap column EAST of its column span
    // (`[ A3 ] { size: 3,1; }` spans columns 0-2, B3 sits at column 4,
    // the edge cell rides column 3) and never enters any spanned cell.
    let mut g = Graph::default();
    let a3 = g.add_node("A3");
    let b3 = g.add_node("B3");
    g.add_edge(a3, b3, true).must("live endpoints");
    g.set_attr(Scope::Node(a3), "size", "3,1");
    let l = layout::layout(&g);
    assert_routed(&l, 0);
    assert_eq!(l.edge_paths[0], vec![(3, 0)], "gap cell east of the span");

    // labelled edge: the path passes through the label cell.
    let l = layout::layout(&labelled_edge_graph());
    assert_routed(&l, 0);
    let label = l.label_cells[0].must("labelled edge must publish a label cell");
    assert!(
        l.edge_paths[0].contains(&label),
        "label cell {label:?} must lie on the routed path"
    );
    // unlabelled edges publish no label cell.
    let l = layout::layout(&Graph::tracer());
    assert_eq!(l.label_cells[0], None, "unlabelled edge has no label cell");

    // self-loop: routed through its gap cell, never through the node.
    let l = layout::layout(&selfloop_graph());
    assert_routed(&l, 0);

    // four parallel edges: pairwise distinct path cells.
    assert_quad_parallel_distinct();
}
