use super::*;

/// ge.layout.p4 (c4, gently-awd): parallel channels between MULTI-ROW
/// node pairs stack as grid ROWS through the boxes' row span — the
/// pair's first edge keeps the straight band-row run, channel k rides
/// the band's continuation row k (upstream places each parallel edge's
/// cell on the next free row of the span; the so-flowchart-size oracle
/// render: three straight runs at distinct heights, the box grown to
/// its full 10-row span). One-row pairs keep the recorded bend shape
/// (parallel.txt: channel 1 bends ABOVE the band).
#[test]
fn p4_parallel_channels_stack_rows() {
    let g = flowchart_size_graph();
    let l = layout::layout(&g);
    assert_all_routed(&l, g.edges.len());
    // Placement: one band, Server col0, Msg Server cols 2-6, Client
    // cols 8-12 (the recorded oracle placement, seed-stable).
    assert_eq!(l.node_cells[0], (8, 0), "Client");
    assert_eq!(l.node_cells[1], (2, 0), "Msg Server");
    assert_eq!(l.node_cells[2], (0, 0), "Server");
    // The channels: e1 (pair's first edge) stays straight on the band
    // row; e2/e3 stack on the continuation rows 1 and 2 — single gap
    // cells east of Msg Server's span (column 7) / west of Client's.
    assert_eq!(
        l.edge_paths[1],
        vec![(7, 0)],
        "channel 0 stays on the band row"
    );
    assert_eq!(
        l.edge_paths[2],
        vec![(7, 1)],
        "channel 1 rides continuation row 1"
    );
    assert_eq!(
        l.edge_paths[3],
        vec![(7, 2)],
        "channel 2 rides continuation row 2"
    );
    gently_core::render::ascii::render(&g, &l).must("flowchart must render");
}
