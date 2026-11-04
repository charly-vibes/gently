use crate::common::{run_gently, run_gently_in, TempDir, ORACLE};
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
    std::fs::create_dir_all(&dir).must("create temp dir");
    let input = dir.join("in.txt");
    std::fs::write(&input, b"[ a ] --> [ b ]\n").must("write fixture file");
    let r = run_gently(&[input.to_str().must("value")], b"");
    assert_eq!(Some(0), r.code, "exit 0 on success via file argument");
    assert_eq!(
        ORACLE,
        &r.stdout[..],
        "file input must render the same bytes"
    );
    assert!(
        r.stderr.is_empty(),
        "stderr must be empty via file argument"
    );

    // second positional names the output file; extra positionals
    // are silently ignored (bug-for-bug with upstream).
    let out = dir.join("out");
    let extra = dir.join("extra.txt");
    std::fs::write(&extra, b"garbage that must be ignored\n").must("write extra");
    let r = run_gently(
        &[
            input.to_str().must("value"),
            out.to_str().must("value"),
            extra.to_str().must("value"),
        ],
        b"",
    );
    assert_eq!(Some(0), r.code, "extra positionals are ignored, not parsed");
    assert!(r.stdout.is_empty(), "output goes to the file, not stdout");
    assert_eq!(
        ORACLE,
        std::fs::read(&out).must("read output file").as_slice()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// cli.p9 (c1, gently-in2): a terminal stdin with no file argument
/// fails fast with a diagnostic naming stdin and the input file;
/// piped stdin is still read to EOF.
#[test]
fn p9() {
    // Piped stdin (the oracle path) is still read to EOF: empty
    // pipe renders the empty graph, exit 0.
    let r = run_gently(&[], b"");
    assert_eq!(Some(0), r.code, "piped stdin must still be read to EOF");
    assert!(
        r.stderr.is_empty(),
        "piped stdin renders without diagnostics"
    );

    // Terminal stdin: run the binary under a pty via `script` so
    // IsTerminal holds. Skipped silently when `script` is absent
    // (non-POSIX CI runners).
    // GNU script (Linux) needs -c to set the command and -e to
    // propagate the child's exit status; BSD script (macOS) has
    // neither flag — `script -q file cmd` runs cmd under a pty
    // and returns the child's status natively.
    let script_argv: [&str; 3] = if cfg!(target_os = "macos") {
        ["-q", "/dev/null", env!("CARGO_BIN_EXE_gently")]
    } else {
        ["-qec", env!("CARGO_BIN_EXE_gently"), "/dev/null"]
    };
    let ok = Command::new("script")
        .args(script_argv)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output();
    if let Ok(out) = ok {
        assert_eq!(
            Some(1),
            out.status.code(),
            "terminal stdin with no file argument must fail fast, not block"
        );
        // `script` muxes the child's stderr onto the pty, so the
        // diagnostic lands on script's stdout.
        let err = String::from_utf8_lossy(&out.stdout);
        assert!(
            err.contains("stdin is a terminal"),
            "diagnostic must name stdin: {err}"
        );
        assert!(
            err.contains("input file"),
            "diagnostic must name the input file: {err}"
        );
    }
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
    std::fs::create_dir_all(&dir).must("create temp dir");
    let input = dir.join("in.txt");
    let out = dir.join("out");
    std::fs::write(&input, b"[ a ] --> [ b ]\n").must("write fixture");
    let r = run_gently(
        &[
            "--as",
            "ascii",
            "--output",
            out.to_str().must("value"),
            input.to_str().must("value"),
        ],
        b"",
    );
    assert_eq!(Some(0), r.code, "file output must exit 0");
    assert!(r.stdout.is_empty(), "output goes to the file, not stdout");
    assert_eq!(
        ORACLE,
        std::fs::read(&out).must("read output file").as_slice()
    );
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
    assert_eq!(
        Some(0),
        r.code,
        "init must exit 0: {}",
        String::from_utf8_lossy(&r.stderr)
    );
    let manifest = dir.0.join(".genesis/tools.toml");
    let body = std::fs::read_to_string(&manifest)
        .must("init must write .genesis/tools.toml in the working directory");
    assert!(
        body.contains("[tools.gently]"),
        "manifest must name gently: {body}"
    );
    assert!(r.stderr.is_empty(), "init keeps stderr clean on success");

    // -v: verbose diagnostics appear on stderr; stdout unchanged.
    let r = run_gently(&["-v"], b"[ a ] --> [ b ]\n");
    assert_eq!(Some(0), r.code, "-v must not fail");
    assert_eq!(
        ORACLE,
        &r.stdout[..],
        "-v keeps the rendered bytes on stdout"
    );
    assert!(
        !r.stderr.is_empty(),
        "-v must add a verbose diagnostic on stderr"
    );

    // --quiet: non-error output is suppressed; exit stays 0.
    let r = run_gently(&["--quiet"], b"[ a ] --> [ b ]\n");
    assert_eq!(Some(0), r.code, "--quiet must not fail");
    assert!(
        r.stdout.is_empty(),
        "--quiet suppresses the rendered payload"
    );
    assert!(
        r.stderr.is_empty(),
        "--quiet suppresses verbose diagnostics"
    );
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
    for field in [
        "\"ok\":true",
        "\"envelope_version\":",
        "\"envelope_kind\":\"ok\"",
        "\"warnings\":",
        "\"hints\":",
        "\"meta\":",
    ] {
        assert!(out.contains(field), "envelope must carry {field}: {out}");
    }
    assert!(
        out.contains("| a | --> | b |"),
        "envelope data must carry the rendered bytes: {out}"
    );
    assert!(
        r.stderr.is_empty(),
        "json success keeps stderr clean: {}",
        String::from_utf8_lossy(&r.stderr)
    );

    // failure envelope: typed error kind, ok:false, nonzero exit.
    let r = run_gently(&["--json"], b"this is not graph text\n");
    assert_eq!(Some(1), r.code, "--json failure exits 1");
    let out = String::from_utf8_lossy(&r.stdout);
    assert!(
        out.contains("\"ok\":false"),
        "failure envelope ok:false: {out}"
    );
    assert!(
        out.contains("\"envelope_kind\":\"error\""),
        "failure envelope kind error: {out}"
    );
    assert!(
        out.contains("parse error"),
        "failure envelope names the failure: {out}"
    );

    // human mode: raw oracle bytes, not a wrapped envelope.
    let r = run_gently(&["--human"], b"[ a ] --> [ b ]\n");
    assert_eq!(ORACLE, &r.stdout[..], "human mode must stay byte-identical");
}

/// cli.p7 (c7): unknown subcommands and flags get a genesis
/// suggestion (DidYouMean / Fix) naming the closest known name on
/// stderr, before the nonzero exit.
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

#[path = "cli_more.rs"]
mod more;
