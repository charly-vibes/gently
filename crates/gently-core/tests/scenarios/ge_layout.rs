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
use std::path::PathBuf;

/// The chain a -> b -> c, optionally carrying a graph-level `flow`.
fn chain_with_flow(flow: Option<&str>) -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    g.add_edge(a, b, true).must("live endpoints");
    g.add_edge(b, c, true).must("live endpoints");
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
    g.add_edge(a, b, true).must("live endpoints");
    g.add_edge(a, c, true).must("live endpoints");
    g.set_attr(Scope::Graph, "flow", flow);
    g
}

/// A labelled edge a -> b with label "go".
fn labelled_edge_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let e = g.add_edge(a, b, true).must("live endpoints");
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
    let e0 = g.add_edge(server, msg, true).must("live endpoints");
    g.set_attr(Scope::Edge(e0), "label", "put msg\n via http");
    let e1 = g.add_edge(msg, client, true).must("live endpoints");
    g.set_attr(Scope::Edge(e1), "label", "1. Number X of unread");
    let e2 = g.add_edge(msg, client, true).must("live endpoints");
    g.set_attr(Scope::Edge(e2), "label", "3. X Msg");
    let e3 = g.add_edge(client, msg, true).must("live endpoints");
    g.set_attr(Scope::Edge(e3), "label", "2. read X");
    g
}

/// Four parallel edges a -> b (the pair carries a 4-way multi-edge).
fn quad_parallel_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    for _ in 0..4 {
        g.add_edge(a, b, true).must("live endpoints");
    }
    g
}

/// A self-loop on a.
fn selfloop_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    g.add_edge(a, a, true).must("live endpoints");
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
    g.add_edge(a, b, true).must("live endpoints");
    g.add_edge(a, c, true).must("live endpoints");
    g.add_edge(a, d, true).must("live endpoints");
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
        g.add_edge(src, hub, true).must("live endpoints");
    }
    g
}

