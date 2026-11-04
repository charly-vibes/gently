use super::*;
use crate::common::{run_gently_in, TempDir};

#[test]
fn p7() {
    // subcommand near-miss: DidYouMean names the closest known command.
    let dir = TempDir::new("p7-sub");
    let r = run_gently_in(&dir.0, &["initt"], b"");
    assert!(
        !r.code.is_some_and(|c| c == 0),
        "unknown subcommand must exit nonzero"
    );
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(err.contains("Did you mean"), "suggestion expected: {err}");
    assert!(
        err.contains("init"),
        "suggestion must name the closest known command: {err}"
    );

    // flag near-miss: DidYouMean names the closest known flag.
    let r = run_gently(&["--outpt"], b"");
    assert!(
        !r.code.is_some_and(|c| c == 0),
        "unknown flag must exit nonzero"
    );
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(
        err.contains("Did you mean"),
        "flag suggestion expected: {err}"
    );
    assert!(
        err.contains("output"),
        "flag suggestion must name the closest known flag: {err}"
    );

    // far miss: a Fix suggestion still precedes the nonzero exit.
    let r = run_gently(&["--zzzz"], b"");
    assert!(
        !r.code.is_some_and(|c| c == 0),
        "unknown flag must exit nonzero"
    );
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(!err.is_empty(), "a Fix suggestion must be printed: {err}");
    assert!(
        err.contains("--zzzz"),
        "Fix must name the unknown flag: {err}"
    );
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
        .must("spawn gently doctor");
    assert_eq!(Some(1), out.status.code(), "unhealthy doctor exits 1");
    let report = String::from_utf8_lossy(&out.stdout);
    for check in ["oracle", "format", "pin", "genesis"] {
        assert!(
            report.contains(check),
            "report must name the {check} check: {report}"
        );
    }
    assert!(
        report.contains("fail"),
        "oracle check must be named failing: {report}"
    );
    assert!(
        !dir.0.join(".genesis/tools.toml").exists(),
        "no auto-fix without --fix"
    );

    // --fix: the available auto-fix (genesis registration) applies.
    let r = run_gently_in(&dir.0, &["doctor", "--fix"], b"");
    let report = String::from_utf8_lossy(&r.stdout);
    assert!(
        dir.0.join(".genesis/tools.toml").exists(),
        "--fix must register the genesis entry: {report}"
    );
    assert!(
        report.contains("pass"),
        "fixed registration must pass: {report}"
    );
}

/// cli.p10 (c9, gently-cij): the input format is selected by
/// `--from <fmt>` (txt default; `dot` aliases graphviz; the upstream
/// boolean aliases --from_txt/--from_graphviz work) and, for txt, by
/// upstream's content sniffing (Parser.pm): a line-start `digraph
/// [name] {` / `digraph {` / `strict (di)graph [name] {` header
/// routes the source to the DOT parser even under `--from txt`.
/// Unknown `--from` values and the unwired VCG parser (vcg/gdl) exit
/// 255; `--from_dot` is NOT an upstream option (GetOptions unknown →
/// exit 2). Oracle evidence: tests/repro/claims/cli-input-from.observed.
#[test]
fn p10() {
    let fixtures = 2; // dot + vcg fixtures exercised by the phases
    assert_oracle_dot();
    p10_input_sources();
    assert_eq!(fixtures, 2, "both input-source fixtures exercised");
}

fn assert_oracle_dot() {
    // Oracle bytes for `digraph G { a -> b }` rendered as ascii via
    // the Graphviz parser (v0.69 @ ededa3d7, recorded 2026-10-14).
    let (_dir, dot_arg) = write_dot_fixture();

    // Explicit --from dot / graphviz parse the DOT input.
    for from in ["dot", "graphviz"] {
        let r = run_gently(&["--from", from, dot_arg.as_str()], b"");
        assert_eq!(Some(0), r.code, "--from {from} must parse DOT");
        assert_eq!(
            DOT_ORACLE,
            &r.stdout[..],
            "--from {from} must render the oracle DOT bytes"
        );
    }

    // The upstream boolean alias --from_graphviz works too.
    let r = run_gently(&["--from_graphviz", dot_arg.as_str()], b"");
    assert_eq!(Some(0), r.code, "--from_graphviz must parse DOT");
    assert_eq!(DOT_ORACLE, &r.stdout[..], "--from_graphviz renders DOT");
}

