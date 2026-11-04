//! Purpose: value-unquoting and the rankdir mapping for the ge.dot_parser
//! grammar (gently-89p, gently-a0k).
//! Responsibilities: unescape a raw quoted-string content exactly once —
//! `\"` becomes `"`, `\\` becomes `\`, any other escape keeps its
//! backslash — so the model's attribute tables receive the final verbatim
//! value with no second unquote pass (the r22 store-layer finding: the
//! model stores what it is given, the parser owns unescaping); map the
//! graph-scope `rankdir` value to `flow` per the oracle's direction map.
//! Rationale: isolated here so the once-only rule has one auditable home,
//! mirroring the `text/` module's attrs split.

/// Unquote raw quoted-string content: escapes resolved exactly once.
pub(super) fn unquote(raw: &str) -> String {
    // Sentinel character terminates the scan so a trailing backslash
    // still emits; escape state rides the scan closure (no mutable
    // bindings).
    raw.chars()
        .chain(['\0'])
        .scan(false, |escaped, c| {
            if *escaped {
                *escaped = false;
                return Some(if c == '\0' {
                    "\\".to_string()
                } else {
                    match c {
                        '"' => "\"".to_string(),
                        '\\' => "\\".to_string(),
                        other => format!("\\{other}"),
                    }
                });
            }
            if c == '\\' {
                *escaped = true;
                return Some(String::new());
            }
            if c == '\0' {
                return None;
            }
            Some(c.to_string())
        })
        .collect()
}

/// The oracle's rankdir→flow map (Graphviz.pm `$rankdir`, gently-a0k):
/// LR→east, RL→west, TB→south, BT→north — any other value falls back to
/// east (`$rankdir->{$dir} || 'east'`).
pub(super) fn rankdir_flow(dir: &str) -> &'static str {
    match dir {
        "RL" => "west",
        "TB" => "south",
        "BT" => "north",
        _ => "east",
    }
}

#[cfg(test)]
mod tests {
    use super::unquote;

    #[test]
    fn unquotes_exactly_once() {
        assert_eq!(unquote("x \\\"y\\\\"), "x \"y\\");
        assert_eq!(unquote("plain"), "plain");
        assert_eq!(
            unquote("a\\nb"),
            "a\\nb",
            "unknown escapes keep the backslash"
        );
        assert_eq!(unquote("trailing\\"), "trailing\\");
    }
}
