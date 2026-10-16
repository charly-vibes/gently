//! decision.trace scenario tests (specs/decision-trace.md p1–p6).
//!
//! p1 executes the versioned-emission contract: every trace event carries
//! the schema version and its governing spec id (`decision.trace`, c1),
//! and with `GENTLY_TRACE` unset emission is silent — no trace file is
//! created and rendering is byte-identical (the E1 regression pin; byte
//! equality stays gated by the pinned corpus tests, this arm pins the
//! API-level gate).
//!
//! p2's enforcement lives in `crates/gently-core/tests/e3_faithfulness.rs`
//! (the E3 pairwise-faithfulness + row_nN subtests — the contract TOML
//! points the `decision.trace::p2` filter there, gently-a3l). p3 pins the
//! minimal-antichain cut semantics. p4/p5/p6 (gently-a3l) enforce the
//! committed-dump corpus gates (c4) and their typed-error arms (c5) and
//! the adjudication-record structure (c6) over the committed registry.

/// The repo root as seen from the scenario tests.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The pinned oracle identity every dump pin header must name
/// (specs/ge-oracle.md c1).
const PIN_COMMIT: &str = "ededa3d787ad89ac532c578c06390e8a7b270499";

/// One committed-dump defect, rendered as the c5 typed error: the defect
/// named plus the remediation (re-dump under the pinned seed per the
/// trace README's recording recipe, or removal from the corpus via the
/// admission probe, [[ge.oracle.c7]]).
fn dump_defect(defect: &str) -> String {
    format!(
        "invalid dump: {defect} — remediation: re-dump under the pinned seed \
         (tests/repro/trace/README.md recording recipe) or remove the fixture \
         from the corpus via the admission probe (ge.oracle.c7)"
    )
}

/// c5's typed gate over one committed dump (c4's companion contract): the
/// oracle pin header, the explicit hash-seed field, and the per-stage
/// observability marks must all be present.
fn check_dump(name: &str, text: &str) -> Result<(), String> {
    let header = text.lines().next().unwrap_or("");
    let pin_ok = header.starts_with("# dump: Graph::Easy v0.69 @ ") && header.contains(PIN_COMMIT);
    if !pin_ok {
        return Err(dump_defect(&format!(
            "{name}: missing the oracle pin header ({PIN_COMMIT})"
        )));
    }
    if !text
        .lines()
        .any(|l| l.starts_with("-- seed ") && l["-- seed ".len()..].trim().parse::<i64>().is_ok())
    {
        return Err(dump_defect(&format!(
            "{name}: missing the explicit hash-seed field"
        )));
    }
    if !text.lines().any(|l| l.starts_with("== invocation")) {
        return Err(dump_defect(&format!(
            "{name}: missing the per-stage observability marks"
        )));
    }
    Ok(())
}

/// Split one dump into its `-- seed N` blocks (comment lines excluded) —
/// the raw material for the tier-2 stability cross-check.
fn seed_blocks(text: &str) -> Vec<Vec<String>> {
    let mut blocks: Vec<Vec<String>> = Vec::new();
    let mut cur: Option<Vec<String>> = None;
    for line in text.lines() {
        if line.starts_with("-- seed ") {
            if let Some(b) = cur.take() {
                blocks.push(b);
            }
            cur = Some(Vec::new());
        } else if cur.is_some() && !line.starts_with('#') {
            cur.as_mut().unwrap().push(line.to_string());
        }
    }
    if let Some(b) = cur.take() {
        blocks.push(b);
    }
    blocks
}

