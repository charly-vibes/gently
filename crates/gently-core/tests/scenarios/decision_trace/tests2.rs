use super::dump::*;
use super::helpers::*;
use super::verdicts::*;
use super::*;

/// decision.trace.p1 (specs/decision-trace.md p1, deriving from c1):
/// gate-off silence + every emitted event carries the schema version and
/// the governing spec id, for each emitted stage.
#[test]
fn p1() {
    let graph = Graph::tracer();
    p1_gate_off_silence(&graph);
    p1_event_headers(&graph);
    assert!(trace_file("p1").file_name().is_some());
}

/// Regression arm: with the gate unset, emission is a no-op — even
/// with a file pointed at a fresh path, no bytes appear
/// (byte-identical corpus stays pinned by the existing corpus tests).
fn p1_gate_off_silence(graph: &Graph) {
    clear_gate();
    let off = trace_file("off");
    std::fs::remove_file(&off).ok();
    // stage=None models "GENTLY_TRACE unset" while the file arm stays
    // pointed — the exact p1 regression shape.
    gently_core::trace::set_test_gate(None, Some(&off));
    let _ = layout::layout(graph);
    assert!(
        !off.exists(),
        "trace-off must not create or write the trace file"
    );
    clear_gate();
}

/// Version arm: with the gate on, every line for every emitted stage
/// carries the schema version, the stage, and the governing spec id.
fn p1_event_headers(graph: &Graph) {
    for stage in ["layout.place", "render.route", "parse"] {
        let file = trace_file(&stage.replace('.', "-"));
        p1_stage(graph, stage, &file);
        assert_stage_events(stage, &file);
    }
    clear_gate();
}

/// Gate on, run the stage's driver, drop the env arm. parse events fire
/// in the text parser's statement loop, not in the layout pass — so the
/// parse arm drives a tiny source through it.
fn p1_stage(graph: &Graph, stage: &str, file: &std::path::Path) {
    std::fs::remove_file(file).ok();
    set_gate(stage, file);
    let _ = layout::layout(graph);
    if stage == "parse" {
        let _ = gently_core::parse::text::parse("[ a ] --> [ b ]\n");
    }
    std::env::remove_var("GENTLY_TRACE");
}

fn assert_stage_events(stage: &str, file: &std::path::Path) {
    let text = std::fs::read_to_string(file)
        .unwrap_or_else(|e| panic!("trace file for stage {stage} missing: {e}"));
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        !lines.is_empty(),
        "stage {stage} must emit at least one event on the tracer graph"
    );
    for line in lines {
        assert_event_headers(line, stage);
    }
}