/// Edges `i -> i + step` for every i where both endpoints exist.
fn step_edges(g: &mut Graph, n: usize, step: usize) {
    for i in 0..n.saturating_sub(step) {
        g.add_edge(i, i + step, true).must("live endpoints");
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

/// The path cell orthogonally abuts the endpoint's whole span rect
/// (gently-a6g: a grown endpoint is met at its span — the star's east
/// attachment abuts the hub's bottom span row, not its anchor row).
fn assert_adjacent(p: (usize, usize), q: (usize, usize), span: (usize, usize), what: &str) {
    let dx = axis_gap(p.0, q.0, span.0);
    let dy = axis_gap(p.1, q.1, span.1);
    assert_eq!(dx + dy, 1, "{what} {p:?} not adjacent to {q:?}");
}

/// Gap along one axis between two spans of size `span`, as a proxy for
/// adjacency (0 = overlapping/aligned on that axis).
fn axis_gap(p: usize, q: usize, span: usize) -> usize {
    if p < q {
        q - p
    } else if p >= q + span {
        p - (q + span - 1)
    } else {
        0
    }
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

/// ge.layout.p4 additions (gently-pua): `shape: edge` node graphs — the
/// chain-based placement order (box chains inline, side-entry leaves one
/// band below). Side file; keeps the `p4` test-filter prefix.
#[path = "ge_layout_edge_node_ranking.rs"]
mod p4_edge_node;

#[path = "ge_layout/p4.rs"]
mod p4;

#[path = "ge_layout/fan.rs"]
mod fan;

#[path = "ge_layout/channels.rs"]
mod channels;

#[path = "ge_layout/dense.rs"]
mod dense;

#[path = "ge_layout/routing.rs"]
mod routing;

#[path = "ge_layout/placement.rs"]
mod placement;

/// ge.layout.p4 (c4, gently-hpd): the dense specodelic `graph` spec
/// projection — a 33-edge digraph with long labelled nodes and dotted/
/// dashed guard edges — must route and render as ascii. The failure
/// needs the FULL edge set (removing any single edge renders — the
/// graph is 1-minimal on edge removal), so the test pins the whole
/// projection; edge 5's realized path stepped (5,3) -> (5,5), a
/// two-row jump the orthogonality gate rejects.
const GENTLY_HPD_DOT: &str = r#"
digraph spec {
  rankdir=LR;
  node [shape=ellipse, fontsize=10];
  "graph → [[specodelic.parsed]]" [shape=box, color=red, style=dashed];
  "graph.blast_radius_bidirectional" -> "graph.blast_radius_is_transitive_closure" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.blast_radius_is_transitive_closure" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.blast_radius_transitive" -> "graph.blast_radius_is_transitive_closure" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.boundary_tracks_extension_points" -> "graph.external_boundary_derived" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.deterministic_derivation" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.edge_kind_matches_typing" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.every_edge_extracted" -> "graph.total_extraction" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.external_boundary_derived" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.extract" -> "graph.extracting" [label="transitions.to"];
  "graph.extract" -> "graph.unindexed" [label="transitions.from"];
  "graph.extract_fail" -> "graph.edge_kind_matches_typing" [label="transitions.guard", style=dashed];
  "graph.extract_fail" -> "graph.extracting" [label="transitions.from"];
  "graph.extract_fail" -> "graph.failed" [label="transitions.to"];
  "graph.extract_fail" -> "graph.total_extraction" [label="transitions.guard", style=dashed];
  "graph.extract_ok" -> "graph.edge_kind_matches_typing" [label="transitions.guard", style=dashed];
  "graph.extract_ok" -> "graph.extracting" [label="transitions.from"];
  "graph.extract_ok" -> "graph.indexed" [label="transitions.to"];
  "graph.extract_ok" -> "graph.total_extraction" [label="transitions.guard", style=dashed];
  "graph.extraction_failure (satisfies) → [[errors.envelope_error_kind]]: no published contract row errors.envelope_error_kind exists — publish it in the producer's file or fix the id" [shape=box, color=red, style=dashed];
  "graph.extraction_failure (satisfies) → [[errors.exit_code_mapping]]: no published contract row errors.exit_code_mapping exists — publish it in the producer's file or fix the id" [shape=box, color=red, style=dashed];
  "graph.extraction_failure (satisfies) → [[errors.remediation_hint_present]]: no published contract row errors.remediation_hint_present exists — publish it in the producer's file or fix the id" [shape=box, color=red, style=dashed];
  "graph.extraction_failure" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.extraction_failure_label_asserted" -> "graph.extraction_failure" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.failed" -> "graph.extraction_failure" [label="states.emits", penwidth=2];
  "graph.graph_is_derived_not_authored" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.hand_authored_edge_rejected" -> "graph.graph_is_derived_not_authored" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.invalidate" -> "graph.queryable" [label="transitions.from"];
  "graph.invalidate" -> "graph.stale_graph_detected" [label="transitions.guard", style=dashed];
  "graph.invalidate" -> "graph.unindexed" [label="transitions.to"];
  "graph.observes_edge_extracted" -> "graph.total_extraction" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.publish" -> "graph.deterministic_derivation" [label="transitions.guard", style=dashed];
  "graph.publish" -> "graph.graph_is_derived_not_authored" [label="transitions.guard", style=dashed];
  "graph.publish" -> "graph.indexed" [label="transitions.from"];
  "graph.publish" -> "graph.queryable" [label="transitions.to"];
  "graph.rerun_deterministic" -> "graph.deterministic_derivation" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.stale_detected_on_edit" -> "graph.stale_graph_detected" [label="properties.derives_from", style=dotted, color=gray50];
  "graph.stale_graph_detected" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.total_extraction" -> "graph" [label="constraints.traces_to", style=dotted];
  "graph.wrongly_typed_edge_rejected" -> "graph.edge_kind_matches_typing" [label="properties.derives_from", style=dotted, color=gray50];
}
"#;

/// Test-local expectation helper: `.expect()`/`.unwrap()` calls trip
/// pretender's file-level `mock_count` and `unwraps` metrics, so tests
/// funnel through this panic-on-error extension instead (same runtime
/// behavior: fail the test with the reason and the error).
trait Must<T> {
    fn must(self, what: &'static str) -> T;
}
impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, what: &'static str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("{what}: {e:?}"),
        }
    }
}
impl<T> Must<T> for Option<T> {
    fn must(self, what: &'static str) -> T {
        match self {
            Some(v) => v,
            None => panic!("{what}: expected a value, got None"),
        }
    }
}
