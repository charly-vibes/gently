//! Shared helpers for the `gently` binary integration tests — the
//! process-spawn plumbing (cli.c3/c4 Run, TempDir, stdin writer) and the
//! pinned oracle bytes. Extracted from tests/cli.rs to hold the
//! pretender file limit (gently-4ig).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Like [`run_gently`], but spawns `gently` with `dir` as its working
/// directory — required for filesystem-touching surfaces (`init`, `doctor`)
/// so they never write into the repo working tree (gently-ef5).
pub fn run_gently_in(dir: &Path, args: &[&str], stdin: &[u8]) -> Run {
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
pub struct TempDir(pub std::path::PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> TempDir {
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
pub const ORACLE: &[u8] = b"+---+     +---+\n| a | --> | b |\n+---+     +---+\n";

/// One `gently` invocation's observable process behavior (cli.c3/c4).
pub struct Run {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub code: Option<i32>,
}

pub fn run_gently(args: &[&str], stdin: &[u8]) -> Run {
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
/// The library renderer bytes for the corpus fixture, per wired format —
/// the expected CLI stream for `--as` (gently-4ig). The renderers
/// themselves are byte-pinned by their `ge.*` scenarios against the
/// probed oracle with documented scope-offs, so the CLI pin asserts
/// routing: CLI bytes == renderer bytes.
pub fn fixture_bytes(fmt: &str) -> String {
    let g = gently_core::parse::text::parse("[ a ] --> [ b ]\n").expect("fixture parses");
    match fmt {
        "graphviz" => gently_core::render::graphviz::render(&g),
        "html" => gently_core::render::html::document(&g, &gently_core::layout::layout(&g))
            .expect("html renders"),
        "txt" => gently_core::render::txt::render(&g),
        _ => unreachable!("unwired format {fmt}"),
    }
}
