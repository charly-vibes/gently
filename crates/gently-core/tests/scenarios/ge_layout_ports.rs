//! ge.layout port routing (gently-89r): the `start:`/`end:` port side
//! attributes — each pinned example is its own graph, byte-pinned by
//! tests/repro/claims/port-side-minimal.observed and the recorded
//! port-* fixtures (Graph::Easy v0.69 @ ededa3d7).
//!
//! Scope notes (probed, not assumed):
//! - DIS-ENTANGLED from the shared-middle bypass shape (gently-jrr): the
//!   no-port chain `[Bonn] --> [Koblenz] ... [Trier] --> [Frankfurt]`
//!   already lays Trier inline and routes Koblenz->Frankfurt over the
//!   top (tests/repro/claims/port-side-minimal.observed, example 1), so
//!   the inline layout is NOT the port's effect; ports never change node
//!   placement. The port's own effect is attachment-only.
//! - Side mapping (upstream Attributes.pm _flow_as_side, east flow):
//!   right = south, left = north, front = east, back = west. A south
//!   port dips the edge through the trailing gap row below the band
//!   (both attachments south, `^` into the target); a north port bends
//!   above the band. front is the default attachment (no-op).
//! - Indexed ports (`front,N`) need variable node heights — out of this
//!   contract's scope (gently-byq); `back` wraps outside the bounding
//!   box — documented divergence (docs/known-hard-cases.md).

use gently_core::graph::{Graph, Scope};
use gently_core::layout::{self, Layout};
use gently_core::render::ascii;
use std::path::PathBuf;

/// One edge with a `start` port attr.
fn start_port_graph(attr: &str) -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let e = g.add_edge(a, b, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e), "start", attr);
    g
}

/// The skip shape a->b->c plus a port-constrained a->c (port-skip-right).
fn skip_port_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    g.add_edge(a, b, true).expect("live endpoints");
    g.add_edge(b, c, true).expect("live endpoints");
    let e = g.add_edge(a, c, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e), "start", "right");
    g
}

/// The chain a -> port-edge b -> c (port-chain-right).
fn chain_port_graph() -> Graph {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let e = g.add_edge(a, b, true).expect("live endpoints");
    g.set_attr(Scope::Edge(e), "start", "right");
    g.add_edge(b, c, true).expect("live endpoints");
    g
}

/// All path cells of `ei` (from node `src` to node `dst`) sit on one gap
/// row below the source band, with the first cell directly below the
/// source column and the last directly below the target column — the
/// dip-below south-port route.
fn assert_dips_below(l: &Layout, ei: usize, src: usize, dst: usize) {
    let path = &l.edge_paths[ei];
    let (su, sv) = (l.node_cells[src], l.node_cells[dst]);
    let row = path[0].1;
    assert!(row > su.1, "edge {ei} must dip below the source band");
    assert!(
        path.iter().all(|&c| c.1 == row),
        "edge {ei} must run along one row below the band: {path:?}"
    );
    assert_eq!(path[0].0, su.0, "edge {ei} exits the source's south side");
    assert_eq!(
        path[path.len() - 1].0,
        sv.0,
        "edge {ei} enters the target's south side"
    );
}

/// p1 — a `start: right` (south) port on a same-band edge dips the route
/// below the band; placement is unchanged (both nodes still on band 0,
/// one rank apart, ge.layout.c1: ports don't move nodes).
#[test]
fn p1_start_right_dips_below() {
    let g = start_port_graph("right");
    let l = layout::layout(&g);
    assert_eq!(l.node_cells[0].1, l.node_cells[1].1, "nodes stay on one band");
    assert_eq!(l.node_cells[1].0 - l.node_cells[0].0, 2, "one rank apart");
    assert_dips_below(&l, 0, 0, 1);
}

/// p2 — `end: right` routes identically (the record shows one byte shape
/// for start and end south ports; port-skip-right fixture pins end too).
#[test]
fn p2_end_right_dips_below() {
    let mut g = start_port_graph("");
    g.set_attr(Scope::Edge(0), "end", "right");
    let l = layout::layout(&g);
    assert!(l.edge_paths[0].iter().all(|&c| c.1 > l.node_cells[0].1));
    assert_dips_below(&l, 0, 0, 1);
}

