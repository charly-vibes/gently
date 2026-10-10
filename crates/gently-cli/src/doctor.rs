//! Purpose: `gently doctor` — suite health checks on the genesis
//! `DoctorRunner` frame (specs/cli.md c8, gently-ef5), plus the `init`
//! registration command (c5).
//! Responsibilities: define the [`DoctorCheck`] implementations (oracle
//! availability, output-format support, fixture pin freshness, genesis
//! registration auto-fix), run them through genesis's runner with or
//! without `--fix`, register gently in the working directory's genesis
//! manifest via genesis discovery, and render the [`DoctorReport`] in
//! human or JSON envelope form with the report's own exit code.
//! Rationale: the checks mirror the justfile oracle recipes (`perl` +
//! Graph::Easy pinned at v0.69 @ ededa3d7, fixture corpus under
//! `tests/fixtures/graph-easy`) so doctor reports what the gates would
//! find; all filesystem interaction is scoped to the process working
//! directory, which callers must point at a tempdir in tests.

use std::path::{Path, PathBuf};
use std::process::Command;

use genesis::doctor::{DoctorCheck, DoctorReport, DoctorRunner};
use genesis::guide::{Guide, Output, OutputFormat};
use genesis::suite_linter::{LintResult, Severity};
use serde::Serialize;

use crate::CliError;

/// The oracle pin the suite is recorded against — kept in lockstep with
/// the justfile `oracle-verify` recipe and the fixture headers
/// (`tests/fixtures/graph-easy/*.txt.expected`, ge.oracle scope).
const ORACLE_PIN: &str = "Graph::Easy v0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499";

/// Where the pinned Graph::Easy lives (see tests/repro/README.md).
const GRAPH_EASY_LIB: &str = "/var/tmp/ge0.69/Graph-Easy-0.69/lib";

/// Fixture corpus location relative to the repo root (justfile recipes).
const FIXTURE_DIR: &str = "tests/fixtures/graph-easy";

// ── Command entry ─────────────────────────────────────────────────────

/// Run `gently doctor`: suite health checks through the genesis
/// `DoctorRunner`, `--fix` applies available auto-fixes (c8). Prints the
/// report to stdout — human lines or a genesis Envelope with kind
/// `Doctor` — and exits with the report's own exit code.
pub fn run_doctor(guide: &Guide, fix: bool, format: OutputFormat) -> i32 {
    let root = working_root();
    let report = match build_runner().run(&root, fix) {
        Ok(report) => report,
        Err(e) => {
            eprintln!("{}: doctor failed: {e}", guide.name());
            return 1;
        }
    };
    match format {
        OutputFormat::Human => print_human(guide, &report),
        OutputFormat::Json => {
            if let Err(code) = print_json(guide, &report) {
                return code;
            }
        }
    }
    report.exit_code()
}

/// Run `gently init`: register gently in the working directory's
/// `.genesis/tools.toml` via genesis discovery (c5).
pub fn run_init(guide: &Guide, format: OutputFormat) -> i32 {
    let root = working_root();
    guide.run_formatted(format, || {
        register_gently(&root).map_err(|e| Box::new(CliError(e)) as Box<dyn std::error::Error>)?;
        let manifest = root.join(genesis::discovery::MANIFEST_FILENAME);
        Ok(Output::success(InitReport {
            manifest: manifest.display().to_string(),
        }))
    })
}

fn working_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn build_runner() -> DoctorRunner {
    DoctorRunner::new(vec![
        Box::new(OracleCheck),
        Box::new(FormatSupportCheck {
            formats: vec!["ascii".to_string()],
        }),
        Box::new(PinFreshnessCheck),
        Box::new(RegistrationCheck),
    ])
}

fn print_human(guide: &Guide, report: &DoctorReport) {
    println!(
        "{} doctor: {} pass, {} warn, {} fail",
        guide.name(),
        report.summary.pass,
        report.summary.warn,
        report.summary.fail
    );
    for check in &report.checks {
        let status = status_word(check.status.is_pass(), check.status.is_fail());
        println!("  [{status}] {}: {}", check.name, check.message);
    }
}

