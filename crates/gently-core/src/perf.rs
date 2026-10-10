//! Purpose: typed performance budget gates for specs/ge-perf.md (c4).
//! Responsibilities: `check` compares a measured duration against a
//! stage's budget and returns a typed [`BudgetViolation`] naming the
//! stage, the budget, and the measured value; `exit_code` carries the
//! non-zero exit semantics that `just perf-check` surfaces when a gate
//! fails (the cargo test run fails, which fails the recipe).
//! Rationale: c4 demands a machine-readable diagnostic with exit
//! semantics, not a bare assertion — the CLI and the just recipe can both
//! consume the same typed violation.
//!
//! Owner: gently-4ln (specs/ge-perf.md c4).

use std::fmt;
use std::time::Duration;

/// A budget violation (ge.perf.c4): the stage that exceeded its budget,
/// the budget itself, and the measured duration. Formats as a diagnostic
/// naming all three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetViolation {
    /// Pipeline stage whose budget was exceeded (e.g. `layout`,
    /// `ascii_render`).
    pub stage: String,
    /// The budget that was violated.
    pub budget: Duration,
    /// The measured duration that exceeded `budget`.
    pub measured: Duration,
}

impl BudgetViolation {
    /// Non-zero exit code for `just perf-check` (ge.perf.c4): a budget
    /// violation must fail the recipe run, so it maps to exit code 1.
    pub fn exit_code(&self) -> i32 {
        1
    }
}

impl fmt::Display for BudgetViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "perf budget exceeded: stage={} budget={:?} measured={:?}",
            self.stage, self.budget, self.measured
        )
    }
}

impl std::error::Error for BudgetViolation {}

/// Check a measured duration against a stage budget (ge.perf.c4):
/// `Ok(())` when the stage stayed within budget, a typed
/// [`BudgetViolation`] naming stage/budget/measured value otherwise.
pub fn check(stage: &str, budget: Duration, measured: Duration) -> Result<(), BudgetViolation> {
    if measured <= budget {
        Ok(())
    } else {
        Err(BudgetViolation {
            stage: stage.to_string(),
            budget,
            measured,
        })
    }
}
