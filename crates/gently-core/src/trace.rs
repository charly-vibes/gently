//! The decision-trace layer (specs/decision-trace.md, design D1): a
//! versioned, stage-structured emission interface for the intermediate
//! decision category `Dec`. This is the real D1 layer — no longer the
//! deletable E1 prototype (gently-asc).
//!
//! Every event carries the schema [`VERSION`] and its governing spec id
//! [`SPEC_ID`] (c1); the stage vocabulary is [`STAGES`]. Emission is
//! env-gated (c1 says nothing else): `GENTLY_TRACE=<stage>` enables one
//! stage; lines go to `GENTLY_TRACE_FILE` (append mode) or stderr when
//! unset. With `GENTLY_TRACE` unset nothing is emitted and the corpus
//! renders byte-identically (the E1 regression pin, p1).
//!
//! Emitted stages today: `layout.place` and `render.route` (promoted from
//! the E1 prototype), plus `parse` (statement dispatch in the text
//! parser's statement loop — the decision is already materialized there).
//! The remaining c1 stages (`graph.model`, `rank.col`, `bands`, `spans`,
//! `bends`, `glyphs`) are declared but unemitted: their decisions are not
//! yet materialized at reachable emission points. Follow-up beads record
//! those landing sites; do NOT refactor distant code to create emission
//! points.

use std::cell::RefCell;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The trace schema version (c1: versioned from day one).
pub const VERSION: u32 = 1;

/// The governing spec id carried by every event (c1).
pub const SPEC_ID: &str = "decision.trace";

/// The declared stage vocabulary (c1): one sub-vocabulary per pipeline
/// stage. `layout.place` covers the spans stage; `render.route` covers
/// route classification (seed vocabulary: the `Route` enum of
/// `layout.rs` — `Selfloop`, `Grew`, `Parallel(usize)`, `Around`,
/// `PortBelow`, `PortAbove`) and bends. The unemitted stages are
/// follow-up work (see the module docs).
pub const STAGES: [&str; 10] = [
    "parse",
    "graph.model",
    "rank.col",
    "bands",
    "spans",
    "route.classify",
    "bends",
    "glyphs",
    "layout.place",
    "render.route",
];

thread_local! {
    /// Test seam: when set, overrides the env gate on this thread only.
    /// Parallel integration tests mutating the process-global
    /// GENTLY_TRACE/GENTLY_TRACE_FILE raced each other's emissions (the
    /// windows-latest CI flake in decision_trace::p1); a thread-local
    /// override isolates each test thread while the env path stays the
    /// real, spec'd behavior for ordinary processes.
    static GATE_OVERRIDE: RefCell<Option<(Option<String>, Option<PathBuf>)>> =
        const { RefCell::new(None) };
}

/// Set the thread-local gate override: `stage` replaces GENTLY_TRACE,
/// `file` replaces GENTLY_TRACE_FILE, each `None` meaning "treat as
/// unset". Dies with the thread — no cleanup needed. Test seam only
/// (`#[doc(hidden)]`); production gating stays env-driven (c1).
#[doc(hidden)]
pub fn set_test_gate(stage: Option<&str>, file: Option<&Path>) {
    GATE_OVERRIDE.with(|g| {
        *g.borrow_mut() = Some((stage.map(|s| s.to_string()), file.map(|f| f.to_path_buf())));
    });
}

/// Whether the given stage is named by `GENTLY_TRACE` (c1 gating), or by
/// the thread-local test override when that is set.
pub fn enabled(stage: &str) -> bool {
    GATE_OVERRIDE.with(|g| {
        if let Some((stage_override, _)) = &*g.borrow() {
            return stage_override.as_deref() == Some(stage);
        }
        env::var("GENTLY_TRACE")
            .map(|v| v == stage)
            .unwrap_or(false)
    })
}

/// Append one trace line for an enabled stage. The line carries the
/// schema version, the stage, and the governing spec id (c1); `event` is
/// the stage-specific payload. No-op — and no file created — when the
/// stage is not enabled (p1 regression arm).
pub fn emit(stage: &str, event: &str) {
    if !enabled(stage) {
        return;
    }
    let line = format!("dt v{VERSION} stage={stage} spec={SPEC_ID} {event}\n");
    let file_override = GATE_OVERRIDE.with(|g| {
        let g = g.borrow();
        g.as_ref()
            .and_then(|(_, f)| f.as_ref().map(|p| p.to_string_lossy().into_owned()))
    });
    match file_override {
        Some(f) => {
            if let Ok(mut out) = OpenOptions::new().create(true).append(true).open(&f) {
                let _ = out.write_all(line.as_bytes());
            }
        }
        None => match env::var("GENTLY_TRACE_FILE") {
            Ok(f) => {
                if let Ok(mut out) = OpenOptions::new().create(true).append(true).open(&f) {
                    let _ = out.write_all(line.as_bytes());
                }
            }
            Err(_) => {
                let _ = std::io::stderr().write_all(line.as_bytes());
            }
        },
    }
}

/// The column/row span of node `idx`, read straight from its attribute
/// table (`columns`/`rows`, falling back to `size`'s halves — same rule
/// as `layout::node_span`). Kept from the E1 prototype: the layout.rs
/// emission site stays self-contained.
pub(crate) fn node_span_of(graph: &crate::graph::Graph, idx: usize) -> (usize, usize) {
    let side = |key: &str| {
        graph
            .get_attr(crate::graph::Scope::Node(idx), key)
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(1)
    };
    (side("columns"), side("rows"))
}
