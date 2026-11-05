//! Oracle-byte extraction helpers for the decision.trace EXCL-001
//! corpus check: pinned `as_ascii` bytes out of claims/<name>.observed
//! (or the fixture's `.ascii.expected`), probe bodies out of probes.pl,
//! and the traced gently run.

use super::*;

/// Run gently over `source` with the given trace stage gated on; returns
/// (rendered ascii bytes, trace text). A parse or render failure renders
/// empty bytes (the divergence is real — the oracle renders).
pub(crate) fn run_traced(tag: &str, source: &str, stage: &str) -> (String, String) {
    let file = std::env::temp_dir().join(format!("gently-dt-p3-{tag}.trace"));
    std::fs::remove_file(&file).ok();
    // Thread-local gate override, never the process-global env: 145
    // sibling tests run layout concurrently and the env is shared state
    // (the windows-latest CI flake). Each test thread sees only its own
    // gate; the env path stays covered by the env_smoke test below.
    gently_core::trace::set_test_gate(Some(stage), Some(&file));
    let traced = traced_run(source, &file);
    gently_core::trace::set_test_gate(None, None);
    std::fs::remove_file(&file).ok();
    traced
}

fn traced_run(source: &str, trace_path: &std::path::Path) -> (String, String) {
    let graph = match gently_core::parse::text::parse(source) {
        Ok(g) => g,
        Err(_) => return (String::new(), String::new()),
    };
    let laid = gently_core::layout::layout(&graph);
    let bytes = gently_core::render::ascii::render(&graph, &laid).unwrap_or_default();
    let trace = std::fs::read_to_string(trace_path).unwrap_or_default();
    (bytes, trace)
}

/// Input: the fixture file when one exists, else the probe body mined
/// out of probes.pl (either probe form).
pub(crate) fn fixture_input(root: &std::path::Path, name: &str) -> String {
    match fs::read_to_string(
        root.join("tests/fixtures/graph-easy")
            .join(format!("{name}.txt")),
    ) {
        Ok(text) => text,
        Err(_) => probe_body(&root.join("tests/repro/probes.pl"), name)
            .unwrap_or_else(|| panic!("{name}: no fixture and no probe body")),
    }
}

/// Pinned oracle bytes: claims/<name>.observed seed 0 as_ascii, or the
/// fixture's .ascii.expected (with its pinned `# oracle:` header lines
/// stripped, per the E1 driver's convention). None = not comparable.
pub(crate) fn oracle_bytes_for(root: &std::path::Path, name: &str) -> Option<String> {
    let claims = root
        .join("tests/repro/claims")
        .join(format!("{name}.observed"));
    claims_ascii(&claims).or_else(|| fixture_ascii_expected(root, name))
}