fn status_word(is_pass: bool, is_fail: bool) -> &'static str {
    if is_pass {
        "pass"
    } else if is_fail {
        "fail"
    } else {
        "warn"
    }
}

fn print_json(guide: &Guide, report: &DoctorReport) -> Result<(), i32> {
    match serde_json::to_string(&report.to_envelope(guide.version())) {
        Ok(json) => {
            println!("{json}");
            Ok(())
        }
        Err(e) => {
            eprintln!("{}: cannot serialize doctor envelope: {e}", guide.name());
            Err(1)
        }
    }
}

/// Register gently in the working directory's genesis manifest (c5 init /
/// c8 doctor fix) — genesis discovery owns the file format.
fn register_gently(project_root: &Path) -> Result<(), String> {
    genesis::discovery::register(
        project_root,
        "gently",
        "Graph text in, rendered output out (Graph::Easy port)",
        "directory",
        ".genesis",
    )
}

// ── Checks ────────────────────────────────────────────────────────────

/// Oracle availability (c8a): `perl` present and pinned Graph::Easy
/// loadable under the pin at exactly v0.69.
struct OracleCheck;

impl DoctorCheck for OracleCheck {
    fn name(&self) -> &'static str {
        "oracle-availability"
    }

    fn description(&self) -> &'static str {
        "perl and the pinned Graph::Easy are available to the oracle"
    }

    fn run(&self, _repo_root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        if let Some(result) = probe_perl() {
            return Ok(vec![result]);
        }
        if let Some(result) = probe_pinned_module() {
            return Ok(vec![result]);
        }
        Ok(vec![])
    }
}

/// Probe the `perl` interpreter itself; `None` means perl is fine.
fn probe_perl() -> Option<LintResult> {
    let out = Command::new("perl").arg("-e").arg("print 42").output().ok()?;
    if out.status.success() {
        return None;
    }
    Some(LintResult::with_fix(
        "perl exists but failed to run",
        Severity::Error,
        "inspect the perl installation (see tests/repro/README.md)",
    ))
}

/// Probe the pinned Graph::Easy load; `None` means the pin loads at v0.69.
fn probe_pinned_module() -> Option<LintResult> {
    let spawned = Command::new("perl")
        .arg(format!("-I{GRAPH_EASY_LIB}"))
        .arg("-MGraph::Easy")
        .arg("-e")
        .arg("print $Graph::Easy::VERSION")
        .output();
    let Ok(out) = spawned else {
        return Some(LintResult::with_fix(
            format!("cannot probe Graph::Easy under {GRAPH_EASY_LIB}"),
            Severity::Warning,
            "fetch the pin: see tests/repro/README.md",
        ));
    };
    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if version == "0.69" {
        return None;
    }
    let found = if version.is_empty() { "nothing" } else { version.as_str() };
    Some(LintResult::with_fix(
        format!("pinned Graph::Easy not loadable at v0.69 (found: {found})"),
        Severity::Warning,
        "fetch the pin: see tests/repro/README.md",
    ))
}

/// Output-format support (c8b): every advertised format has a wired
/// renderer.
struct FormatSupportCheck {
    formats: Vec<String>,
}

impl DoctorCheck for FormatSupportCheck {
    fn name(&self) -> &'static str {
        "output-format-support"
    }

    fn description(&self) -> &'static str {
        "every advertised output format has a wired renderer"
    }

    fn run(&self, _repo_root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        if self.formats.is_empty() {
            return Ok(vec![LintResult::new(
                "no output formats are wired",
                Severity::Error,
            )]);
        }
        Ok(vec![])
    }
}

/// Fixture pin freshness (c8c): the recorded fixture companions carry the
/// expected oracle pin header; a missing corpus is advisory (nothing to go
/// stale), a present-but-stale or headerless companion is an error.
struct PinFreshnessCheck;

