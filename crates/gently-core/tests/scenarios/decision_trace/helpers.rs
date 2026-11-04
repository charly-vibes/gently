use super::*;

/// c5's manifest gate over `tests/repro/admission.tsv`: the pin header
/// with the tier-1 gate fields (seed-pin, envelope), the closed verdict
/// set, every admitted (stable) row's dump companion committed, and NO
/// tier-2 fixture admitted — an admitted row whose dump's seed blocks
/// differ is exactly the tier-2-admitted defect.
pub(crate) fn check_manifest(
    manifest: &str,
    dump_of: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
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

/// Parse a checklist verdict table — rows `| N | item | verdict | ...`
/// with N in 1..=5 — into `(item number, verdict)` pairs.
pub(crate) fn checklist_verdicts(text: &str) -> Vec<(usize, String)> {
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
pub(crate) fn checklist_rows(text: &str) -> Vec<usize> {
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

/// Per-phase trace file (unique per process; the test body runs its
/// phases sequentially so the env mutations never race each other).
pub(crate) fn trace_file(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("gently-dt-p1-{}-{tag}", std::process::id()))
}

pub(crate) fn set_gate(stage: &str, file: &std::path::Path) {
    gently_core::trace::set_test_gate(Some(stage), Some(file));
}

pub(crate) fn clear_gate() {
    gently_core::trace::set_test_gate(None, None);
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
pub(crate) fn excl001() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let dumps_dir = root.join("tests/repro/dumps");

    let mut byte_divergent = 0usize;
    let mut byte_identical = 0usize;
    let mut skipped = 0usize;

    for entry in fs::read_dir(&dumps_dir).must("dumps dir must exist") {
        let path = entry.must("dump entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt")
            || !path.to_string_lossy().contains(".dump")
        {
            continue;
        }
        let name = path
            .file_stem()
            .must("value")
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
pub(crate) fn run_traced(tag: &str, source: &str, stage: &str) -> (String, String) {
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

/// Extract a probe body from probes.pl. Two committed forms:
///   `..._probe_body('<name>', <<'MARKER')` … MARKER
///   `    '<name>' => <<'MARKER',` … MARKER
pub(crate) fn probe_body(probes: &Path, name: &str) -> Option<String> {
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
pub(crate) fn claims_ascii(path: &Path) -> Option<String> {
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