fn fixture_ascii_expected(root: &std::path::Path, name: &str) -> Option<String> {
    fs::read_to_string(
        root.join("tests/fixtures/graph-easy")
            .join(format!("{name}.ascii.expected")),
    )
    .ok()
    .map(|t| {
        t.lines()
            .filter(|l| !l.starts_with("# oracle:"))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// The trace-diff contract for one fixture class (EXCL-001):
///   - byte-divergent class → nonempty minimal cut on the dump's stage;
///   - byte-identical class → empty cut, or the typed DecisionOnly
///     refusal (surfaced informationally via --informational).
pub(crate) fn assert_trace_contract(
    name: &str,
    oracle_bytes: &str,
    gently_bytes: &str,
    aligned: &str,
    gently_trace: &str,
    divergent: bool,
) {
    match gently_core::trace_diff::trace_diff(
        oracle_bytes,
        gently_bytes,
        aligned,
        gently_trace,
        false,
    ) {
        Ok(cut) if cut.is_empty() => {
            assert!(!divergent, "{name}: byte-identical fixture fully agrees");
        }
        Ok(cut) => assert_divergent_cut(name, divergent, &cut),
        Err(gently_core::trace_diff::TraceDiffError::DecisionOnly { cut }) => {
            assert_decision_only(name, divergent, &cut)
        }
    }
}

fn assert_divergent_cut(name: &str, divergent: bool, cut: &[gently_core::trace_diff::CutEvent]) {
    assert!(
        divergent,
        "{name}: trace-diff must gate byte-identical fixtures"
    );
    assert!(
        !cut.is_empty(),
        "{name}: EXCL-001 — byte-divergent fixture must yield a nonempty trace diff"
    );
    assert!(
        cut.iter().all(|e| e.stage == "layout.place"),
        "{name}: cut must live on the dump's observed stage: {cut:?}"
    );
    eprintln!(
        "excl001 {name}: byte-divergent, cut = {}",
        cut.iter()
            .map(|e| format!("{}[{}]", e.stage, e.key))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn assert_decision_only(name: &str, divergent: bool, cut: &[gently_core::trace_diff::CutEvent]) {
    assert!(!cut.is_empty());
    eprintln!(
        "excl001 {name}: byte-identical, decision-only cut = {}",
        cut.iter()
            .map(|e| format!("{}[{}]", e.stage, e.key))
            .collect::<Vec<_>>()
            .join(", ")
    );
    assert!(
        !divergent,
        "{name}: byte gate fired on byte-DIVERGENT fixture — gate bug"
    );
}

/// Extract a probe body from probes.pl. Two committed forms:
///   `..._probe_body('<name>', <<'MARKER')` … MARKER
///   `    '<name>' => <<'MARKER',` … MARKER
pub(crate) fn probe_body(probes: &Path, name: &str) -> Option<String> {
    let text = fs::read_to_string(probes).ok()?;
    let opener = probe_opener(name);
    let marker = text.lines().find_map(|line| marker_after(line, &opener))?;
    let body = collect_probe_body(&text, &opener, marker);
    let mut out = body.join("\n");
    out.push('\n');
    Some(out)
}

fn probe_opener(name: &str) -> String {
    format!("'{name}', <<'")
}

/// The heredoc marker token on an opener line, for either committed
/// probe form.
fn marker_after<'a>(line: &'a str, opener: &str) -> Option<&'a str> {
    let i = line
        .find(opener)
        .or_else(|| line.find(&hash_opener_of(opener)))?;
    let rest = &line[i + marker_opener_len(line, opener)..];
    rest.split('\'').next()
}

fn hash_opener_of(opener: &str) -> String {
    // `'<name>', <<'` → `'<name>' => <<'`
    format!("{} => <<'", opener.trim_end_matches(", <<'"))
}

fn marker_opener_len(line: &str, opener: &str) -> usize {
    if line.find(opener).is_some() {
        opener.len()
    } else {
        hash_opener_of(opener).len()
    }
}

/// Collect body lines after the opener line until the marker line.
fn collect_probe_body<'a>(text: &'a str, opener: &str, marker: &str) -> Vec<&'a str> {
    text.lines()
        .skip_while(move |l| !l.contains(opener))
        .skip(1)
        .take_while(|l| l.trim_end() != marker)
        .collect()
}

/// Extract the pinned oracle `as_ascii` block for seed 0 from a
/// claims/<name>.observed file.
pub(crate) fn claims_ascii(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let block = seed0_ascii_block(&text).unwrap_or_else(|| input_block(&text));
    trimmed_block(&block)
}

/// The `as_ascii:` block under `== seed 0 ==`, up to the next seed
/// header or the `as_txt:` terminator.
fn seed0_ascii_block(text: &str) -> Option<Vec<&str>> {
    if !text.lines().any(|l| l.trim_end() == "== seed 0 ==") {
        return None;
    }
    let body: Vec<&str> = text
        .lines()
        .skip_while(|l| l.trim_end() != "== seed 0 ==")
        .skip(1)
        .take_while(|l| !l.starts_with("== seed "))
        .skip_while(|l| l.trim_end() != "as_ascii:")
        .skip(1)
        .take_while(|l| l.trim_end() != "as_txt:")
        .collect();
    if body.is_empty() {
        None
    } else {
        Some(body)
    }
}

/// Fallback bytes: everything after the first `== input:` header up to
/// the next one.
fn input_block(text: &str) -> Vec<&str> {
    text.lines()
        .skip_while(|l| !l.starts_with("== input:"))
        .skip(1)
        .take_while(|l| !l.starts_with("== input:"))
        .collect()
}

/// Trim blank edge lines; None when nothing remains.
fn trimmed_block(block: &[&str]) -> Option<String> {
    let first = block
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(block.len());
    let last = block
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map_or(0, |i| i + 1);
    if first >= last {
        return None;
    }
    Some(block[first..last].join("\n"))
}
