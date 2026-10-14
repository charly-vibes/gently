//! ge.layout (gently-4nx): the deterministic grid layout contract — each
//! property row of specs/ge-layout.md is one test below, named for its id.
//!
//! Scope notes (probed, not assumed):
//! - The default (east) flow is byte-pinned by the recorded oracle corpus
//!   (tb.corpus); non-east flows and edge labels appear in no tier-1
//!   fixture, so they are free design here.
//! - specs/ge-layout.md c3 (re-derived in gently-89d): strict-earlier
//!   holds for acyclic, uniform-flow graphs; the oracle itself falsifies
//!   the unqualified claim for cycles (tests/repro/claims
//!   /layout-flow-direction.observed — the return edge routes against the
//!   flow) and for the packed fan-out/shared-target shapes the corpus
//!   pins (diamond: c below a; shared-target: b east of c). Those shapes
//!   are excluded from p3's generator, matching the amended row. The
//!   same amendment scoped c1 (tiered verification — the oracle's 5 s
//!   alarm) and c4 (within-envelope — the oracle silently drops paths at
//!   scale; tests/repro/claims/layout-edge-drop.observed).

use gently_core::graph::{Graph, Scope};
use gently_core::layout::{self, Layout};
use gently_core::parse::text;

/// The chain a -> b -> c, optionally carrying a graph-level `flow`.
fn chain_with_flow(flow: Option<&str>) -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    g.add_edge(a, b, true).expect("live endpoints");
    g.add_edge(b, c, true).expect("live endpoints");
    if let Some(f) = flow {
        g.set_attr(Scope::Graph, "flow", f);
    }
    g
}

/// A fan-out a -> b, a -> c under the given flow.
fn fanout_with_flow(flow: &str) -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    g.add_edge(a, b, true).expect("live endpoints");
    g.add_edge(a, c, true).expect("live endpoints");
    g.set_attr(Scope::Graph, "flow", flow);
    g
}

/// A labelled edge a -> b with label "go".
fn labelled_edge_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let e = g.add_edge(a, b, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e), "label", "go");
    g
}

/// The so-flowchart-size graph (gently-awd): Client/Msg Server carry
/// `{ size: 5,10; }` and the pair carries THREE edges — two east-going
/// Msg Server -> Client, one west-going Client -> Msg Server — in model
/// order (put msg, 1. Number, 3. X Msg, 2. read X).
fn flowchart_size_graph() -> Graph {
    let mut g = Graph::default();
    let client = g.add_node("Client");
    let msg = g.add_node("Msg Server");
    let server = g.add_node("Server");
    g.set_attr(Scope::Node(client), "size", "5,10");
    g.set_attr(Scope::Node(msg), "size", "5,10");
    let e0 = g.add_edge(server, msg, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e0), "label", "put msg\n via http");
    let e1 = g.add_edge(msg, client, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e1), "label", "1. Number X of unread");
    let e2 = g.add_edge(msg, client, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e2), "label", "3. X Msg");
    let e3 = g.add_edge(client, msg, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e3), "label", "2. read X");
    g
}

/// Four parallel edges a -> b (the pair carries a 4-way multi-edge).
fn quad_parallel_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    for _ in 0..4 {
        g.add_edge(a, b, true).expect("live endpoints");
    }
    g
}

/// A self-loop on a.
fn selfloop_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    g.add_edge(a, a, true).expect("live endpoints");
    g
}

/// The comma-chain fan (gently-4ga): a -> b, a -> c, a -> d. Placement
/// stacks c and d below a in one column, so the deeper edge's route must
/// jog around the shallower node instead of jumping its band row.
fn fanout_stack_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let d = g.add_node("d");
    g.add_edge(a, b, true).expect("live endpoints");
    g.add_edge(a, c, true).expect("live endpoints");
    g.add_edge(a, d, true).expect("live endpoints");
    g
}

/// The fan-in star (gently-6hh repro class, widened into gently-4ga):
/// n1..n4 -> hub — the deepest source's route walks a column stacked
/// with the shallower sources.
fn fanin_star_graph() -> Graph {
    let mut g = Graph::default();
    let n1 = g.add_node("n1");
    let n2 = g.add_node("n2");
    let n3 = g.add_node("n3");
    let n4 = g.add_node("n4");
    let hub = g.add_node("hub");
    for src in [n1, n2, n3, n4] {
        g.add_edge(src, hub, true).expect("live endpoints");
    }
    g
}

/// Edges `i -> i + step` for every i where both endpoints exist.
fn step_edges(g: &mut Graph, n: usize, step: usize) {
    for i in 0..n.saturating_sub(step) {
        g.add_edge(i, i + step, true).expect("live endpoints");
    }
}

/// A wide, dense grid-like graph: n nodes in a chain plus skip edges
/// (i -> i+2) — dense crossing regions for the bounded-stage probe.
fn dense_grid_graph(n: usize) -> Graph {
    let mut g = Graph::default();
    for i in 0..n {
        g.add_node(&format!("n{i}"));
    }
    step_edges(&mut g, n, 1);
    step_edges(&mut g, n, 2);
    g
}

/// No two node cells coincide.
fn assert_disjoint_cells(l: &Layout) {
    for i in 0..l.node_cells.len() {
        for j in i + 1..l.node_cells.len() {
            assert_ne!(
                l.node_cells[i], l.node_cells[j],
                "nodes {i} and {j} overlap"
            );
        }
    }
}