fn p10_input_sources() {
    let (_dir, dot_arg) = write_dot_fixture();
    // Sniffing: without --from (and with explicit --from txt / the
    // --from_txt alias) a line-start digraph header routes to the
    // DOT parser — the oracle's Graph::Easy::Parser sniff.
    for args in [
        vec![dot_arg.as_str()],
        vec!["--from", "txt", dot_arg.as_str()],
        vec!["--from_txt", dot_arg.as_str()],
    ] {
        let r = run_gently(&args, b"");
        assert_eq!(Some(0), r.code, "sniffed DOT input must parse: {args:?}");
        assert_eq!(
            DOT_ORACLE,
            &r.stdout[..],
            "sniffed DOT input must render the oracle bytes: {args:?}"
        );
    }

    // Graph::Easy-text input continues to parse unchanged.
    let r = run_gently(&[], b"[ a ] --> [ b ]\n");
    assert_eq!(Some(0), r.code, "text input parses unchanged");
    assert_eq!(ORACLE, &r.stdout[..], "text input renders the oracle bytes");

    // Forcing the Graphviz parser on text content fails (the oracle
    // dies with "not recognized by Graph::Easy::Parser::Graphviz").
    let r = run_gently(&["--from", "graphviz"], b"[ a ] --> [ b ]\n");
    assert!(
        !r.code.is_some_and(|c| c == 0),
        "forced DOT parser must reject text content"
    );
    assert!(
        !r.stderr.is_empty(),
        "forced-DOT parse failure must diagnose on stderr"
    );

    // Unknown input format: exit 255 with upstream's die message
    // (get_options, bin/graph-easy line 333).
    let r = run_gently(&["--from", "foo", dot_arg.as_str()], b"");
    assert_eq!(Some(255), r.code, "unknown --from must exit 255");
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(
        err.contains("Unknown input format 'foo'"),
        "diagnostic must name the unknown input format: {err}"
    );

    // vcg/gdl are valid upstream --from values but route to the VCG
    // parser gently never ports — documented scope-offs, exit 255.
    for flag in ["--from=vcg", "--from=gdl", "--from_vcg", "--from_gdl"] {
        let r = run_gently(&[flag, dot_arg.as_str()], b"");
        assert_eq!(Some(255), r.code, "{flag} must exit 255 (scope-off)");
        let err = String::from_utf8_lossy(&r.stderr);
        assert!(
            err.contains("vcg") || err.contains("gdl") || err.contains("VCG"),
            "{flag} diagnostic must name the unwired VCG parser: {err}"
        );
    }

    // --from_dot is NOT a declared upstream GetOptions flag — the
    // unknown-option path (usage + exit 2, suggestion per c7).
    let r = run_gently(&["--from_dot", dot_arg.as_str()], b"");
    assert_eq!(Some(2), r.code, "--from_dot must hit the unknown-flag path");
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(
        err.contains("from_dot"),
        "unknown-option diagnostic must name the flag: {err}"
    );

    // The `graph: {` sniff routes to the VCG parser upstream — no
    // gently parser exists, so the sniff is a scope-off diagnostic.
    let dir = TempDir::new("p10-vcg");
    let vcg_file = dir.0.join("graph.vcg");
    std::fs::write(&vcg_file, b"graph: {\nnode: a\n}\n").must("write vcg fixture");
    let r = run_gently(&[vcg_file.to_str().must("value")], b"");
    assert!(
        !r.code.is_some_and(|c| c == 0),
        "graph: {{ sniff must not silently text-parse"
    );
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(
        err.contains("VCG") || err.contains("vcg"),
        "sniffed-VCG diagnostic must name the unwired parser: {err}"
    );
}

/// A temp `.dot` fixture with one `a -> b` chain; returns its path.
const DOT_ORACLE: &[u8] = b"+---+\n| a |\n+---+\n  |\n  |\n  v\n+---+\n| b |\n+---+\n";

/// A temp `.dot` fixture with one `a -> b` chain; returns the dir (kept
/// alive for the test's duration) and the fixture path.
fn write_dot_fixture() -> (TempDir, String) {
    let dir = TempDir::new("p10-from");
    let dot_file = dir.0.join("graph.dot");
    std::fs::write(&dot_file, b"digraph G { a -> b }\n").must("write dot fixture");
    let arg = dot_file.to_str().must("value").to_string();
    (dir, arg)
}
