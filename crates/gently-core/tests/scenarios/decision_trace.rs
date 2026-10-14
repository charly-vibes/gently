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
    if let gently_core::trace_diff::TraceDiffError::DecisionOnly { .. } = &err {
        assert!(
            !cut.is_empty(),
            "the refused cut is reported for --informational"
        );
    } else {
        panic!("byte gate must be the typed DecisionOnly refusal");
    }
    // --informational downgrades the decision-only diff.
    let info = gently_core::trace_diff::trace_diff(same_bytes, same_bytes, a, b, true)
        .expect("informational mode shows the decision-only diff");
    assert!(!info.is_empty());
    // byte-equal AND trace-equal: nothing to report at all.
    assert!(
        gently_core::trace_diff::trace_diff(same_bytes, same_bytes, a, a, false)
            .unwrap()
            .is_empty()
    );
    // byte-DIVERGENT fixtures: the gate never fires — the cut is the meter.
    let divergent = gently_core::trace_diff::trace_diff("a\n", "b\n", a, b, false)
        .expect("byte-divergent fixtures are adjudicable");
    assert!(!divergent.is_empty());

    // ---- arm 4: EXCL-001 corpus property ------------------------------
    excl001();
}

/// EXCL-001 (task 5.3): for every fixture class with a committed oracle
/// dump, classify it by byte-divergence against the pinned oracle bytes
/// (`claims/<name>.observed` seed 0 as_ascii, or the fixture's
/// `.ascii.expected`), then assert the trace-diff contract:
///   - byte-divergent class → nonempty minimal cut against the aligned
///     dump (the dump observes `layout.place`, so the cut's stage is
///     `layout.place`);
///   - byte-identical class with a decision-only trace diff → the byte
///     gate refuses it (typed `DecisionOnly`), and --informational shows
///     the diff.
/// Byte equality itself stays pinned by the existing corpus tests.
fn excl001() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let dumps_dir = root.join("tests/repro/dumps");

    let mut byte_divergent = 0usize;
    let mut byte_identical = 0usize;
    let mut skipped = 0usize;

    for entry in fs::read_dir(&dumps_dir).expect("dumps dir must exist") {
        let path = entry.expect("dump entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt")
            || !path.to_string_lossy().contains(".dump")
        {
            continue;
        }
        let name = path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace(".dump", "");
        let dump = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}"));

        // Input: the fixture file when one exists, else the probe body
        // mined out of probes.pl (either probe form).
        let input = match fs::read_to_string(
            root.join("tests/fixtures/graph-easy")
                .join(format!("{name}.txt")),
        ) {
            Ok(text) => text,
            Err(_) => probe_body(&root.join("tests/repro/probes.pl"), &name)
                .unwrap_or_else(|| panic!("{name}: no fixture and no probe body")),
        };

        // Pinned oracle bytes: claims/<name>.observed seed 0 as_ascii, or
        // the fixture's .ascii.expected (with its pinned `# oracle:`
        // header lines stripped, per the E1 driver's convention).
        let fixture_expected = fs::read_to_string(
            root.join("tests/fixtures/graph-easy")
                .join(format!("{name}.ascii.expected")),
        )
        .map(|t| {
            t.lines()
                .filter(|l| !l.starts_with("# oracle:"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .ok();
        let oracle_bytes = match claims_ascii(
            &root
                .join("tests/repro/claims")
                .join(format!("{name}.observed")),
        ) {
            Some(bytes) => bytes,
            None => match fixture_expected {
                Some(bytes) => bytes,
                // dot-bare-scope-in-named.observed is a multi-case OK log
                // with no seed/as_ascii form — not comparable here.
                None => {
                    skipped += 1;
                    continue;
                }
            },
        };

        // Run gently with the layout.place gate on.
        let (gently_bytes, gently_trace) = run_traced(&name, &input, "layout.place");

        let divergent = gently_bytes.trim_end() != oracle_bytes.trim_end();
        let aligned = gently_core::trace_diff::align_dump(&dump);

        match gently_core::trace_diff::trace_diff(
            &oracle_bytes,
            &gently_bytes,
            &aligned,
            &gently_trace,
            false,
        ) {
            Ok(cut) if cut.is_empty() => {
                assert!(!divergent, "{name}: byte-identical fixture fully agrees");
                byte_identical += 1;
            }
            Ok(cut) => {
                assert!(
                    divergent,
                    "{name}: trace-diff must gate byte-identical fixtures"
                );
                assert!(
                    !cut.is_empty(),
                    "{name}: EXCL-001 — byte-divergent fixture must yield a nonempty trace diff"
                );
                assert!(
                    cut.iter().all(|e| e.stage == "layout.place"),
                    "{name}: cut must live on the dump's observed stage: {cut:?}"
                );
                byte_divergent += 1;
                eprintln!(
                    "excl001 {name}: byte-divergent, cut = {}",
                    cut.iter()
                        .map(|e| format!("{}[{}]", e.stage, e.key))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            Err(gently_core::trace_diff::TraceDiffError::DecisionOnly { cut }) => {
                assert!(!cut.is_empty());
                eprintln!(
                    "excl001 {name}: byte-identical, decision-only cut = {}",
                    cut.iter()
                        .map(|e| format!("{}[{}]", e.stage, e.key))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                assert!(
                    !divergent,
                    "{name}: byte gate fired on byte-DIVERGENT fixture — gate bug"
                );
                byte_identical += 1;
            }
        }
    }

    assert!(
        byte_divergent > 0,
        "EXCL-001 scope must contain at least one byte-divergent fixture class"
    );
    assert!(
        skipped <= 1,
        "at most one dump class may lack a comparable oracle-bytes form, skipped {skipped}"
    );
    assert!(
        byte_divergent + byte_identical >= 23,
        "the dump corpus must be iterated (all comparable dumps), got \
         {byte_divergent}+{byte_identical}, skipped {skipped}"
    );
}

/// Run gently over `source` with the given trace stage gated on; returns
/// (rendered ascii bytes, trace text). A parse or render failure renders
/// empty bytes (the divergence is real — the oracle renders).
fn run_traced(tag: &str, source: &str, stage: &str) -> (String, String) {
    let file = std::env::temp_dir().join(format!("gently-dt-p3-{tag}.trace"));
    std::fs::remove_file(&file).ok();
    std::env::set_var("GENTLY_TRACE", stage);
    std::env::set_var("GENTLY_TRACE_FILE", &file);
    let traced = || -> (String, String) {
        let graph = match gently_core::parse::text::parse(source) {
            Ok(g) => g,
            Err(_) => return (String::new(), String::new()),
        };
        let laid = gently_core::layout::layout(&graph);
        let bytes = gently_core::render::ascii::render(&graph, &laid).unwrap_or_default();
        let trace = std::fs::read_to_string(&file).unwrap_or_default();
        (bytes, trace)
    }();
    std::env::remove_var("GENTLY_TRACE");
    std::env::remove_var("GENTLY_TRACE_FILE");
    std::fs::remove_file(&file).ok();
    traced
}

/// Extract a probe body from probes.pl. Two committed forms:
///   `..._probe_body('<name>', <<'MARKER')` … MARKER
///   `    '<name>' => <<'MARKER',` … MARKER
fn probe_body(probes: &Path, name: &str) -> Option<String> {
    let text = fs::read_to_string(probes).ok()?;
    let opener = format!("'{name}', <<'");
    let hash_opener = format!("'{name}' => <<'");
    let mut marker: Option<&str> = None;
    for line in text.lines() {
        if let Some(i) = line.find(&opener) {
            let rest = &line[i + opener.len()..];
            marker = Some(rest.split('\'').next()?);
            break;
        }
        if let Some(i) = line.find(&hash_opener) {
            let rest = &line[i + hash_opener.len()..];
            marker = Some(rest.split('\'').next()?);
            break;
        }
    }
    let marker = marker?;
    let mut body = Vec::new();
    let mut collecting = false;
    for line in text.lines() {
        if !collecting {
            // The opener line precedes the body.
            if line.contains(&opener) || line.contains(&hash_opener) {
                collecting = true;
            }
            continue;
        }
        if line.trim_end() == marker {
            break;
        }
        body.push(line);
    }
    let mut out = body.join("\n");
    out.push('\n');
    Some(out)
}

/// Extract the pinned oracle `as_ascii` block for seed 0 from a
/// claims/<name>.observed file.
fn claims_ascii(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let mut in_seed0 = false;
    let mut block: Vec<&str> = Vec::new();
    let mut collecting = false;
    for line in text.lines() {
        if line.trim_end() == "== seed 0 ==" {
            in_seed0 = true;
            continue;
        }
        if line.starts_with("== seed ") {
            break;
        }
        if !in_seed0 {
            continue;
        }
        if line.trim_end() == "as_ascii:" {
            collecting = true;
            continue;
        }
        if collecting {
            if line.trim_end() == "as_txt:" {
                break;
            }
            block.push(line);
        }
    }
    if block.is_empty() {
        // Fallback form (`== input:` blocks, e.g. fan-layout.observed):
        // the bytes follow the first header up to the next one.
        let mut raw: Vec<&str> = Vec::new();
        let mut take = false;
        for line in text.lines() {
            if line.starts_with("== input:") {
                if !raw.is_empty() {
                    break;
                }
                take = true;
                continue;
            }
            if take {
                raw.push(line);
            }
        }
        block = raw;
    }
    while block.first().map_or(false, |l| l.trim().is_empty()) {
        block.remove(0);
    }
    while block.last().map_or(false, |l| l.trim().is_empty()) {
        block.pop();
    }
    if block.is_empty() {
        return None;
    }
    Some(block.join("\n"))
}

use std::fs;
use std::path::{Path, PathBuf};
