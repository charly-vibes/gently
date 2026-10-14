//! Env-gated decision-trace prototype emission (gently-asc). Deletable —
//! NOT the trace layer design (D1–D6 stay provisional). No default-path
//! behavior change: with `GENTLY_TRACE` unset nothing is emitted and the
//! corpus renders byte-identically.
//!
//! Stages: `layout.place` (per-node grid cell + span) and `render.route`
//! (per-edge path cell walk). Set `GENTLY_TRACE=<stage>` to enable;
//! lines go to `GENTLY_TRACE_FILE` (append mode) or stderr when unset.

use std::env;
use std::fs::OpenOptions;
use std::io::Write;

/// Whether the given stage is named by `GENTLY_TRACE`.
pub(crate) fn enabled(stage: &str) -> bool {
    env::var("GENTLY_TRACE")
        .map(|v| v == stage)
        .unwrap_or(false)
}

/// Append one compact trace line for an enabled stage.
pub(crate) fn emit(stage: &str, line: String) {
    if !enabled(stage) {
        return;
    }
    let line = format!("{line}\n");
    match env::var("GENTLY_TRACE_FILE") {
        Ok(f) => {
            if let Ok(mut out) = OpenOptions::new().create(true).append(true).open(&f) {
                let _ = out.write_all(line.as_bytes());
            }
        }
        Err(_) => {
            let _ = std::io::stderr().write_all(line.as_bytes());
        }
    }
}

/// The column/row span of node `idx`, read straight from its attribute
/// table (`columns`/`rows`, falling back to `size`'s halves — same rule
/// as `layout::node_span`, kept self-contained so the prototype patch
/// applies cleanly onto pre-span revisions in the E1 driver).
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