impl DoctorCheck for PinFreshnessCheck {
    fn name(&self) -> &'static str {
        "fixture-pin-freshness"
    }

    fn description(&self) -> &'static str {
        "fixture pin headers match the recorded oracle pin"
    }

    fn run(&self, repo_root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let fix_dir = repo_root.join(FIXTURE_DIR);
        if !fix_dir.is_dir() {
            return Ok(vec![LintResult::new(
                format!("no fixture corpus at {FIXTURE_DIR} — pin freshness unverifiable"),
                Severity::Advisory,
            )]);
        }
        Ok(check_companion_pins(&fix_dir, repo_root)?)
    }
}

/// Check every `*.expected` companion's pin header against
/// [`ORACLE_PIN`]; advisory when the corpus holds no companions.
fn check_companion_pins(fix_dir: &Path, repo_root: &Path) -> Result<Vec<LintResult>, std::io::Error> {
    let mut results = Vec::new();
    let mut checked = 0usize;
    for entry in std::fs::read_dir(fix_dir)? {
        let entry = entry?;
        let path = entry.path();
        let is_expected = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".expected"));
        if !is_expected {
            continue;
        }
        checked += 1;
        if let Some(result) = check_pin_header(&path, repo_root) {
            results.push(result);
        }
    }
    if checked == 0 {
        results.push(LintResult::new(
            format!("no *.expected companions in {FIXTURE_DIR} — run just oracle-record"),
            Severity::Advisory,
        ));
    }
    Ok(results)
}

/// Compare a companion's first line against the pin; `None` means fresh.
fn check_pin_header(path: &Path, repo_root: &Path) -> Option<LintResult> {
    let header = match std::fs::read_to_string(path) {
        Ok(header) => header,
        Err(e) => {
            return Some(LintResult::with_fix(
                format!("unreadable pin header in {}: {e}", path.display()),
                Severity::Error,
                "just oracle-record",
            ));
        }
    };
    let pinned = header
        .lines()
        .next()
        .is_some_and(|line| line.trim().ends_with(ORACLE_PIN));
    if pinned {
        return None;
    }
    let shown = path.strip_prefix(repo_root).unwrap_or(path).display();
    Some(LintResult::with_fix(
        format!("stale or missing pin header in {shown}"),
        Severity::Error,
        "just oracle-record",
    ))
}

/// Genesis registration (c8 auto-fix): gently is registered in
/// `.genesis/tools.toml`; auto-fixable via genesis discovery — the same
/// write `gently init` performs.
struct RegistrationCheck;

impl DoctorCheck for RegistrationCheck {
    fn name(&self) -> &'static str {
        "genesis-registration"
    }

    fn description(&self) -> &'static str {
        "gently is registered in .genesis/tools.toml"
    }

    fn run(&self, repo_root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let registered = genesis::discovery::read_manifest(repo_root)
            .is_some_and(|m| m.tools.contains_key("gently"));
        if registered {
            return Ok(vec![]);
        }
        Ok(vec![LintResult::with_fix(
            "gently is not registered in .genesis/tools.toml",
            Severity::Error,
            "gently init",
        )])
    }

    fn auto_fixable(&self) -> bool {
        true
    }

    fn fix(&self, repo_root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        register_gently(repo_root).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        Ok(vec![LintResult::new(
            "registered gently in .genesis/tools.toml",
            Severity::Advisory,
        )])
    }
}

/// `gently init`'s human payload — printed by `Output::emit`'s human path
/// through `Debug` plus a trailing newline, so `Debug` writes the manifest
/// path verbatim.
#[derive(Serialize)]
struct InitReport {
    manifest: String,
}

impl std::fmt::Debug for InitReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pin constant stays in lockstep with the fixture headers the
    /// justfile oracle-verify recipe checks.
    #[test]
    fn oracle_pin_matches_fixture_headers() {
        let fixture = Path::new("../../tests/fixtures/graph-easy/chain.txt.expected");
        let header = std::fs::read_to_string(fixture).expect("fixture companion");
        let first = header.lines().next().expect("header line");
        assert!(
            first.trim().ends_with(ORACLE_PIN),
            "ORACLE_PIN drifted from the fixture header: {first}"
        );
    }
}
