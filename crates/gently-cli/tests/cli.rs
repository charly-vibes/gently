//! Integration tests for the `gently` binary — the end-to-end tracer
//! pipeline (epic gently-2po, tb.cli / gently-2po.9).
//!
//! Binding constraints: `specs/cli.md` c3 (rendered output to stdout or the
//! `--output` file, diagnostics to stderr, exit 0 on success / nonzero on
//! parse or render errors) and c4 (unknown format → diagnostic naming the
//! requested and valid formats, exit 255 — bug-for-bug with the upstream
//! script's uncaught `die`, re-derived under the ge.oracle.c7 scope
//! contract, gently-ghh). Flag roles follow upstream bin/graph-easy v0.69:
//! `--as <fmt>` selects the format, `--output <file>` names the output
//! file, positionals are `[inputfile [outputfile]]`, extras are ignored
//! (cli.c1/c2, gently-0h9). The ascii oracle is pinned from Graph::Easy
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

/// cli.c4: an unknown output format exits 255 with a diagnostic on stderr
/// naming both the requested format and the valid formats; stdout stays
/// empty (cli.c3: diagnostics go to stderr). Exit 255 is bug-for-bug with
/// the upstream script, whose unknown `--as` dies calling the missing
/// `as_<fmt>` method (re-derived against upstream v0.69, gently-0h9).
#[test]
fn unknown_format_exits_255_naming_formats() {
    let r = run_gently(&["--as", "html"], b"[ a ] --> [ b ]\n");
    assert_eq!(Some(255), r.code, "unknown format must exit 255");
    assert!(r.stdout.is_empty(), "diagnostics must not touch stdout");
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(err.contains("html"), "diagnostic must name the requested format: {err}");
    assert!(err.contains("ascii"), "diagnostic must name the valid formats: {err}");
}

/// cli.c2: `--as` selects the renderer; `--output` names the output file
/// and receives the rendered bytes (upstream bug-for-bug, gently-0h9).
#[test]
fn output_flag_writes_rendered_bytes_to_file() {
    let dir = std::env::temp_dir().join(format!("gently-out-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let input = dir.join("in.txt");
    let out = dir.join("out");
    std::fs::write(&input, b"[ a ] --> [ b ]\n").expect("write fixture");
    let r = run_gently(
        &["--as", "ascii", "--output", out.to_str().unwrap(), input.to_str().unwrap()],
        b"",
    );
    assert_eq!(Some(0), r.code, "file output must exit 0");
    assert!(r.stdout.is_empty(), "output goes to the file, not stdout");
    assert_eq!(ORACLE, std::fs::read(&out).expect("read output file").as_slice());
    let _ = std::fs::remove_dir_all(&dir);
}

/// cli.c1/c2: a second positional argument names the output file (upstream
/// `graph-easy [options] [inputfile [outputfile]]`), and format defaults to
/// ascii when neither `--as` nor a mapped extension applies (gently-0h9).
#[test]
fn second_positional_is_output_file() {
    let dir = std::env::temp_dir().join(format!("gently-pos-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let input = dir.join("in.txt");
    let out = dir.join("out");
    std::fs::write(&input, b"[ a ] --> [ b ]\n").expect("write fixture");
    let r = run_gently(&[input.to_str().unwrap(), out.to_str().unwrap()], b"");
    assert_eq!(Some(0), r.code, "positional output must exit 0");
    assert!(r.stdout.is_empty(), "output goes to the file, not stdout");
    assert_eq!(ORACLE, std::fs::read(&out).expect("read output file").as_slice());

    // cli.c1 bug-for-bug: positional arguments beyond input+output are
    // silently ignored by the upstream script.
    let extra = dir.join("extra.txt");
    std::fs::write(&extra, b"garbage that must be ignored\n").expect("write extra");
    let out2 = dir.join("out2");
    let r = run_gently(
        &[input.to_str().unwrap(), out2.to_str().unwrap(), extra.to_str().unwrap()],
        b"",
    );
    assert_eq!(Some(0), r.code, "extra positionals are ignored, not parsed");
    assert_eq!(ORACLE, std::fs::read(&out2).expect("read output file").as_slice());
    let _ = std::fs::remove_dir_all(&dir);
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