/// c5's manifest gate over `tests/repro/admission.tsv`: the pin header
/// with the tier-1 gate fields (seed-pin, envelope), the closed verdict
/// set, every admitted (stable) row's dump companion committed, and NO
/// tier-2 fixture admitted — an admitted row whose dump's seed blocks
/// differ is exactly the tier-2-admitted defect.
fn check_manifest(manifest: &str, dump_of: &dyn Fn(&str) -> Option<String>) -> Result<(), String> {
    let mut lines = manifest.lines();
    let header = lines
        .next()
        .ok_or_else(|| dump_defect("admission manifest is empty"))?;
    if !(header.starts_with(&format!("# admission: Graph::Easy v0.69 @ {PIN_COMMIT}"))
        && header.contains("seed-pin=pinned")
        && header.contains("envelope="))
    {
        return Err(dump_defect(
            "admission manifest header missing the pin or the tier-1 gate \
             fields (seed-pin, envelope)",
        ));
    }
    for line in lines.filter(|l| !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 3 {
            return Err(dump_defect(
                "admission manifest row is not a name<TAB>verdict<TAB>seconds triple",
            ));
        }
        let (name, verdict) = (cols[0], cols[1]);
        if !matches!(verdict, "stable" | "hash-dependent" | "render-error") {
            return Err(dump_defect(&format!(
                "{name}: verdict '{verdict}' is outside the closed set \
                 (stable, hash-dependent, render-error)"
            )));
        }
        if verdict == "stable" {
            let dump = dump_of(name).ok_or_else(|| {
                dump_defect(&format!(
                    "{name}: admitted (stable) but its dump companion is missing"
                ))
            })?;
            let blocks = seed_blocks(&dump);
            if blocks.len() >= 2 && blocks.iter().any(|b| *b != blocks[0]) {
                return Err(dump_defect(&format!(
                    "{name}: admitted (stable) but its dump's seed blocks differ — \
                     a tier-2 (hash-seed-dependent) fixture is admitted"
                )));
            }
        }
    }
    Ok(())
}

