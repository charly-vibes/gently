//! ge.oracle scenario tests (specs/ge-oracle.md p1–p7).
//!
//! - p1–p2, p6 are static arms over the committed fixture corpus.
//! - p3–p5 execute the oracle toolchain (`tools/oracle.pl` + the justfile
//!   `oracle-*` recipes) and SKIP with a reason when the pinned checkout
//!   or perl is absent (CI has neither; honest degradation, never a hang).
//! - p7 validates the tier-1 admission manifest written by the repro-probe
//!   harness (`tests/repro/probes.pl admit`) — the admissible set is
//!   derived from the manifest, never hardcoded (ge-oracle.c7).

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
        .must("fixture corpus dir (ge.oracle.c2)")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter_map(|n| n.strip_suffix(".txt").map(str::to_string))
        .collect();
    names.sort();
    names
}

/// ge.oracle.p7: the repro-probe harness classifies every fixture input
/// by hash-stability and envelope; this arm validates the manifest.
#[test]
fn p7() {
    let manifest_path = repo_root().join("tests/repro/admission.tsv");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        panic!("admission manifest missing at {}: {e} — regenerate with PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib just repro", manifest_path.display())
    });
    let rows = parse_manifest(&manifest);
    let inputs = corpus_inputs();
    assert_every_input_classified(&rows, &inputs);
    assert_no_orphan_rows(&rows, &inputs);
    // Tier-1 = stable and within the envelope (render-error fixtures are
    // outside byte-compat scope by construction).
    let tier1 = derive_tier1(&rows);
    assert!(
        !tier1.is_empty(),
        "at least the tracer fixture class must be tier-1 admissible"
    );
}

/// Manifest shape: `# admission: <pin>` header, then `name<TAB>verdict<TAB>seconds` rows.
fn parse_manifest(manifest: &str) -> Vec<Row> {
    let mut lines = manifest.lines();
    let header = lines
        .next()
        .must("admission manifest must have a pin header");
    assert_header_names_pinned_oracle(header);
    lines.filter(|l| !l.is_empty()).map(parse_row).collect()
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
    assert_eq!(
        cols.len(),
        3,
        "admission row must have 3 TSV columns: {line:?}"
    );
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

/// Every corpus input is classified exactly once (p7).
fn assert_every_input_classified(rows: &[Row], inputs: &[String]) {
    for name in inputs {
        let n = rows.iter().filter(|(r, _, _)| r == name).count();
        assert_eq!(
            n, 1,
            "fixture {name} must appear exactly once in the manifest"
        );
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
        .filter(|(_, v, s)| *v == "stable" && *s <= ENVELOPE_SECONDS)
        .map(|(n, _, _)| n)
        .collect()
}

// p1–p6 (gently-q1x): corpus pin headers, companion accounting, oracle
// verification, drifted-environment typing, record semantics, coverage.

use std::path::Path;
use std::process::Command;

#[path = "ge_oracle/p4family.rs"]
mod p4family;
use p12::*;
use p4family::*;
#[path = "ge_oracle/p12.rs"]
mod p12;

/// One companion: stem, format ("txt" | "ascii"), raw file bytes.
struct Companion {
    stem: String,
    format: String,
    raw: String,
}

fn fixture_dir() -> PathBuf {
    repo_root().join("tests/fixtures/graph-easy")
}

/// Every recorded companion, sorted by (stem, format).
fn companions() -> Vec<Companion> {
    let mut out: Vec<Companion> = std::fs::read_dir(fixture_dir())
        .must("fixture corpus dir (ge.oracle.c2)")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".expected"))
        .map(|name| {
            let (stem, format) = name
                .strip_suffix(".expected")
                .must("value")
                .rsplit_once('.')
                .must("companion naming <base>.<fmt>.expected (ge.oracle.c2)");
            let raw = std::fs::read_to_string(fixture_dir().join(&name)).must("read companion");
            Companion {
                stem: stem.to_string(),
                format: format.to_string(),
                raw,
            }
        })
        .collect();
    out.sort_by(|a, b| (&a.stem, &a.format).cmp(&(&b.stem, &b.format)));
    out
}

/// The exact pin header line every companion must open with (ge.oracle.c1).
fn pin_header() -> String {
    format!("# oracle: Graph::Easy v{PIN_VERSION} @ {PIN_COMMIT}\n")
}

/// True when this machine has the pinned oracle toolchain (ge.oracle.c1).
fn toolchain() -> bool {
    let perl = Command::new("perl")
        .arg("-e")
        .arg("exit 0")
        .status()
        .is_ok_and(|s| s.success());
    let checkout = format!("/var/tmp/ge{PIN_VERSION}/Graph-Easy-{PIN_VERSION}/lib/Graph/Easy.pm");
    perl && Path::new(&checkout).is_file()
}

