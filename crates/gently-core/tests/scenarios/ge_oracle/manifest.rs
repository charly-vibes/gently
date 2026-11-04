use super::*;

/// Manifest shape: `# admission: <pin>` header, then `name<TAB>verdict<TAB>seconds` rows.
pub(crate) fn parse_manifest(manifest: &str) -> Vec<Row> {
    let mut lines = manifest.lines();
    let header = lines
        .next()
        .must("admission manifest must have a pin header");
    assert_header_names_pinned_oracle(header);
    lines.filter(|l| !l.is_empty()).map(parse_row).collect()
}

pub(crate) fn assert_header_names_pinned_oracle(header: &str) {
    let expected = format!("# admission: Graph::Easy v{PIN_VERSION} @ ");
    assert!(
        header.starts_with(&expected) && header.contains(PIN_COMMIT),
        "admission manifest header must name the pinned oracle, got: {header}"
    );
}

pub(crate) fn parse_row(line: &str) -> Row {
    let cols: Vec<&str> = line.split('\t').collect();
    assert_eq!(
        cols.len(),
        3,
        "admission row must have 3 TSV columns: {line:?}"
    );
    assert_valid_verdict(cols[1]);
    let seconds = parse_seconds(line, cols[2]);
    (cols[0].to_string(), cols[1].to_string(), seconds)
}

/// The verdict set is closed: stable, hash-dependent, or render-error.
pub(crate) fn assert_valid_verdict(verdict: &str) {
    assert!(
        matches!(verdict, "stable" | "hash-dependent" | "render-error"),
        "admission verdict must be a closed set, got: {verdict:?}"
    );
}

pub(crate) fn parse_seconds(line: &str, raw: &str) -> f64 {
    let seconds: f64 = raw
        .parse()
        .unwrap_or_else(|e| panic!("admission seconds must be a number, got {e}: {line:?}"));
    assert!(
        seconds.is_finite() && seconds >= 0.0,
        "admission seconds must be finite and non-negative: {line:?}"
    );
    seconds
}

/// Every corpus input is classified exactly once (p7).
pub(crate) fn assert_every_input_classified(rows: &[Row], inputs: &[String]) {
    for name in inputs {
        let n = rows.iter().filter(|(r, _, _)| r == name).count();
        assert_eq!(
            n, 1,
            "fixture {name} must appear exactly once in the manifest"
        );
    }
}

pub(crate) fn assert_no_orphan_rows(rows: &[Row], inputs: &[String]) {
    for (name, _, _) in rows {
        assert!(
            inputs.iter().any(|i| i == name),
            "manifest row {name} is not a corpus input"
        );
    }
}

pub(crate) fn derive_tier1(rows: &[Row]) -> Vec<&String> {
    rows.iter()
        .filter(|(_, v, s)| *v == "stable" && *s <= ENVELOPE_SECONDS)
        .map(|(n, _, _)| n)
        .collect()
}