/// decision.trace.p4 (c4): every committed dump carries the pin header,
/// an explicit hash-seed field, and per-stage observability marks, and
/// the admission manifest admits no tier-2 fixture (ge.oracle.c7).
#[test]
fn p4() {
    let dumps = repo_root().join("tests/repro/dumps");
    let mut names = Vec::new();
    for entry in fs::read_dir(&dumps).expect("dumps dir must exist") {
        let path = entry.expect("dumps entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        check_dump(&name, &text).unwrap_or_else(|e| panic!("committed dump gate (c4/c5): {e}"));
        names.push((name, text));
    }
    assert!(
        names.len() >= 5,
        "the committed-dump corpus must be non-trivial, got {}",
        names.len()
    );
    let manifest_path = repo_root().join("tests/repro/admission.tsv");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let dump_of = |name: &str| {
        names
            .iter()
            .find(|(n, _)| *n == name || format!("{name}.dump") == *n)
            .map(|(_, t)| t.clone())
    };
    check_manifest(&manifest, &dump_of)
        .unwrap_or_else(|e| panic!("admission manifest gate (c4/c5): {e}"));
}

/// decision.trace.p5 (c5): each invalid-dump environment yields the typed
/// error naming the defect and the remediation — missing pin header,
/// missing seed field, missing observability marks, and a tier-2 fixture
/// admitted as stable.
#[test]
fn p5() {
    let pinned = format!("# dump: Graph::Easy v0.69 @ {PIN_COMMIT} — claim: x");
    let body = "-- seed 0\n== invocation 1 (case0) nodes=1 edge-cells=0 group-cells=0\nband y=0 nodes=1 edge-cells=0 group-cells=0\n  A @ (0,0) 5x3\n";
    let remediation = "remediation";

    // Missing pin header.
    let e =
        check_dump("x", &format!("-- seed 0\n{body}")).expect_err("missing pin header must fail");
    assert!(e.contains("pin header") && e.contains(remediation), "{e}");

    // Missing seed field (the body without its `-- seed 0` line).
    let e = check_dump(
        "x",
        &format!(
            "{pinned}\n== invocation 1 (case0) nodes=1 edge-cells=0 group-cells=0\n\
             band y=0 nodes=1 edge-cells=0 group-cells=0\n  A @ (0,0) 5x3\n"
        ),
    )
    .expect_err("missing seed field must fail");
    assert!(
        e.contains("hash-seed field") && e.contains(remediation),
        "{e}"
    );

    // Missing observability marks.
    let e = check_dump(
        "x",
        &format!("{pinned}\n-- seed 0\nnodes=1 edge-cells=0 group-cells=0\n"),
    )
    .expect_err("missing observability marks must fail");
    assert!(
        e.contains("observability marks") && e.contains(remediation),
        "{e}"
    );

    // A tier-2 fixture admitted as stable: manifest row `stable` over a
    // dump whose seed blocks differ.
    let unstable = format!(
        "{pinned}\n-- seed 0\n== invocation 1 (c) A @ (0,0)\n\
         -- seed 1\n== invocation 1 (c) A @ (9,9)\n"
    );
    let manifest = format!(
        "# admission: Graph::Easy v0.69 @ {PIN_COMMIT} perl=v seeds=4 seed-pin=pinned=0 envelope=10s\n\
         fan\tstable\t0.1\n"
    );
    let e = check_manifest(&manifest, &|n: &str| (n == "fan").then(|| unstable.clone()))
        .expect_err("an admitted tier-2 fixture must fail");
    assert!(e.contains("tier-2") && e.contains("remediation"), "{e}");

    // The same manifest over a seed-stable dump admits cleanly.
    let stable = format!(
        "{pinned}\n-- seed 0\n== invocation 1 (c) A @ (0,0)\n\
         -- seed 1\n== invocation 1 (c) A @ (0,0)\n"
    );
    check_manifest(&manifest, &|n: &str| (n == "fan").then(|| stable.clone()))
        .expect("a stable admitted row passes");
}

/// decision.trace.p6 (c6): the adjudication registry is committed under
/// `tests/repro/trace/` listing the five checklist items; every wai
/// adjudication decision record carries per-item verdicts for all five
/// items; every record classed (b) has all five holding; the gently-58m
/// precedent record exists, classed (b), five of five.
#[test]
fn p6() {
    let root = repo_root();

    // The registry: committed, listing exactly the five checklist items.
    let registry = fs::read_to_string(root.join("tests/repro/trace/adjudication-checklist.md"))
        .expect("the (b)-checklist registry must be committed under tests/repro/trace/");
    let items = checklist_rows(&registry);
    assert_eq!(
        items.len(),
        5,
        "the registry must pre-register exactly five checklist items: {items:?}"
    );

    // Every wai adjudication record under the decision-trace designs: a
    // Final class section must come with the five-item verdict table; a
    // (b) class must hold all five.
    let designs = root.join(".wai/projects/decision-trace/designs");
    let mut adjudications = 0;
    for entry in fs::read_dir(&designs).expect("decision-trace designs dir must exist") {
        let path = entry.expect("designs entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let Some(class_line) = text.lines().find(|l| l.starts_with("## Final class")) else {
            continue;
        };
        adjudications += 1;
        let rows = checklist_verdicts(&text);
        assert_eq!(
            rows.len(),
            5,
            "{}: an adjudication record must carry per-item verdicts for all \
             five checklist items, got {}",
            path.display(),
            rows.len()
        );
        if class_line.contains("(b)") {
            for (n, verdict) in &rows {
                assert!(
                    verdict.starts_with("HOLDS"),
                    "{}: (b) verdict requires item {n} to hold, got {verdict:?}",
                    path.display()
                );
            }
        }
    }
    assert!(
        adjudications >= 1,
        "at least the gently-58m dry-run adjudication must be recorded"
    );

    // The precedent: gently-58m classed (b) with all five holding.
    let precedent = fs::read_to_string(root.join(
        ".wai/projects/decision-trace/designs/\
         2026-10-14-decision-e4-adjudication-dry-run-gently-58m.md",
    ))
    .expect("the gently-58m adjudication record must exist");
    assert!(
        precedent.contains("## Final class: (b)"),
        "gently-58m must be classed (b)"
    );
    let verdicts = checklist_verdicts(&precedent);
    assert_eq!(verdicts.len(), 5, "gently-58m: five item verdicts");
    assert!(
        verdicts.iter().all(|(_, v)| v.starts_with("HOLDS")),
        "gently-58m: all five items must hold: {verdicts:?}"
    );
}

/// Parse a checklist verdict table — rows `| N | item | verdict | ...`
/// with N in 1..=5 — into `(item number, verdict)` pairs.
fn checklist_verdicts(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if !l.starts_with("| ") {
                return None;
            }
            let cells: Vec<&str> = l.split('|').collect();
            // "| a | b | ... |" splits into ["", a, b, ...]
            if cells.len() < 4 {
                return None;
            }
            let n: usize = cells[1].trim().parse().ok()?;
            if !(1..=5).contains(&n) {
                return None;
            }
            // Verdicts are **HOLDS**-style bolded; a holding verdict may
            // carry a qualifier ("HOLDS, modified form") — holding is the
            // HOLDS prefix, never the FAILS prefix.
            let verdict = cells[3].trim().trim_matches('*');
            Some((n, verdict.to_string()))
        })
        .collect()
}