/// p3 — a `start: left` (north) port bends the route above the band.
#[test]
fn p3_start_left_bends_above() {
    let g = start_port_graph("left");
    let l = layout::layout(&g);
    let path = &l.edge_paths[0];
    let su = l.node_cells[0];
    let row = path[0].1;
    assert!(row < su.1, "edge must bend above the source band");
    assert!(path.iter().all(|&c| c.1 == row), "one row above the band");
    assert_eq!(path[0].0, su.0, "exits the source's north side");
    assert_eq!(path[path.len() - 1].0, l.node_cells[1].0, "enters the target's north side");
}

/// p4 — the skip shape a->b->c with `start: right` on a->c: the port
/// edge dips below the whole chain while b stays inline (the straight
/// edges keep their gap-column routes).
#[test]
fn p4_skip_right_dips_below_chain() {
    let g = skip_port_graph();
    let l = layout::layout(&g);
    assert_dips_below(&l, 2, 0, 2);
    assert_eq!(
        l.edge_paths[0].len(),
        1,
        "the plain a->b edge stays a straight gap-column run"
    );
}

/// p5 — the chain a -> port-edge b -> c: only the port edge dips; the
/// downstream b->c edge keeps its straight route (port-chain-right).
#[test]
fn p5_chain_right_dips_one_edge() {
    let g = chain_port_graph();
    let l = layout::layout(&g);
    assert_dips_below(&l, 0, 0, 1);
    assert_eq!(l.edge_paths[1].len(), 1, "b->c stays straight");
}

/// p6 — `start: front` (east) is the default attachment: the straight
/// gap-column route, byte-identical to the unattributed edge.
#[test]
fn p6_start_front_is_noop() {
    let g = start_port_graph("front");
    let plain = layout::layout(&{
        let mut g = Graph::default();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.add_edge(a, b, true).expect("live endpoints");
        g
    });
    assert_eq!(layout::layout(&g).edge_paths, plain.edge_paths);
}

// --- recorded-fixture byte pins (the port-* corpus companions) ---

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/graph-easy")
}

fn rendered_ascii(base: &str) -> String {
    let input = std::fs::read_to_string(fixture_dir().join(format!("{base}.txt")))
        .unwrap_or_else(|e| panic!("fixture input {base}.txt: {e}"));
    let g = gently_core::parse::text::parse(&input)
        .unwrap_or_else(|e| panic!("{base} must parse: {e}"));
    let l = layout::layout(&g);
    ascii::render(&g, &l).unwrap_or_else(|e| panic!("{base} must render: {e}"))
}

fn oracle_body(base: &str, format: &str) -> Vec<u8> {
    let s = std::fs::read_to_string(fixture_dir().join(format!("{base}.{format}.expected")))
        .unwrap_or_else(|e| panic!("companion {base}.{format}.expected: {e}"));
    let mut rest = s.as_str();
    while let Some(idx) = rest.find('\n') {
        if rest[..idx].starts_with("# oracle: ") {
            rest = &rest[idx + 1..];
        } else {
            break;
        }
    }
    rest.as_bytes().to_vec()
}

macro_rules! fixture_pin {
    ($fn_name:ident, $base:literal) => {
        #[test]
        fn $fn_name() {
            assert_eq!(
                rendered_ascii($base).into_bytes(),
                oracle_body($base, "ascii"),
                "fixture {}/{} must render byte-identically to the recorded oracle",
                $base, "ascii"
            );
        }
    };
}

fixture_pin!(port_start_right_renders_oracle_ascii, "port-start-right");
fixture_pin!(port_end_right_renders_oracle_ascii, "port-end-right");
fixture_pin!(port_start_left_renders_oracle_ascii, "port-start-left");
fixture_pin!(port_skip_right_renders_oracle_ascii, "port-skip-right");
fixture_pin!(port_chain_right_renders_oracle_ascii, "port-chain-right");
