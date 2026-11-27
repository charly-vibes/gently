//! gently-481: directly-adjacent vertical edges must route a real path,
//! never the empty path the polyline renderer reports as a typed error.
//! The slot accounting skips gap rows when a band rides inside a grown
//! span (gently-a6g); when that ride falls back to the cursor the pair
//! packs adjacent and the vertical walk has no interior cell — the
//! router must fall back to a side-face lane.

use gently_core::graph::{Graph, Scope};

trait Must<T> {
    fn must(self, what: &'static str) -> T;
}
impl<T> Must<T> for Option<T> {
    fn must(self, what: &'static str) -> T {
        self.expect(what)
    }
}
impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, what: &'static str) -> T {
        self.expect(what)
    }
}

fn paths_all_nonempty(l: &gently_core::layout::Layout) -> bool {
    l.edge_paths.iter().all(|p| !p.is_empty())
}

/// The ticket repro: a grown fan-in hub whose hosting ride failed, the
/// chain packed onto the cursor one row south of the hub's span.
#[test]
fn adjacent_stack_fan_in_routes_side_lane() {
    let mut g = Graph::default();
    let d = g.add_node("d");
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let p = g.add_node("p");
    let q = g.add_node("q");
    g.add_edge(p, q, true).must("live endpoints");
    g.add_edge(q, d, true).must("live endpoints");
    g.add_edge(a, d, true).must("live endpoints");
    g.add_edge(b, d, true).must("live endpoints");
    g.add_edge(c, d, true).must("live endpoints");
    let l = gently_core::layout::layout(&g);
    assert!(paths_all_nonempty(&l), "every edge needs a routed path");
}

/// A tall (3-row) source stacked directly above its target: the walk's
/// effective start (`walk_from`) is the span's bottom row, adjacent to
/// the target's top row — the same empty-path family.
#[test]
fn tall_source_adjacent_stack_routes_side_lane() {
    let mut g = Graph::default();
    let q = g.add_node("q");
    let x = g.add_node("x");
    let d = g.add_node("d");
    g.add_edge(q, x, true).must("live endpoints");
    g.add_edge(q, d, true).must("live endpoints");
    g.set_attr(Scope::Node(q), "size", "1,3");
    let l = gently_core::layout::layout(&g);
    assert!(paths_all_nonempty(&l), "every edge needs a routed path");
}