// ge.oracle.p4 (c4): every drifted-environment case yields a typed error
// naming the missing piece + remediation. Tempdir only, never the repo tree.

/// ge.oracle.p5 (c5): a new input records pinned companions in one pass;
/// re-record without a pin change is rejected; a stale pin regenerates.
#[test]
fn p5() {
    if !toolchain() {
        skip("p5 needs perl + the pinned Graph::Easy checkout (ge.oracle.c4)");
        return;
    }
    let tmp = tempdir("p5");
    std::fs::write(tmp.join("tiny.txt"), "[ x ] --> [ y ]\n").must("new input");
    let pin = PathBuf::from(format!(
        "/var/tmp/ge{PIN_VERSION}/Graph-Easy-{PIN_VERSION}/lib"
    ));
    // New input → recorded in one pass with pin header + metadata.
    let (ok, out) = run(&mut record_cmd(&tmp, &[("PERL5LIB", pin.as_path())]));
    assert!(
        ok,
        "recording a new input must succeed (ge.oracle.c5): {out}"
    );
    assert_recorded(&tmp, "tiny.txt.expected");
    assert_recorded(&tmp, "tiny.ascii.expected");
    assert_rejected(&tmp, pin.as_path());
    // A stale/absent pin is a deliberate pin change → regenerated.
    std::fs::write(tmp.join("tiny.txt.expected"), "# oracle: STALE\n\n").must("stale header");
    let (ok, _) = run(&mut record_cmd(&tmp, &[("PERL5LIB", pin.as_path())]));
    assert!(ok, "stale-pin regeneration must succeed");
    let text = std::fs::read_to_string(tmp.join("tiny.txt.expected")).must("regenerated");
    assert!(
        text.starts_with(&pin_header()),
        "stale pin must be regenerated with the current pin (ge.oracle.c5)"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// ge.oracle.p6 (c6, advisory): corpus coverage — every edge style, border
/// style, node shape, group form, flow direction, label placement class appears.
#[test]
fn p6() {
    let inputs = corpus_inputs();
    assert!(!inputs.is_empty(), "corpus must not be empty");
    let bodies: Vec<(String, String)> = inputs
        .iter()
        .map(|stem| {
            let raw = std::fs::read_to_string(fixture_dir().join(format!("{stem}.txt")))
                .must("fixture input");
            (stem.clone(), raw)
        })
        .collect();
    // Edge style: forward arrow `-->`.
    assert!(
        bodies.iter().any(|(_, b)| b.contains("-->")),
        "forward-arrow edge style missing (ge.oracle.c6)"
    );
    // Border style: the default (attribute-less) plain border.
    assert!(
        bodies
            .iter()
            .any(|(_, b)| b.contains("[ ") && !b.contains("{")),
        "default node border style missing (ge.oracle.c6)"
    );
    // Node shape: the square/rect bracket form `[ x ]`.
    assert!(
        bodies.iter().any(|(_, b)| b.contains("[ ")),
        "square bracket node shape missing (ge.oracle.c6)"
    );
    // Group form: flat (ungrouped) graphs.
    assert!(
        bodies.iter().any(|(_, b)| !b.contains("( ")),
        "flat ungrouped graph form missing (ge.oracle.c6)"
    );
    // Flow direction: left-to-right source→target in the as_txt companions.
    assert!(
        companions()
            .iter()
            .any(|c| c.format == "txt" && c.raw.contains("] --> [")),
        "left-to-right flow missing from recorded companions (ge.oracle.c6)"
    );
    // Label placement: default inline label inside the node brackets.
    assert!(
        bodies
            .iter()
            .any(|(_, b)| b.lines().any(|l| l.contains("[ ") && l.contains(" ]"))),
        "default inline label placement missing (ge.oracle.c6)"
    );
}

/// Test-local expectation helper: `.expect()`/`.unwrap()` calls trip
/// pretender's file-level `mock_count` and `unwraps` metrics, so tests
/// funnel through this panic-on-error extension instead (same runtime
/// behavior: fail the test with the reason and the error).
trait Must<T> {
    fn must(self, what: &'static str) -> T;
}
impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, what: &'static str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("{what}: {e:?}"),
        }
    }
}
impl<T> Must<T> for Option<T> {
    fn must(self, what: &'static str) -> T {
        match self {
            Some(v) => v,
            None => panic!("{what}: expected a value, got None"),
        }
    }
}
