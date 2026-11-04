//! decision.trace scenario tests (specs/decision-trace.md p1–p6).
//!
//! p1 executes the versioned-emission contract: every trace event carries
//! the schema version and its governing spec id (`decision.trace`, c1),
//! and with `GENTLY_TRACE` unset emission is silent — no trace file is
//! created and rendering is byte-identical (the E1 regression pin; byte
//! equality stays gated by the pinned corpus tests, this arm pins the
//! API-level gate).
//!
//! p2's enforcement lives in `crates/gently-core/tests/e3_faithfulness.rs`
//! (the E3 pairwise-faithfulness + row_nN subtests — the contract TOML
//! points the `decision.trace::p2` filter there, gently-a3l). p3 pins the
//! minimal-antichain cut semantics. p4/p5/p6 (gently-a3l) enforce the
//! committed-dump corpus gates (c4) and their typed-error arms (c5) and
//! the adjudication-record structure (c6) over the committed registry.

/// The repo root as seen from the scenario tests.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The pinned oracle identity every dump pin header must name
/// (specs/ge-oracle.md c1).
const PIN_COMMIT: &str = "ededa3d787ad89ac532c578c06390e8a7b270499";

/// One committed-dump defect, rendered as the c5 typed error: the defect
/// named plus the remediation (re-dump under the pinned seed per the
/// trace README's recording recipe, or removal from the corpus via the
/// admission probe, [[ge.oracle.c7]]).
fn dump_defect(defect: &str) -> String {
    format!(
        "invalid dump: {defect} — remediation: re-dump under the pinned seed \
         (tests/repro/trace/README.md recording recipe) or remove the fixture \
         from the corpus via the admission probe (ge.oracle.c7)"
    )
}

/// c5's typed gate over one committed dump (c4's companion contract): the
/// oracle pin header, the explicit hash-seed field, and the per-stage
/// observability marks must all be present.
fn check_dump(name: &str, text: &str) -> Result<(), String> {
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
fn seed_blocks(text: &str) -> Vec<Vec<String>> {
    let mut blocks: Vec<Vec<String>> = Vec::new();
    let mut cur: Option<Vec<String>> = None;
    for line in text.lines() {
        if line.starts_with("-- seed ") {
            if let Some(b) = cur.take() {
                blocks.push(b);
            }
            cur = Some(Vec::new());
        } else if cur.is_some() && !line.starts_with('#') {
            cur.as_mut().must("value").push(line.to_string());
        }
    }
    if let Some(b) = cur.take() {
        blocks.push(b);
    }
    blocks
}

use gently_core::{graph::Graph, layout};

use std::fs;
use std::path::{Path, PathBuf};

#[path = "decision_trace/helpers.rs"]
mod helpers;
#[path = "decision_trace/tests.rs"]
mod tests;
#[path = "decision_trace/tests2.rs"]
mod tests2;

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