/// c1: every emitted event carries the schema version, the governing
/// spec id, and its own stage.
fn assert_event_headers(line: &str, stage: &str) {
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

/// decision.trace.p3 (specs/decision-trace.md p3, deriving from c3) plus
/// the EXCL-001 corpus property (design doc D3/D6, task 5.3): trace-diff
/// reports the minimal antichain (cut set) of differing events over the
/// decision dependency poset — never a unique "first" — and runs only on
/// byte-divergent fixtures; byte-equal fixtures yield at most an
/// informational decision-only diff (the typed byte-gate refusal), byte
/// equality staying the CI gate.
///
/// EXCL-001: every byte-divergent fixture in the iterated scope (the
/// committed oracle dumps + the byte-divergent probe classes they pin)
/// yields a nonempty trace diff against its committed dump, aligned onto
/// the `layout.place` vocabulary by the promoted E1 alignment shim.
#[test]
fn p3() {
    // ---- arm 1: the cut is the antichain, not a first event ----------
    let a = "dt v1 stage=layout.place spec=decision.trace place A x=0 y=0 cx=1 cy=1\n\
             dt v1 stage=layout.place spec=decision.trace place B x=2 y=0 cx=1 cy=1\n\
             dt v1 stage=render.route spec=decision.trace route e0 A->B cells=3 bends=0\n";
    let b = "dt v1 stage=layout.place spec=decision.trace place A x=0 y=1 cx=1 cy=1\n\
             dt v1 stage=layout.place spec=decision.trace place B x=2 y=2 cx=1 cy=1\n\
             dt v1 stage=render.route spec=decision.trace route e0 A->B cells=5 bends=2\n";
    let cut = gently_core::trace_diff::minimal_cut(a, b);
    assert!(
        cut.len() >= 2,
        "cut must carry the full antichain (all differing place events), got {cut:?}"
    );
    assert!(
        cut.iter().all(|e| e.stage == "layout.place"),
        "cut stage must be the earliest divergent stage (layout.place): {cut:?}"
    );
    assert_eq!(
        cut.len(),
        gently_core::trace_diff::minimal_cut(b, a).len(),
        "cut is symmetric"
    );

    // ---- arm 2: cut never skips an earlier divergent stage -----------
    let early_a = "dt v1 stage=parse spec=decision.trace stmt line=0 kind=edge\n\
                   dt v1 stage=layout.place spec=decision.trace place A x=0 y=0 cx=1 cy=1\n";
    let early_b = "dt v1 stage=parse spec=decision.trace stmt line=0 kind=node\n\
                   dt v1 stage=layout.place spec=decision.trace place A x=0 y=0 cx=1 cy=1\n";
    let cut = gently_core::trace_diff::minimal_cut(early_a, early_b);
    assert!(
        cut.iter().all(|e| e.stage == "parse"),
        "earliest divergent stage (parse) owns the cut: {cut:?}"
    );

    // ---- arm 3: the byte gate (5.2) -----------------------------------
    let same_bytes = "same rendered bytes\n";
    let err = gently_core::trace_diff::trace_diff(same_bytes, same_bytes, a, b, false)
        .expect_err("byte gate must refuse byte-identical fixtures with differing traces");
    assert_eq!(err.to_string(), gently_core::trace_diff::INFORMATIONAL_HINT);
    let gently_core::trace_diff::TraceDiffError::DecisionOnly { cut } = err;
    assert!(
        !cut.is_empty(),
        "the refused cut is reported for --informational"
    );
    // --informational downgrades the decision-only diff.
    let info = gently_core::trace_diff::trace_diff(same_bytes, same_bytes, a, b, true)
        .must("informational mode shows the decision-only diff");
    assert!(!info.is_empty());
    // byte-equal AND trace-equal: nothing to report at all.
    assert!(
        gently_core::trace_diff::trace_diff(same_bytes, same_bytes, a, a, false)
            .must("value")
            .is_empty()
    );
    // byte-DIVERGENT fixtures: the gate never fires — the cut is the meter.
    let divergent = gently_core::trace_diff::trace_diff("a\n", "b\n", a, b, false)
        .must("byte-divergent fixtures are adjudicable");
    assert!(!divergent.is_empty());

    // ---- arm 4: EXCL-001 corpus property ------------------------------
    excl001();
}

/// The real env path (c1) exercised without racing parallel siblings:
/// the gate stage is set on the real env while this thread runs layout,
/// and the assertions only pin schema shape (version + spec id), never
/// the stage name — a concurrent sibling's layout emitting into the
/// unique temp file appends valid lines we happily accept. No other test
/// touches the env, and `gently-dt-env-smoke` is not an emitted stage
/// name, so no sibling can be polluted by this window.
#[test]
fn env_smoke() {
    let file =
        std::env::temp_dir().join(format!("gently-dt-env-smoke-{}.trace", std::process::id()));
    std::fs::remove_file(&file).ok();
    std::env::set_var("GENTLY_TRACE", "layout.place");
    std::env::set_var("GENTLY_TRACE_FILE", &file);
    let graph = Graph::tracer();
    let _ = layout::layout(&graph);
    std::env::remove_var("GENTLY_TRACE");
    std::env::remove_var("GENTLY_TRACE_FILE");
    let text = std::fs::read_to_string(&file).must("env-gated emission writes the file");
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert!(!lines.is_empty(), "env gate must enable emission");
    for line in lines {
        assert!(
            line.contains("dt v") && line.contains("spec=decision.trace"),
            "env-path event lacks the schema shape: {line:?}"
        );
    }
    std::fs::remove_file(&file).ok();
}