/// Parse the registry's item list — rows `| N | item |` with N in 1..=5.
fn checklist_rows(text: &str) -> Vec<usize> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if !l.starts_with("| ") {
                return None;
            }
            let cells: Vec<&str> = l.split('|').collect();
            if cells.len() != 4 {
                return None;
            }
            let n: usize = cells[1].trim().parse().ok()?;
            (1..=5).contains(&n).then_some(n)
        })
        .collect()
}

use gently_core::{graph::Graph, layout};

/// Per-phase trace file (unique per process; the test body runs its
/// phases sequentially so the env mutations never race each other).
fn trace_file(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("gently-dt-p1-{}-{tag}", std::process::id()))
}

fn set_gate(stage: &str, file: &std::path::Path) {
    gently_core::trace::set_test_gate(Some(stage), Some(file));
}

fn clear_gate() {
    gently_core::trace::set_test_gate(None, None);
}

/// decision.trace.p1 (specs/decision-trace.md p1, deriving from c1):
/// gate-off silence + every emitted event carries the schema version and
/// the governing spec id, for each emitted stage.
#[test]
fn p1() {
    let graph = Graph::tracer();

    // Regression arm: with the gate unset, emission is a no-op — even
    // with a file pointed at a fresh path, no bytes appear
    // (byte-identical corpus stays pinned by the existing corpus tests).
    clear_gate();
    let off = trace_file("off");
    std::fs::remove_file(&off).ok();
    // stage=None models "GENTLY_TRACE unset" while the file arm stays
    // pointed — the exact p1 regression shape.
    gently_core::trace::set_test_gate(None, Some(&off));
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
    let gently_core::trace_diff::TraceDiffError::DecisionOnly { cut } = err;
    assert!(
        !cut.is_empty(),
        "the refused cut is reported for --informational"
    );
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
///
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
    // Thread-local gate override, never the process-global env: 145
    // sibling tests run layout concurrently and the env is shared state
    // (the windows-latest CI flake). Each test thread sees only its own
    // gate; the env path stays covered by the env_smoke test below.
    gently_core::trace::set_test_gate(Some(stage), Some(&file));
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
    gently_core::trace::set_test_gate(None, None);
    std::fs::remove_file(&file).ok();
    traced
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
    let text = std::fs::read_to_string(&file).expect("env-gated emission writes the file");
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
    while block.first().is_some_and(|l| l.trim().is_empty()) {
        block.remove(0);
    }
    while block.last().is_some_and(|l| l.trim().is_empty()) {
        block.pop();
    }
    if block.is_empty() {
        return None;
    }
    Some(block.join("\n"))
}

use std::fs;
use std::path::{Path, PathBuf};
