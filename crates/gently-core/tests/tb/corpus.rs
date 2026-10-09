//! tb.corpus (gently-0of): recorded-oracle corpus scenario tests.

/// tb.corpus (gently-0of): the full recorded fixture corpus — for every
/// `tests/fixtures/graph-easy/*.txt` input, gently's parse -> layout ->
/// ascii render pipeline must reproduce the recorded oracle companion
/// (`*.ascii.expected`, Graph::Easy v0.69 @ ededa3d7, pin header
/// stripped) byte-identically (ge.oracle.c3). The txt companions are
/// covered by the tb::oracle differential tests and ge_txt_render.
use gently_core::{layout, parse::text, render::ascii};
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/graph-easy")
}

/// Read a fixture input by base name (e.g. `"chain"`).
fn fixture_input(base: &str) -> String {
    let bytes = std::fs::read(fixture_dir().join(format!("{base}.txt")))
        .unwrap_or_else(|e| panic!("fixture input {base}.txt: {e}"));
    std::str::from_utf8(&bytes)
        .unwrap_or_else(|e| panic!("fixture {base}.txt must be utf-8: {e}"))
        .to_string()
}

/// Read a recorded oracle companion and strip every leading
/// `# oracle: ` pin-header line, leaving the payload bytes.
fn oracle_body(base: &str, format: &str) -> Vec<u8> {
    let bytes = std::fs::read(fixture_dir().join(format!("{base}.{format}.expected")))
        .unwrap_or_else(|e| panic!("recorded companion {base}.{format}.expected: {e}"));
    let s = std::str::from_utf8(&bytes)
        .unwrap_or_else(|e| panic!("companion {base}.{format}.expected must be utf-8: {e}"));
    let mut rest = s;
    while let Some(idx) = rest.find('\n') {
        if rest[..idx].starts_with("# oracle: ") {
            rest = &rest[idx + 1..];
        } else {
            break;
        }
    }
    rest.as_bytes().to_vec()
}

/// Parse -> layout -> ascii render through gently's pipeline (the
/// same path the `gently` binary drives).
fn rendered_ascii(base: &str) -> String {
    let g = text::parse(&fixture_input(base)).unwrap_or_else(|e| panic!("{base} must parse: {e}"));
    let l = layout::layout(&g);
    ascii::render(&g, &l).unwrap_or_else(|e| panic!("{base} must render: {e}"))
}

/// ge.oracle.c3: chain (a->b->c) renders as a three-box row.
#[test]
fn chain_renders_oracle_ascii() {
    assert_eq!(rendered_ascii("chain").into_bytes(), oracle_body("chain", "ascii"));
}

/// ge.oracle.c3: mixed_isolated (chain + isolated d) — chain row,
/// isolated node as its own left-aligned box below.
#[test]
fn mixed_isolated_renders_oracle_ascii() {
    assert_eq!(
        rendered_ascii("mixed_isolated").into_bytes(),
        oracle_body("mixed_isolated", "ascii")
    );
}

/// ge.oracle.c3: parallel (a->b twice) — second edge bends around
/// above the boxes.
#[test]
fn parallel_renders_oracle_ascii() {
    assert_eq!(
        rendered_ascii("parallel").into_bytes(),
        oracle_body("parallel", "ascii")
    );
}

/// ge.oracle.c3: selfloop (a->a) — loop elbow above the box with a
/// `v` arrowhead.
#[test]
fn selfloop_renders_oracle_ascii() {
    assert_eq!(
        rendered_ascii("selfloop").into_bytes(),
        oracle_body("selfloop", "ascii")
    );
}

/// ge.oracle.c3: txt-diamond (a->b, a->c, b->d, c->d) — ranks 0/1/2,
/// c below rank 1, vertical segments + arrowheads as recorded.
#[test]
fn txt_diamond_renders_oracle_ascii() {
    assert_eq!(
        rendered_ascii("txt-diamond").into_bytes(),
        oracle_body("txt-diamond", "ascii")
    );
}

/// ge.oracle.c3: txt-isolated — already-passing shape stays
/// byte-identical.
#[test]
fn txt_isolated_renders_oracle_ascii() {
    assert_eq!(
        rendered_ascii("txt-isolated").into_bytes(),
        oracle_body("txt-isolated", "ascii")
    );
}

/// ge.oracle.c3: txt-shared-target — b east of c, edge drawn `<--`.
#[test]
fn txt_shared_target_renders_oracle_ascii() {
    assert_eq!(
        rendered_ascii("txt-shared-target").into_bytes(),
        oracle_body("txt-shared-target", "ascii")
    );
}

/// ge.oracle.c3: tracer — already-passing shape stays
/// byte-identical.
#[test]
fn tracer_renders_oracle_ascii() {
    assert_eq!(
        rendered_ascii("tracer").into_bytes(),
        oracle_body("tracer", "ascii")
    );
}