/// Every cell in `cells` lies inside the `width x height` grid.
fn assert_inside(cells: &[(usize, usize)], width: usize, height: usize) {
    for &(x, y) in cells {
        assert!(
            x < width && y < height,
            "cell ({x},{y}) outside {width}x{height}"
        );
    }
}

/// Every path step of edge `ei` is orthogonal and grid-adjacent.
fn assert_orthogonal(l: &Layout, ei: usize) {
    for w in l.edge_paths[ei].windows(2) {
        assert!(
            w[0].0 == w[1].0 || w[0].1 == w[1].1,
            "segment {:?}->{:?} not orthogonal",
            w[0],
            w[1]
        );
        let dist =
            (w[0].0 as isize - w[1].0 as isize).abs() + (w[0].1 as isize - w[1].1 as isize).abs();
        assert_eq!(dist, 1, "cells {:?}->{:?} not grid-adjacent", w[0], w[1]);
    }
}

/// No path cell of edge `ei` enters any node cell.
fn avoids_node_cells(l: &Layout, ei: usize) {
    for &c in &l.edge_paths[ei] {
        for &n in &l.node_cells {
            assert_ne!(c, n, "path of edge {ei} enters node cell {n:?}");
        }
    }
}

/// Full routing sanity for one edge: non-empty, orthogonal, node-free.
fn assert_routed(l: &Layout, ei: usize) {
    assert!(!l.edge_paths[ei].is_empty(), "edge {ei} must be routed");
    assert_orthogonal(l, ei);
    avoids_node_cells(l, ei);
}

/// Paths of edges `i` and `j` share no cell.
fn assert_disjoint_paths(l: &Layout, i: usize, j: usize) {
    for &c in &l.edge_paths[i] {
        assert!(
            !l.edge_paths[j].contains(&c),
            "parallel edges {i} and {j} share path cell {c:?}"
        );
    }
}

/// Every edge among the first `count` is routed sanely.
fn assert_all_routed(l: &Layout, count: usize) {
    for ei in 0..count {
        assert_routed(l, ei);
    }
}

/// The first `count` paths are pairwise cell-disjoint.
fn assert_parallel_pairwise(l: &Layout, count: usize) {
    for i in 0..count {
        for j in i + 1..count {
            assert_disjoint_paths(l, i, j);
        }
    }
}

/// Four parallel edges a -> b: all routed, pairwise distinct cells.
fn assert_quad_parallel_distinct() {
    let l = layout::layout(&quad_parallel_graph());
    assert_all_routed(&l, 4);
    assert_parallel_pairwise(&l, 4);
}

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
    cyclic.add_edge(b, a, true).expect("live endpoints");
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
        wide.add_edge(prev, node, true).expect("live endpoints");
    }
    let dense = dense_grid_graph(10);
    for g in [&wide, &dense] {
        let l = layout::layout(g);
        assert_disjoint_cells(&l);
        assert_inside(&l.node_cells, l.width, l.height);
        assert_eq!(l.node_cells.len(), g.nodes.len(), "every node placed");
    }
}

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
    g.add_edge(a3, b3, true).expect("live endpoints");
    g.set_attr(Scope::Node(a3), "size", "3,1");
    let l = layout::layout(&g);
    assert_routed(&l, 0);
    assert_eq!(l.edge_paths[0], vec![(3, 0)], "gap cell east of the span");

    // labelled edge: the path passes through the label cell.
    let l = layout::layout(&labelled_edge_graph());
    assert_routed(&l, 0);
    let label = l.label_cells[0].expect("labelled edge must publish a label cell");
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
            assert_adjacent(path[0], su, "edge {ei} first path cell");
            assert_adjacent(path[path.len() - 1], sv, "edge {ei} last path cell");
        }
        gently_core::render::ascii::render(g, &l).expect("fan must render");
    }
}

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
    gently_core::render::ascii::render(&g, &l).expect("flowchart must render");
}

/// The two cells differ by exactly one step on one axis.
fn assert_adjacent(p: (usize, usize), q: (usize, usize), what: &str) {
    let dist = (p.0 as isize - q.0 as isize).abs() + (p.1 as isize - q.1 as isize).abs();
    assert_eq!(dist, 1, "{what} {p:?} not adjacent to {q:?}");
}

/// The layout publishes a complete grid-output contract: every node
/// placed, every edge routed, all cells inside the grid extents.
fn assert_published_contract(g: &Graph, l: &Layout) {
    assert_eq!(l.node_cells.len(), g.nodes.len(), "every node placed");
    assert_eq!(l.edge_paths.len(), g.edges.len(), "every edge routed");
    assert_inside(&l.node_cells, l.width, l.height);
    for path in &l.edge_paths {
        assert_inside(path, l.width, l.height);
    }
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
        let art = gently_core::render::ascii::render(g, &l).expect("consumer must render");
        assert!(
            !art.is_empty() || g.nodes.is_empty(),
            "consumer produced bytes"
        );
    }
}

/// ge.layout.p4 additions (gently-pua): `shape: edge` node graphs — the
/// chain-based placement order (box chains inline, side-entry leaves one
/// band below). Side file; keeps the `p4` test-filter prefix.
#[path = "ge_layout_edge_node_ranking.rs"]
mod p4_edge_node;

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
