use super::*;

/// c5's typed gate over one committed dump (c4's companion contract): the
/// oracle pin header, the explicit hash-seed field, and the per-stage
/// observability marks must all be present.
pub(crate) fn check_dump(name: &str, text: &str) -> Result<(), String> {
    let header = text.lines().next().unwrap_or("");
    let pin_ok = header.starts_with("# dump: Graph::Easy v0.69 @ ") && header.contains(PIN_COMMIT);
    if !pin_ok {
        return Err(dump_defect(&format!(
            "{name}: missing the oracle pin header ({PIN_COMMIT})"
        )));
    }
    if !text
        .lines()
        .any(|l| l.starts_with("-- seed ") && l["-- seed ".len()..].trim().parse::<i64>().is_ok())
    {
        return Err(dump_defect(&format!(
            "{name}: missing the explicit hash-seed field"
        )));
    }
    if !text.lines().any(|l| l.starts_with("== invocation")) {
        return Err(dump_defect(&format!(
            "{name}: missing the per-stage observability marks"
        )));
    }
    Ok(())
}

/// Split one dump into its `-- seed N` blocks (comment lines excluded) —
/// the raw material for the tier-2 stability cross-check.
pub(crate) fn seed_blocks(text: &str) -> Vec<Vec<String>> {
    // Split at seed headers (anchored to line starts so body text that
    // merely mentions seeds never splits); the first chunk is the
    // preamble before the first seed header.
    text.split("\n-- seed ")
        .skip(1)
        .map(|chunk| {
            chunk
                .lines()
                .skip(1) // the remainder of the `-- seed N` header line
                .filter(|l| !l.starts_with('#'))
                .map(str::to_string)
                .collect()
        })
        .collect()
}
