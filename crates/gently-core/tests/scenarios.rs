//! Scenario tests, grouped per capability.
//!
//! Filters used by espectacular contracts (`ah check --run-tests`) match
//! these module paths: `scenarios::<cap>::<pid>`.

/// ge.graph_model (gently-4ht): the shared model contract (specs/ge-graph_model.md p1–p5).
#[path = "scenarios/tb.rs"]
mod tb;

#[path = "scenarios/ge_graph_model.rs"]
mod ge_graph_model;

/// ge.txt_render (gently-3hv): the canonical txt serialization contract (specs/ge-txt_render.md p1–p5).
#[path = "scenarios/ge_txt_render.rs"]
mod ge_txt_render;

/// ge.text_parser (gently-bzx): the Graph::Easy text-format grammar — the
/// full capability (operators, labels, attributes, groups, comments)
/// (specs/ge-text_parser.md p1–p5).
#[path = "scenarios/ge_text_parser.rs"]
mod ge_text_parser;

/// ge.ascii_render (gently-0cq): the classic ASCII renderer — border
/// styles, edge style runs, shapes, label/wide-glyph handling
/// (specs/ge-ascii_render.md p1–p5).
#[path = "scenarios/ge_ascii_render.rs"]
mod ge_ascii_render;

/// ge.layout (gently-4nx): the deterministic grid layout capability (specs/ge-layout.md p1–p5).
#[path = "scenarios/ge_layout.rs"]
mod ge_layout;
#[path = "scenarios/ge_layout_ports.rs"]
mod ge_layout_ports; // gently-89r

/// ge.oracle (gently-mwo): tier-1 admission enforcement — the repro-probe
/// harness (`tests/repro/probes.pl admit`) classifies every corpus fixture
/// by hash-stability × envelope; machine-checked against the corpus
/// (specs/ge-oracle.md p7).
#[path = "scenarios/ge_oracle.rs"]
mod ge_oracle;

/// ge.boxart_render (gently-css): the Unicode boxart renderer — border
/// styles, edge styles, junctions, shapes (specs/ge-boxart_render.md p1–p5).
#[path = "scenarios/ge_boxart_render.rs"]
mod ge_boxart_render;

/// ge.html_render (gently-eyo): the table-based HTML renderer — grid
/// mapping, node labels/links, edge style classes, embedded CSS, colors
/// and shapes (specs/ge-html_render.md p1–p5).
#[path = "scenarios/ge_html_render.rs"]
mod ge_html_render;

#[path = "scenarios/ge_dot_parser.rs"]
mod ge_dot_parser;

/// decision.trace (gently-p7y): the versioned trace layer — every event
/// carries the schema version and its governing spec id; gate-off silence
/// (specs/decision-trace.md p1).
#[path = "scenarios/decision_trace.rs"]
mod decision_trace;

/// ge.perf (gently-4ln): performance budget gates (specs/ge-perf.md p1–p5).
#[path = "scenarios/ge_perf.rs"]
mod ge_perf;

/// ge.graphviz_render (gently-b4v): the DOT output renderer — node, edge,
/// and group emission plus the dot_parser round trip (specs/ge-graphviz_render.md).
#[path = "scenarios/ge_graphviz_render.rs"]
mod ge_graphviz_render;

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
