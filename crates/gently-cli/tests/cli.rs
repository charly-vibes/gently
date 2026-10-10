//! Integration tests for the `gently` binary — the end-to-end tracer
//! pipeline (epic gently-2po, tb.cli / gently-2po.9), wired to the cli
//! capability contracts (`scenarios::cli::p1`–`p4`, gently-0h9).
//!
//! Binding constraints: `specs/cli.md` c1 (stdin / positional
//! `[inputfile [outputfile]]`, extras ignored), c2 (`--as` format +
//! `--output` file), c3 (stdout-vs-stderr streams, exit 0/nonzero), c4
//! (unknown format → diagnostic naming requested and valid formats, exit
//! 255 — bug-for-bug with the upstream script's uncaught `die`, re-derived
//! under the ge.oracle.c7 scope contract, gently-ghh). Flag roles follow
//! upstream bin/graph-easy v0.69. The ascii oracle is pinned from
//! Graph::Easy v0.69 @ ededa3d7 (`add_edge("a","b"); as_ascii`).
//!
//! Deepened by gently-ef5 (--json envelope, verbosity surface, init/doctor)
//! and gently-bzx (full parser capability).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Like [`run_gently`], but spawns `gently` with `dir` as its working
/// directory — required for filesystem-touching surfaces (`init`, `doctor`)
/// so they never write into the repo working tree (gently-ef5).
fn run_gently_in(dir: &Path, args: &[&str], stdin: &[u8]) -> Run {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gently"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn gently");
    if let Err(e) = child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(stdin)
    {
        assert_eq!(e.kind(), std::io::ErrorKind::BrokenPipe, "write stdin");
    }
    let out = child.wait_with_output().expect("wait for gently");
    Run {
        stdout: out.stdout,
        stderr: out.stderr,
        code: out.status.code(),
    }
}

/// A fresh tempdir scoped to this test process, cleaned up on drop.
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!("gently-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Pinned oracle (Graph::Easy v0.69 @ ededa3d7, recorded 2026-10-08):
/// `perl -IGraph-Easy-0.69/lib -MGraph::Easy -e 'my $g = Graph::Easy->new;
/// $g->add_edge("a","b"); print $g->as_ascii'`.
const ORACLE: &[u8] = b"+---+     +---+\n| a | --> | b |\n+---+     +---+\n";

/// One `gently` invocation's observable process behavior (cli.c3/c4).
struct Run {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    code: Option<i32>,
}

fn run_gently(args: &[&str], stdin: &[u8]) -> Run {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gently"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn gently");
    // The child may exit before consuming stdin (e.g. unknown --as format
    // dies with 255 without reading — cli.c4) — a broken pipe is that
    // exit's observable signature, not a test failure. Any other IO error
    // still panics; correctness of p1-p3 is guarded by their assertions on
    // stdout/stderr/exit code.
    if let Err(e) = child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(stdin)
    {
        assert_eq!(
            e.kind(),
            std::io::ErrorKind::BrokenPipe,
            "write stdin: unexpected IO error"
        );
    }
    let out = child.wait_with_output().expect("wait for gently");
    Run {
        stdout: out.stdout,
        stderr: out.stderr,
        code: out.status.code(),
    }
}

/// Contract scenarios for `specs/cli.md`, one module per property row —
/// the filter `scenarios::cli::pN` used by `.espectacular/cli/pN.toml`
/// matches these test paths.
mod scenarios {
    mod cli {
        use super::super::{run_gently, run_gently_in, TempDir, ORACLE};
        use std::process::{Command, Stdio};

