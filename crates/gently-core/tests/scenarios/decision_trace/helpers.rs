use super::dump::seed_blocks;
use super::oracle::run_traced;
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
    check_manifest_header(header)?;
    for line in lines.filter(|l| !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 3 {
            return Err(dump_defect(
                "admission manifest row is not a name<TAB>verdict<TAB>seconds triple",
            ));
        }
        check_manifest_row(cols[0], cols[1], dump_of)?;
    }
    Ok(())
}

/// The pin header with the tier-1 gate fields (seed-pin, envelope).
fn check_manifest_header(header: &str) -> Result<(), String> {
    let pinned = header.starts_with(&format!("# admission: Graph::Easy v0.69 @ {PIN_COMMIT}"))
        && header.contains("seed-pin=pinned")
        && header.contains("envelope=");
    if pinned {
        Ok(())
    } else {
        Err(dump_defect(
            "admission manifest header missing the pin or the tier-1 gate \
             fields (seed-pin, envelope)",
        ))
    }
}

/// One manifest row: the verdict must be from the closed set, and an
/// admitted (stable) row must carry a dump companion whose seed blocks
/// agree (a tier-2 hash-seed-dependent fixture may not be admitted).
pub(crate) fn check_manifest_row(
    name: &str,
    verdict: &str,
    dump_of: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    if !is_closed_set_verdict(verdict) {
        return Err(dump_defect(&format!(
            "{name}: verdict '{verdict}' is outside the closed set \
             (stable, hash-dependent, render-error)"
        )));
    }
    if verdict != "stable" {
        return Ok(());
    }
    check_stable_companion(name, dump_of)
}

fn is_closed_set_verdict(verdict: &str) -> bool {
    matches!(verdict, "stable" | "hash-dependent" | "render-error")
}

/// An admitted (stable) row must carry a dump companion whose seed
/// blocks agree — a tier-2 (hash-seed-dependent) fixture may not be
/// admitted.
fn check_stable_companion(
    name: &str,
    dump_of: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    let dump = dump_of(name).ok_or_else(|| {
        dump_defect(&format!(
            "{name}: admitted (stable) but its dump companion is missing"
        ))
    })?;
    let blocks = seed_blocks(&dump);
    if seed_blocks_agree(&blocks) {
        Ok(())
    } else {
        Err(dump_defect(&format!(
            "{name}: admitted (stable) but its dump's seed blocks differ — \
             a tier-2 (hash-seed-dependent) fixture is admitted"
        )))
    }
}

fn seed_blocks_agree(blocks: &[Vec<String>]) -> bool {
    blocks.len() < 2 || blocks.iter().all(|b| *b == blocks[0])
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

    let mut byte_divergent = 0usize;
    let mut byte_identical = 0usize;
    let mut skipped = 0usize;

    for entry in fs::read_dir(root.join("tests/repro/dumps")).must("dumps dir must exist") {
        let path = entry.must("dump entry").path();
        if !is_dump_file(&path) {
            continue;
        }
        let name = path
            .file_stem()
            .must("value")
            .to_string_lossy()
            .replace(".dump", "");
        let dump = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
        match excl001_one(&root, &name, &dump) {
            Excl001Outcome::Divergent => byte_divergent += 1,
            Excl001Outcome::Identical => byte_identical += 1,
            Excl001Outcome::Skipped => skipped += 1,
        }
    }

    assert_excl001_scope(byte_divergent, byte_identical, skipped);
}

/// A dump file: `.txt` extension carrying a `.dump` infix.
fn is_dump_file(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("txt")
        && path.to_string_lossy().contains(".dump")
}

fn assert_excl001_scope(divergent: usize, identical: usize, skipped: usize) {
    assert!(
        divergent > 0,
        "EXCL-001 scope must contain at least one byte-divergent fixture class"
    );
    assert!(
        skipped <= 1,
        "at most one dump class may lack a comparable oracle-bytes form, skipped {skipped}"
    );
    assert!(
        divergent + identical >= 23,
        "the dump corpus must be iterated (all comparable dumps), got \
         {divergent}+{identical}, skipped {skipped}"
    );
}

enum Excl001Outcome {
    Divergent,
    Identical,
    Skipped,
}

/// Run the EXCL-001 contract for one dump class; classify its outcome.
fn excl001_one(root: &std::path::Path, name: &str, dump: &str) -> Excl001Outcome {
    let Some(oracle_bytes) = super::oracle::oracle_bytes_for(root, name) else {
        // dot-bare-scope-in-named.observed is a multi-case OK log with no
        // seed/as_ascii form — not comparable here.
        return Excl001Outcome::Skipped;
    };
    let input = super::oracle::fixture_input(root, name);

    // Run gently with the layout.place gate on.
    let (gently_bytes, gently_trace) = run_traced(name, &input, "layout.place");

    let divergent = gently_bytes.trim_end() != oracle_bytes.trim_end();
    let aligned = gently_core::trace_diff::align_dump(dump);
    super::oracle::assert_trace_contract(
        name,
        &oracle_bytes,
        &gently_bytes,
        &aligned,
        &gently_trace,
        divergent,
    );
    if divergent {
        Excl001Outcome::Divergent
    } else {
        Excl001Outcome::Identical
    }
}
