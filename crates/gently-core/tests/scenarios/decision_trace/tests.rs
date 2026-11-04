use super::dump::*;
use super::helpers::*;
use super::verdicts::*;
use super::*;

/// decision.trace.p4 (c4): every committed dump carries the pin header,
/// an explicit hash-seed field, and per-stage observability marks, and
/// the admission manifest admits no tier-2 fixture (ge.oracle.c7).
#[test]
fn p4() {
    let dumps = repo_root().join("tests/repro/dumps");
    let mut names = Vec::new();
    for entry in fs::read_dir(&dumps).must("dumps dir must exist") {
        let path = entry.must("dumps entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let name = path.file_stem().must("value").to_string_lossy().to_string();
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
        .must("a stable admitted row passes");
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
        .must("the (b)-checklist registry must be committed under tests/repro/trace/");
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
    let records: Vec<_> = fs::read_dir(&designs)
        .must("decision-trace designs dir must exist")
        .filter_map(|entry| {
            let path = entry.must("designs entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                return None;
            }
            fs::read_to_string(&path).ok().map(|text| (path, text))
        })
        .collect();
    for (path, text) in &records {
        adjudications += check_adjudication(path, text);
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
    .must("the gently-58m adjudication record must exist");
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

/// One adjudication record under the decision-trace designs: must carry
/// the five-item verdict table; a `(b)` class must hold all five.
/// Returns 1 when the record is a Final-class adjudication, 0 otherwise.
fn check_adjudication(path: &std::path::Path, text: &str) -> usize {
    let Some(class_line) = text.lines().find(|l| l.starts_with("## Final class")) else {
        return 0;
    };
    let rows = checklist_verdicts(text);
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
    1
}