        /// cli.p1 (c1): stdin and the first file argument reach the parser
        /// with the same bytes; the second positional names the output file;
        /// further positionals are ignored.
        #[test]
        fn p1() {
            // stdin: no file arguments → input is read from stdin.
            let r = run_gently(&[], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(0), r.code, "exit 0 on success");
            assert_eq!(ORACLE, &r.stdout[..], "stdout must match the oracle bytes");
            assert!(
                r.stderr.is_empty(),
                "stderr must be empty on success, got: {}",
                String::from_utf8_lossy(&r.stderr)
            );

            // file argument: same bytes through the file path.
            let dir = std::env::temp_dir().join(format!("gently-p1-{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("create temp dir");
            let input = dir.join("in.txt");
            std::fs::write(&input, b"[ a ] --> [ b ]\n").expect("write fixture file");
            let r = run_gently(&[input.to_str().unwrap()], b"");
            assert_eq!(Some(0), r.code, "exit 0 on success via file argument");
            assert_eq!(ORACLE, &r.stdout[..], "file input must render the same bytes");
            assert!(r.stderr.is_empty(), "stderr must be empty via file argument");

            // second positional names the output file; extra positionals
            // are silently ignored (bug-for-bug with upstream).
            let out = dir.join("out");
            let extra = dir.join("extra.txt");
            std::fs::write(&extra, b"garbage that must be ignored\n").expect("write extra");
            let r = run_gently(
                &[input.to_str().unwrap(), out.to_str().unwrap(), extra.to_str().unwrap()],
                b"",
            );
            assert_eq!(Some(0), r.code, "extra positionals are ignored, not parsed");
            assert!(r.stdout.is_empty(), "output goes to the file, not stdout");
            assert_eq!(ORACLE, std::fs::read(&out).expect("read output file").as_slice());
            let _ = std::fs::remove_dir_all(&dir);
        }

        /// cli.p2 (c2): each requested format renders through its renderer;
        /// the default is ascii; `--output` receives the rendered bytes
        /// instead of stdout.
        #[test]
        fn p2() {
            // default (no --as) is ascii.
            let r = run_gently(&[], b"[ a ] --> [ b ]\n");
            assert_eq!(ORACLE, &r.stdout[..], "default output must be ascii");

            // --as ascii selects the renderer explicitly.
            let r = run_gently(&["--as", "ascii"], b"[ a ] --> [ b ]\n");
            assert_eq!(ORACLE, &r.stdout[..], "--as ascii must render ascii");

            // --output names the output file and receives the bytes.
            let dir = std::env::temp_dir().join(format!("gently-p2-{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("create temp dir");
            let input = dir.join("in.txt");
            let out = dir.join("out");
            std::fs::write(&input, b"[ a ] --> [ b ]\n").expect("write fixture");
            let r = run_gently(
                &[
                    "--as", "ascii",
                    "--output", out.to_str().unwrap(),
                    input.to_str().unwrap(),
                ],
                b"",
            );
            assert_eq!(Some(0), r.code, "file output must exit 0");
            assert!(r.stdout.is_empty(), "output goes to the file, not stdout");
            assert_eq!(ORACLE, std::fs::read(&out).expect("read output file").as_slice());
            let _ = std::fs::remove_dir_all(&dir);
        }

        /// cli.p3 (c3): streams and exit codes — success prints bytes to
        /// stdout with empty stderr and exit 0; failure prints a diagnostic
        /// to stderr, keeps stdout empty, and exits nonzero.
        #[test]
        fn p3() {
            let r = run_gently(&[], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(0), r.code, "success exits 0");
            assert_eq!(ORACLE, &r.stdout[..]);
            assert!(r.stderr.is_empty(), "success keeps stderr empty");

            let r = run_gently(&[], b"this is not graph text\n");
            assert_eq!(Some(1), r.code, "user-facing parse errors exit 1");
            assert!(r.stdout.is_empty(), "diagnostics must not touch stdout");
            let err = String::from_utf8_lossy(&r.stderr);
            assert!(!err.is_empty(), "a diagnostic must be printed to stderr");
            assert!(
                err.contains("parse error"),
                "diagnostic should name the failure kind: {err}"
            );
        }

        /// cli.p4 (c4): an unknown output format exits 255 with a diagnostic
        /// on stderr naming both the requested format and the valid formats;
        /// stdout stays empty. Exit 255 is bug-for-bug with the upstream
        /// script, whose unknown `--as` dies calling the missing
        /// `as_<fmt>` method (re-derived against upstream v0.69, gently-0h9).
        #[test]
        fn p4() {
            let r = run_gently(&["--as", "html"], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(255), r.code, "unknown format must exit 255");
            assert!(r.stdout.is_empty(), "diagnostics must not touch stdout");
            let err = String::from_utf8_lossy(&r.stderr);
            assert!(err.contains("html"), "diagnostic must name the requested format: {err}");
            assert!(err.contains("ascii"), "diagnostic must name the valid formats: {err}");
        }

        /// cli.p5 (c5): genesis foundation — `init` registers gently in
        /// `.genesis/tools.toml` of the working directory (tempdir, never
        /// the repo), verbosity is honored (`-v` adds a verbose diagnostic
        /// on stderr; `--quiet` suppresses non-error output), and default
        /// human dispatch is unchanged.
        #[test]
        fn p5() {
            // init: registers the tool entry in the tempdir's manifest.
            let dir = TempDir::new("p5-init");
            let r = run_gently_in(&dir.0, &["init"], b"");
            assert_eq!(Some(0), r.code, "init must exit 0: {}", String::from_utf8_lossy(&r.stderr));
            let manifest = dir.0.join(".genesis/tools.toml");
            let body = std::fs::read_to_string(&manifest)
                .expect("init must write .genesis/tools.toml in the working directory");
            assert!(body.contains("[tools.gently]"), "manifest must name gently: {body}");
            assert!(r.stderr.is_empty(), "init keeps stderr clean on success");

            // -v: verbose diagnostics appear on stderr; stdout unchanged.
            let r = run_gently(&["-v"], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(0), r.code, "-v must not fail");
            assert_eq!(ORACLE, &r.stdout[..], "-v keeps the rendered bytes on stdout");
            assert!(!r.stderr.is_empty(), "-v must add a verbose diagnostic on stderr");

            // --quiet: non-error output is suppressed; exit stays 0.
            let r = run_gently(&["--quiet"], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(0), r.code, "--quiet must not fail");
            assert!(r.stdout.is_empty(), "--quiet suppresses the rendered payload");
            assert!(r.stderr.is_empty(), "--quiet suppresses verbose diagnostics");
        }

        /// cli.p6 (c6): `--json` wraps results in the genesis Envelope
        /// (ok, envelope_version, envelope_kind, data, warnings, hints,
        /// meta); failures carry a typed error envelope; human mode stays
        /// byte-identical to the unwrapped oracle stream.
        #[test]
        fn p6() {
            // success envelope: ok:true, kind "ok", rendered bytes in data.
            let r = run_gently(&["--json"], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(0), r.code, "--json success exits 0");
            let out = String::from_utf8_lossy(&r.stdout);
            for field in ["\"ok\":true", "\"envelope_version\":", "\"envelope_kind\":\"ok\"", "\"warnings\":", "\"hints\":", "\"meta\":"] {
                assert!(out.contains(field), "envelope must carry {field}: {out}");
            }
            assert!(out.contains("| a | --> | b |"), "envelope data must carry the rendered bytes: {out}");
            assert!(r.stderr.is_empty(), "json success keeps stderr clean: {}", String::from_utf8_lossy(&r.stderr));

            // failure envelope: typed error kind, ok:false, nonzero exit.
            let r = run_gently(&["--json"], b"this is not graph text\n");
            assert_eq!(Some(1), r.code, "--json failure exits 1");
            let out = String::from_utf8_lossy(&r.stdout);
            assert!(out.contains("\"ok\":false"), "failure envelope ok:false: {out}");
            assert!(out.contains("\"envelope_kind\":\"error\""), "failure envelope kind error: {out}");
            assert!(out.contains("parse error"), "failure envelope names the failure: {out}");

            // human mode: raw oracle bytes, not a wrapped envelope.
            let r = run_gently(&["--human"], b"[ a ] --> [ b ]\n");
            assert_eq!(ORACLE, &r.stdout[..], "human mode must stay byte-identical");
        }

        /// cli.p7 (c7): unknown subcommands and flags get a genesis
        /// suggestion (DidYouMean / Fix) naming the closest known name on
        /// stderr, before the nonzero exit.
        #[test]
        fn p7() {
            // subcommand near-miss: DidYouMean names the closest known command.
            let dir = TempDir::new("p7-sub");
            let r = run_gently_in(&dir.0, &["initt"], b"");
            assert!(r.code.map_or(true, |c| c != 0), "unknown subcommand must exit nonzero");
            let err = String::from_utf8_lossy(&r.stderr);
            assert!(err.contains("Did you mean"), "suggestion expected: {err}");
            assert!(err.contains("init"), "suggestion must name the closest known command: {err}");

            // flag near-miss: DidYouMean names the closest known flag.
            let r = run_gently(&["--outpt"], b"");
            assert!(r.code.map_or(true, |c| c != 0), "unknown flag must exit nonzero");
            let err = String::from_utf8_lossy(&r.stderr);
            assert!(err.contains("Did you mean"), "flag suggestion expected: {err}");
            assert!(err.contains("output"), "flag suggestion must name the closest known flag: {err}");

            // far miss: a Fix suggestion still precedes the nonzero exit.
            let r = run_gently(&["--zzzz"], b"");
            assert!(r.code.map_or(true, |c| c != 0), "unknown flag must exit nonzero");
            let err = String::from_utf8_lossy(&r.stderr);
            assert!(!err.is_empty(), "a Fix suggestion must be printed: {err}");
            assert!(err.contains("--zzzz"), "Fix must name the unknown flag: {err}");
        }

        /// cli.p8 (c8): `gently doctor` runs suite health checks through
        /// genesis DoctorRunner — oracle availability, output-format
        /// support, fixture pin freshness — names each check's status, and
        /// applies available auto-fixes (--fix registers the genesis
        /// entry). Runs in a tempdir; never the repo.
        #[test]
        fn p8() {
            // Without the oracle toolchain (PATH stripped): the oracle check
            // is named and failing; without --fix nothing is auto-repaired.
            let dir = TempDir::new("p8-doctor");
            let out = Command::new(env!("CARGO_BIN_EXE_gently"))
                .args(["doctor"])
                .current_dir(&dir.0)
                .env("PATH", "/nonexistent-gently-p8")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("spawn gently doctor");
            assert_eq!(Some(1), out.status.code(), "unhealthy doctor exits 1");
            let report = String::from_utf8_lossy(&out.stdout);
            for check in ["oracle", "format", "pin", "genesis"] {
                assert!(report.contains(check), "report must name the {check} check: {report}");
            }
            assert!(report.contains("fail"), "oracle check must be named failing: {report}");
            assert!(!dir.0.join(".genesis/tools.toml").exists(), "no auto-fix without --fix");

            // --fix: the available auto-fix (genesis registration) applies.
            let r = run_gently_in(&dir.0, &["doctor", "--fix"], b"");
            let report = String::from_utf8_lossy(&r.stdout);
            assert!(dir.0.join(".genesis/tools.toml").exists(), "--fix must register the genesis entry: {report}");
            assert!(report.contains("pass"), "fixed registration must pass: {report}");
        }
    }
}