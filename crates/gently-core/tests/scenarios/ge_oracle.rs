//! ge.oracle tier-1 admission (specs/ge-oracle.md c7/p7, gently-mwo).
//!
//! The repro-probe harness (`tests/repro/probes.pl admit`) classifies every
//! corpus fixture by the two tier-1 admission probes: hash-stability (the
//! pinned oracle renders the input identically across seeds) and envelope
//! (recorded within 10 s wall time). This module is the machine-checkable
//! arm: it validates the classification manifest against the corpus —
//! the admissible set is *derived* from the manifest, never hardcoded
//! (ge-oracle.c7: "the admissible set is defined by the probes, not by a
//! hardcoded count").

use std::path::PathBuf;

/// The pinned oracle (specs/ge-oracle.md c1) — must match tools/oracle.pl.
const PIN_VERSION: &str = "0.69";
const PIN_COMMIT: &str = "ededa3d787ad89ac532c578c06390e8a7b270499";
/// Tier-1 envelope: 10 s wall time (ge-oracle.c7).
const ENVELOPE_SECONDS: f64 = 10.0;

/// One manifest row: fixture stem, probe verdict, worst render seconds.
type Row = (String, String, f64);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Corpus input stems (no .txt extension) — the manifest keys fixtures by
/// stem.
fn corpus_inputs() -> Vec<String> {
    let fix = repo_root().join("tests/fixtures/graph-easy");
    let mut names: Vec<String> = std::fs::read_dir(&fix)
        .expect("fixture corpus dir (ge.oracle.c2)")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".txt"))
        .map(|n| n.trim_end_matches(".txt").to_string())
        .collect();
    names.sort();
    names
}

/// ge.oracle.p7: the corpus stability and envelope probes over every fixture
/// input — each fixture is classified hash-stable or hash-dependent and
/// inside or outside the envelope, and the classification is
/// machine-checkable by the repro-probe harness.
#[test]
fn p7() {
    let manifest_path = repo_root().join("tests/repro/admission.tsv");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        panic!(
            "admission manifest missing at {}: {e} — regenerate with \
             PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib just repro",
            manifest_path.display()
        )
    });
    let rows = parse_manifest(&manifest);

    let inputs = corpus_inputs();
    assert_every_input_classified(&rows, &inputs);
    assert_no_orphan_rows(&rows, &inputs);

    // Tier-1 membership is re-derived from the probe fields: stable and
    // within the envelope. render-error fixtures are outside byte-compat
    // scope by construction (the oracle cannot record them).
    let tier1 = derive_tier1(&rows);
    assert!(
        !tier1.is_empty(),
        "at least the tracer fixture class must be tier-1 admissible"
    );
}

/// Manifest shape: a `# admission: <pin> ...` header naming the pinned
/// oracle, then one `name<TAB>verdict<TAB>seconds` row per input.
fn parse_manifest(manifest: &str) -> Vec<Row> {
    let mut lines = manifest.lines();
    let header = lines
        .next()
        .expect("admission manifest must have a pin header");
    assert_header_names_pinned_oracle(header);
    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        rows.push(parse_row(line));
    }
    rows
}

fn assert_header_names_pinned_oracle(header: &str) {
    let expected = format!("# admission: Graph::Easy v{PIN_VERSION} @ ");
    assert!(
        header.starts_with(&expected) && header.contains(PIN_COMMIT),
        "admission manifest header must name the pinned oracle, got: {header}"
    );
}

fn parse_row(line: &str) -> Row {
    let cols: Vec<&str> = line.split('\t').collect();
    assert_eq!(cols.len(), 3, "admission row must have 3 TSV columns: {line:?}");
    assert_valid_verdict(cols[1]);
    let seconds = parse_seconds(line, cols[2]);
    (cols[0].to_string(), cols[1].to_string(), seconds)
}

/// The verdict set is closed: stable, hash-dependent, or render-error.
fn assert_valid_verdict(verdict: &str) {
    assert!(
        matches!(verdict, "stable" | "hash-dependent" | "render-error"),
        "admission verdict must be a closed set, got: {verdict:?}"
    );
}

fn parse_seconds(line: &str, raw: &str) -> f64 {
    let seconds: f64 = raw
        .parse()
        .unwrap_or_else(|e| panic!("admission seconds must be a number, got {e}: {line:?}"));
    assert!(
        seconds.is_finite() && seconds >= 0.0,
        "admission seconds must be finite and non-negative: {line:?}"
    );
    seconds
}

/// Every corpus input is classified, exactly once (p7: "over every fixture
/// input").
fn assert_every_input_classified(rows: &[Row], inputs: &[String]) {
    for name in inputs {
        let n = rows.iter().filter(|(r, _, _)| r == name).count();
        assert_eq!(n, 1, "fixture {name} must appear exactly once in the manifest");
    }
}

fn assert_no_orphan_rows(rows: &[Row], inputs: &[String]) {
    for (name, _, _) in rows {
        assert!(
            inputs.iter().any(|i| i == name),
            "manifest row {name} is not a corpus input"
        );
    }
}

fn derive_tier1(rows: &[Row]) -> Vec<&String> {
    rows.iter()
        .filter(|(_, verdict, seconds)| *verdict == "stable" && *seconds <= ENVELOPE_SECONDS)
        .map(|(name, _, _)| name)
        .collect()
}
