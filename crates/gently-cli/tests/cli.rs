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
use std::process::{Command, Stdio};

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
    child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(stdin)
        .expect("write stdin");
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
        use super::super::{run_gently, ORACLE};

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
    }
}