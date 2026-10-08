//! Integration tests for the `gently` binary — the end-to-end tracer
//! pipeline (epic gently-2po, tb.cli / gently-2po.9).
//!
//! Binding constraints: `specs/cli.md` c3 (rendered output to stdout,
//! diagnostics to stderr, exit 0 on success / nonzero on parse or render
//! errors) and c4 (unknown format → diagnostic naming the requested and
//! valid formats, exit 2). The ascii oracle is pinned from Graph::Easy
//! v0.69 @ ededa3d7 (`add_edge("a","b"); as_ascii`).
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

/// cli.c3 + the pinned oracle: tracer text over stdin renders byte-identical
/// ascii to stdout, exit 0, empty stderr. A file argument (cli.c1) reaches
/// the same bytes.
#[test]
fn tracer_end_to_end_ascii_matches_oracle() {
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
    let path = std::env::temp_dir().join(format!("gently-tracer-{}.txt", std::process::id()));
    std::fs::write(&path, b"[ a ] --> [ b ]\n").expect("write fixture file");
    let r = run_gently(&[path.to_str().unwrap()], b"");
    assert_eq!(Some(0), r.code, "exit 0 on success via file argument");
    assert_eq!(ORACLE, &r.stdout[..], "file input must render the same bytes");
    assert!(r.stderr.is_empty(), "stderr must be empty via file argument");
    let _ = std::fs::remove_file(&path);
}

/// cli.c4: an unknown output format exits 2 with a diagnostic on stderr
/// naming both the requested format and the valid formats; stdout stays
/// empty (cli.c3: diagnostics go to stderr).
#[test]
fn unknown_format_exits_2_naming_formats() {
    let r = run_gently(&["--format", "html"], b"[ a ] --> [ b ]\n");
    assert_eq!(Some(2), r.code, "unknown format must exit 2");
    assert!(r.stdout.is_empty(), "diagnostics must not touch stdout");
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(err.contains("html"), "diagnostic must name the requested format: {err}");
    assert!(err.contains("ascii"), "diagnostic must name the valid formats: {err}");
}

/// cli.c3: unparseable input fails with a nonzero exit and a diagnostic on
/// stderr; stdout stays empty.
#[test]
fn unparseable_input_fails_nonzero_with_diagnostic() {
    let r = run_gently(&[], b"this is not graph text\n");
    assert_ne!(Some(0), r.code, "parse error must exit nonzero");
    assert_eq!(Some(1), r.code, "user-facing parse errors exit 1");
    assert!(r.stdout.is_empty(), "diagnostics must not touch stdout");
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(!err.is_empty(), "a diagnostic must be printed to stderr");
    assert!(
        err.contains("parse error"),
        "diagnostic should name the failure kind: {err}"
    );
}