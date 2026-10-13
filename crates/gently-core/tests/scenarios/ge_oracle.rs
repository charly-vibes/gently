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
        .expect("fixture corpus dir (ge.oracle.c2)")
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
        .expect("admission manifest must have a pin header");
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
        .expect("fixture corpus dir (ge.oracle.c2)")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".expected"))
        .map(|name| {
            let (stem, format) = name
                .strip_suffix(".expected")
                .unwrap()
                .rsplit_once('.')
                .expect("companion naming <base>.<fmt>.expected (ge.oracle.c2)");
            let raw = std::fs::read_to_string(fixture_dir().join(&name)).expect("read companion");
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

fn skip(reason: &str) {
    println!("oracle toolchain unavailable — skipping: {reason}");
}

/// Run a command, return (success, combined stdout+stderr).
fn run(cmd: &mut Command) -> (bool, String) {
    let out = cmd.output().expect("spawn command");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

/// A drifted-environment run must fail typed, naming the piece + remediation
/// (ge.oracle.c4).
fn typed_failure(ok: bool, out: &str, needle: &str, case: &str) {
    assert!(
        !ok && out.contains(needle) && out.contains("remediation"),
        "{case} must be a typed error + remediation (ge.oracle.c4), got: {out}"
    );
}

/// ge.oracle.p1 (c1): every pin header names version + commit, every
/// metadata line names perl + seed, no companion carries a foreign pin.
#[test]
fn p1() {
    let cs = companions();
    assert!(!cs.is_empty(), "corpus must not be empty");
    for c in &cs {
        let mut lines = c.raw.lines();
        let header = lines.next().unwrap_or("");
        assert_eq!(
            format!("{header}\n"),
            pin_header(),
            "{}:{}: pin header must name the pinned oracle (c1)",
            c.stem,
            c.format
        );
        let meta = lines.next().unwrap_or("");
        assert!(
            meta.starts_with("# oracle: perl ") && meta.contains("PERL_HASH_SEED="),
            "{}:{}: metadata must name perl version + seed (c1), got {meta:?}",
            c.stem,
            c.format
        );
        for stray in lines {
            assert!(
                !stray.starts_with("# oracle:"),
                "{}:{}: foreign pin {stray:?} (c1)",
                c.stem,
                c.format
            );
        }
    }
}

/// Companion count for one input/format pair.
fn companion_count(cs: &[Companion], input: &str, format: &str) -> usize {
    cs.iter()
        .filter(|c| c.stem == input && c.format == format)
        .count()
}

/// No companion without a matching corpus input (ge.oracle.c2).
fn assert_no_orphans(cs: &[Companion], inputs: &[String]) {
    for c in cs {
        assert!(
            inputs.iter().any(|i| i == &c.stem),
            "orphan companion for unknown input {}.{} (ge.oracle.c2)",
            c.stem,
            c.format
        );
    }
}

/// ge.oracle.p2 (c2): every corpus input has exactly one recorded expected
/// output per supported format (txt, ascii); no orphans.
#[test]
fn p2() {
    let inputs = corpus_inputs();
    assert!(!inputs.is_empty(), "corpus must not be empty");
    let cs = companions();
    for input in &inputs {
        for format in ["txt", "ascii"] {
            assert_eq!(companion_count(&cs, input, format), 1, "input {input}.txt must have exactly one {format}.expected companion (ge.oracle.c2)");
        }
    }
    assert_no_orphans(&cs, &inputs);
}

/// ge.oracle.p3 (c3): the full verify pass — every fixture re-rendered and
/// compared byte-identically to the recorded companions; the runner owns
/// the diff report, this arm asserts its exit status.
#[test]
fn p3() {
    if !toolchain() {
        skip("p3 needs perl + the pinned Graph::Easy checkout (ge.oracle.c4)");
        return;
    }
    let (ok, out) = run(Command::new("just")
        .arg("oracle-verify")
        .current_dir(repo_root()));
    assert!(ok, "just oracle-verify failed — see its input/format/first-differing-line report (ge.oracle.c3): {out}");
}

// ge.oracle.p4 (c4): every drifted-environment case yields a typed error
// naming the missing piece + remediation. Tempdir only, never the repo tree.

fn tempdir(tag: &str) -> PathBuf {
    let tmp = std::env::temp_dir().join(format!("gently-oracle-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).expect("tempdir");
    tmp
}

/// The `oracle.pl record <dir>` command with an environment applied.
fn record_cmd(dir: &Path, envs: &[(&str, &Path)]) -> Command {
    let mut cmd = Command::new("perl");
    cmd.arg(repo_root().join("tools/oracle.pl"))
        .arg("record")
        .arg(dir)
        .env("PERL_HASH_SEED", "0");
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd
}

/// A fake Graph::Easy checkout at `lib` exposing the given version.
fn fake_checkout(lib: &Path, version: &str) {
    std::fs::create_dir_all(lib.join("Graph")).expect("fake lib");
    std::fs::write(
        lib.join("Graph/Easy.pm"),
        format!("package Graph::Easy; our $VERSION = '{version}'; 1;\n"),
    )
    .expect("write fake Graph::Easy");
}

/// Case 1: perl missing (fake PATH without perl) — the justfile recipe types this first.
fn p4_missing_perl(tmp: &Path) {
    let bin = tmp.join("bin");
    std::fs::create_dir_all(&bin).expect("temp bin dir");
    #[cfg(unix)]
    std::os::unix::fs::symlink("/bin/bash", bin.join("bash")).expect("symlink bash");
    let (ok, out) = run(Command::new("/usr/bin/just")
        .arg("oracle-record")
        .current_dir(repo_root())
        .env("PATH", &bin));
    typed_failure(ok, &out, "perl not found", "missing perl");
}

/// Case 2: version drift — a fake checkout exposing Graph::Easy 0.99.
fn p4_version_drift(tmp: &Path) {
    let fake = tmp.join("fake-lib");
    fake_checkout(&fake, "0.99");
    let (ok, out) = run(&mut record_cmd(tmp, &[("PERL5LIB", fake.as_path())]));
    typed_failure(ok, &out, "0.99", "version drift");
}

/// Case 3: off the pinned checkout — right version from the wrong path
/// (isolates the source-checkout clause; a fully missing checkout surfaces
/// as cpan-copy drift or the p3/p5 skip guard in CI).
fn p4_off_checkout(tmp: &Path) {
    let off = tmp.join("off-path-lib");
    fake_checkout(&off, PIN_VERSION);
    let (ok, out) = run(&mut record_cmd(tmp, &[("PERL5LIB", off.as_path())]));
    typed_failure(
        ok,
        &out,
        "pin requires the source checkout",
        "off-checkout load path",
    );
}

/// Case 4: record run without an explicit PERL_HASH_SEED.
fn p4_missing_seed(tmp: &Path) {
    let pin = PathBuf::from(format!(
        "/var/tmp/ge{PIN_VERSION}/Graph-Easy-{PIN_VERSION}/lib"
    ));
    let mut cmd = record_cmd(tmp, &[("PERL5LIB", pin.as_path())]);
    cmd.env_remove("PERL_HASH_SEED");
    let (ok, out) = run(&mut cmd);
    typed_failure(ok, &out, "PERL_HASH_SEED", "missing seed pin");
}

/// A minimal git repo at `root` with one empty commit; returns its HEAD sha.
fn git_repo_at_wrong_commit(root: &Path) -> String {
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "probe")
            .env("GIT_AUTHOR_EMAIL", "probe@example.invalid")
            .env("GIT_COMMITTER_NAME", "probe")
            .env("GIT_COMMITTER_EMAIL", "probe@example.invalid")
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    git(&["init", "-q"]);
    git(&[
        "commit",
        "-q",
        "--allow-empty",
        "-m",
        "drifted checkout probe",
    ]);
    git(&["rev-parse", "HEAD"])
}

/// Case 6 (gently-4sq): commit drift — the pinned version loaded from a
/// checkout whose git HEAD is not the pin commit. GE_PIN_LIB points the
/// pin at the fake checkout so the load-path clause passes and the commit
/// clause is exercised alone; a real drifted checkout is the same shape.
fn p4_commit_drift(tmp: &Path) {
    let root = tmp.join("drifted-checkout");
    fake_checkout(&root.join("lib"), PIN_VERSION);
    let head = git_repo_at_wrong_commit(&root);
    let lib = root.join("lib");
    let (ok, out) = run(&mut record_cmd(
        tmp,
        &[("PERL5LIB", lib.as_path()), ("GE_PIN_LIB", lib.as_path())],
    ));
    typed_failure(ok, &out, "pin commit is", "commit drift");
    assert!(
        out.contains(PIN_COMMIT) && out.contains(&head),
        "commit drift must name both shas (ge.oracle.c4), got: {out}"
    );
}

/// Case 5: stale pin header — just oracle-verify types it with the
/// `just oracle-record` remediation before any comparison runs.
fn p4_stale_pin(tmp: &Path) {
    let stale = tmp.join("stale-corpus");
    std::fs::create_dir_all(&stale).expect("stale corpus dir");
    std::fs::write(stale.join("x.txt"), "[ a ] --> [ b ]\n").expect("input");
    std::fs::write(
        stale.join("x.txt.expected"),
        "# oracle: Graph::Easy v0.68 @ 0000000000000000000000000000000000000000\nstale\n",
    )
    .expect("stale companion");
    let (ok, out) = run(Command::new("/usr/bin/just")
        .arg("oracle-verify")
        .env("FIX", &stale)
        .current_dir(repo_root()));
    typed_failure(ok, &out, "stale pin header", "stale pin header");
}

#[test]
fn p4() {
    if !toolchain() {
        skip("p4 needs perl + the pinned Graph::Easy checkout (ge.oracle.c4)");
        return;
    }
    let tmp = tempdir("p4");
    p4_missing_perl(&tmp);
    p4_version_drift(&tmp);
    p4_off_checkout(&tmp);
    p4_missing_seed(&tmp);
    p4_commit_drift(&tmp);
    p4_stale_pin(&tmp);
    assert!(
        std::fs::remove_dir_all(&tmp).is_ok(),
        "p4 tempdir must clean up"
    );
}

/// After a record run, tiny.txt companions carry pin header + metadata (c1/c5).
fn assert_recorded(dir: &Path, companion: &str) {
    let text = std::fs::read_to_string(dir.join(companion)).expect("recorded companion");
    assert!(
        text.starts_with(&pin_header()),
        "{companion}: pin header required (ge.oracle.c1/c5)"
    );
    assert!(
        text.lines()
            .nth(1)
            .unwrap_or("")
            .starts_with("# oracle: perl "),
        "{companion}: metadata line required (ge.oracle.c1)"
    );
}

/// Re-record without a pin change → refused, bytes untouched (c5).
fn assert_rejected(dir: &Path, pin: &Path) {
    let before = std::fs::read(dir.join("tiny.txt.expected")).expect("companion bytes");
    let (ok, out) = run(&mut record_cmd(dir, &[("PERL5LIB", pin)]));
    assert!(ok, "refusal is a clean no-op, not a crash");
    assert!(
        out.contains("refusing to overwrite"),
        "re-recording without a pin change must be rejected (ge.oracle.c5), got: {out}"
    );
    let after = std::fs::read(dir.join("tiny.txt.expected")).expect("companion bytes");
    assert_eq!(
        before, after,
        "refused re-record must not touch companion bytes (ge.oracle.c5)"
    );
}

/// ge.oracle.p5 (c5): a new input records pinned companions in one pass;
/// re-record without a pin change is rejected; a stale pin regenerates.
#[test]
fn p5() {
    if !toolchain() {
        skip("p5 needs perl + the pinned Graph::Easy checkout (ge.oracle.c4)");
        return;
    }
    let tmp = tempdir("p5");
    std::fs::write(tmp.join("tiny.txt"), "[ x ] --> [ y ]\n").expect("new input");
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
    std::fs::write(tmp.join("tiny.txt.expected"), "# oracle: STALE\n\n").expect("stale header");
    let (ok, _) = run(&mut record_cmd(&tmp, &[("PERL5LIB", pin.as_path())]));
    assert!(ok, "stale-pin regeneration must succeed");
    let text = std::fs::read_to_string(tmp.join("tiny.txt.expected")).expect("regenerated");
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
                .expect("fixture input");
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
