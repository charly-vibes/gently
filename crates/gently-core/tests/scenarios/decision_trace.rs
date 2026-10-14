//! decision.trace scenario tests (specs/decision-trace.md p1).
//!
//! p1 executes the versioned-emission contract: every trace event carries
//! the schema version and its governing spec id (`decision.trace`, c1),
//! and with `GENTLY_TRACE` unset emission is silent — no trace file is
//! created and rendering is byte-identical (the E1 regression pin; byte
//! equality stays gated by the pinned corpus tests, this arm pins the
//! API-level gate).

use gently_core::{graph::Graph, layout};

/// Per-phase trace file (unique per process; the test body runs its
/// phases sequentially so the env mutations never race each other).
fn trace_file(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("gently-dt-p1-{}-{tag}", std::process::id()))
}

fn set_gate(stage: &str, file: &std::path::Path) {
    std::env::set_var("GENTLY_TRACE", stage);
    std::env::set_var("GENTLY_TRACE_FILE", file);
}

fn clear_gate() {
    std::env::remove_var("GENTLY_TRACE");
    std::env::remove_var("GENTLY_TRACE_FILE");
}

/// decision.trace.p1 (specs/decision-trace.md p1, deriving from c1):
/// gate-off silence + every emitted event carries the schema version and
/// the governing spec id, for each emitted stage.
#[test]
fn p1() {
    let graph = Graph::tracer();

    // Regression arm: with GENTLY_TRACE unset, emission is a no-op — even
    // with GENTLY_TRACE_FILE pointed at a fresh path, no bytes appear
    // (byte-identical corpus stays pinned by the existing corpus tests).
    clear_gate();
    let off = trace_file("off");
    std::fs::remove_file(&off).ok();
    set_gate("", &off);
    std::env::remove_var("GENTLY_TRACE");
    let _ = layout::layout(&graph);
    assert!(
        !off.exists(),
        "trace-off must not create or write the trace file"
    );
    clear_gate();

    // Version arm: with the gate on, every line for every emitted stage
    // carries the schema version, the stage, and the governing spec id.
    for stage in ["layout.place", "render.route", "parse"] {
        let file = trace_file(&stage.replace('.', "-"));
        std::fs::remove_file(&file).ok();
        set_gate(stage, &file);
        let _ = layout::layout(&graph);
        if stage == "parse" {
            // parse events fire in the text parser's statement loop, not
            // in the layout pass — drive a tiny source through it.
            let _ = gently_core::parse::text::parse("[ a ] --> [ b ]\n");
        }
        std::env::remove_var("GENTLY_TRACE");
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("trace file for stage {stage} missing: {e}"));
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        assert!(
            !lines.is_empty(),
            "stage {stage} must emit at least one event on the tracer graph"
        );
        for line in lines {
            assert!(
                line.contains("dt v1"),
                "event lacks the schema version (c1): {line:?}"
            );
            assert!(
                line.contains("spec=decision.trace"),
                "event lacks the governing spec id (c1): {line:?}"
            );
            assert!(
                line.contains(&format!("stage={stage}")),
                "event lacks its stage: {line:?}"
            );
        }
        clear_gate();
        std::fs::remove_file(&file).ok();
    }
}
