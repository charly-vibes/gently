//! The trace-diff driver (specs/decision-trace.md c3/p3, design D3/D6):
//! diffs two decision traces over the stage vocabulary and reports the
//! minimal antichain (cut set) of differing events — never a unique
//! "first" (a linear presentation is presentation only).
//!
//! c3: divergence is a set over the decision dependency poset. The poset
//! here is the [`crate::trace::STAGES`] pipeline order: an event at a
//! later stage depends on the decisions of the earlier stages, and
//! within one stage the per-object events (per-node placements, per-edge
//! routes) are mutually incomparable — an antichain. The minimal cut is
//! therefore the set of differing events at the EARLIEST divergent stage:
//! every earlier stage agrees, so any differing event's predecessors
//! agree, and reporting all differing events of that stage (keyed by
//! identity) is exactly the minimal antichain. Promoted from the E1
//! retrospective prototype (gently-asc / tests/repro/trace/e1.sh, whose
//! prototype-injection hack is obsolete: this driver runs against the
//! library trace as committed at any rev).
//!
//! The byte gate (5.2): trace-diff runs only on byte-divergent fixtures.
//! When the two rendered byte strings are identical and the traces still
//! differ, the diff is decision-only — nothing to adjudicate — and the
//! strict call refuses with the typed [`TraceDiffError::DecisionOnly`];
//! the caller may pass `informational = true` to see the diff anyway.
//! Byte equality itself stays the CI gate (the pinned corpus tests).

use crate::trace::STAGES;
use std::collections::BTreeMap;
use std::fmt;

/// One cut event: an identity-keyed differing event pair at the cut
/// stage. `a`/`b` carry the event payload on the side that has it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutEvent {
    pub stage: String,
    pub key: String,
    pub a: Option<String>,
    pub b: Option<String>,
}

/// The typed byte-gate refusal (c3, 5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceDiffError {
    /// Rendered bytes are identical and the traces differ: nothing to
    /// adjudicate — the diff is informational only.
    DecisionOnly { cut: Vec<CutEvent> },
}

/// The hint the byte gate reports (c3: downgraded to informational).
pub const INFORMATIONAL_HINT: &str =
    "decision-only diff — informational, run with --informational to see it";

impl fmt::Display for TraceDiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceDiffError::DecisionOnly { .. } => f.write_str(INFORMATIONAL_HINT),
        }
    }
}

impl std::error::Error for TraceDiffError {}

/// Parse trace lines into `(stage, payload)` pairs. A line that does not
/// carry the versioned emission header (`dt v… stage=… spec=…`) is not a
/// trace event and is skipped — the dump alignment shim only emits real
/// events, so foreign dump metadata (headers, band summaries, seeds)
/// never leaks into the diff.
pub fn parse_lines(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("dt v")?;
            let rest = rest.split_once(' ')?.1;
            let rest = rest.strip_prefix("stage=")?;
            let (stage, rest) = rest.split_once(' ')?;
            let rest = rest.strip_prefix("spec=")?;
            let payload = rest.split_once(' ')?.1;
            Some((stage.to_string(), payload.to_string()))
        })
        .collect()
}

/// The identity key of an event payload: the meaningful prefix before
/// the trailing attribute assignments (`k=v` tokens). `place A x=0 y=0`
/// keys as `place A`; `route e0 A->B cells=5` keys as `route e0 A->B`;
/// `stmt line=0 kind=edge` keeps its whole payload (stripping would
/// collapse every statement into one key). Within a stage, the key is
/// the object identity the stage decides about — per-node placements,
/// per-edge routes, per-line statements.
fn event_key(payload: &str) -> String {
    let tokens: Vec<&str> = payload.split_whitespace().collect();
    let trailing_attrs = tokens.iter().rev().take_while(|t| t.contains('=')).count();
    let meaningful = &tokens[..tokens.len() - trailing_attrs];
    if meaningful.len() > 1 {
        meaningful.join(" ")
    } else {
        payload.to_string()
    }
}

/// Index a trace into `stage -> key -> payload` (later events win on a
/// repeated identity — the stage's final decision about that object).
fn index_trace(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut index: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for (stage, payload) in parse_lines(text) {
        let key = event_key(&payload);
        index.entry(stage).or_default().insert(key, payload);
    }
    index
}

fn stage_rank(stage: &str) -> usize {
    STAGES
        .iter()
        .position(|s| *s == stage)
        .unwrap_or(usize::MAX)
}

/// The minimal antichain (cut set) of differing events between two
/// traces: the differing identity-keyed events of the earliest divergent
/// stage, ordered by the trace's stage vocabulary then by key. Traces
/// with no differing event yield an empty cut.
pub fn minimal_cut(a: &str, b: &str) -> Vec<CutEvent> {
    let (ia, ib) = (index_trace(a), index_trace(b));
    let mut stages: Vec<&String> = ia.keys().collect();
    for stage in ib.keys() {
        if !ia.contains_key(stage) {
            stages.push(stage);
        }
    }
    stages.sort_by_key(|s| (stage_rank(s), (*s).clone()));
    for stage in stages {
        let empty = BTreeMap::new();
        let (sa, sb) = (
            ia.get(stage).unwrap_or(&empty),
            ib.get(stage).unwrap_or(&empty),
        );
        let mut cut = Vec::new();
        let keys: std::collections::BTreeSet<&String> = sa.keys().chain(sb.keys()).collect();
        for key in keys {
            match (sa.get(key), sb.get(key)) {
                (Some(pa), Some(pb)) if pa == pb => {}
                (Some(pa), Some(pb)) => cut.push(CutEvent {
                    stage: (*stage).clone(),
                    key: key.clone(),
                    a: Some(pa.clone()),
                    b: Some(pb.clone()),
                }),
                (Some(pa), None) => cut.push(CutEvent {
                    stage: (*stage).clone(),
                    key: key.clone(),
                    a: Some(pa.clone()),
                    b: None,
                }),
                (None, Some(pb)) => cut.push(CutEvent {
                    stage: (*stage).clone(),
                    key: key.clone(),
                    a: None,
                    b: Some(pb.clone()),
                }),
                (None, None) => unreachable!("key came from one of the sides"),
            }
        }
        if !cut.is_empty() {
            return cut; // earliest divergent stage owns the cut
        }
    }
    Vec::new()
}

/// The full trace-diff entry point (c3/p3): the minimal cut between the
/// two traces, gated by the byte comparison. Byte-identical fixtures
/// with differing traces refuse with [`TraceDiffError::DecisionOnly`]
/// unless `informational` is set; byte-divergent fixtures always
/// adjudicate. Byte equality itself remains the CI gate.
pub fn trace_diff(
    bytes_a: &str,
    bytes_b: &str,
    trace_a: &str,
    trace_b: &str,
    informational: bool,
) -> Result<Vec<CutEvent>, TraceDiffError> {
    let cut = minimal_cut(trace_a, trace_b);
    if bytes_a.trim_end() == bytes_b.trim_end() {
        if cut.is_empty() {
            return Ok(Vec::new());
        }
        if !informational {
            return Err(TraceDiffError::DecisionOnly { cut });
        }
    }
    Ok(cut)
}

/// The alignment shim promoted from the E1 driver: normalize an oracle
/// `_prepare_layout` dump's seed-0, invocation-1, band-0 node lines into
/// `layout.place` trace lines. A dump line
/// `  Client @ (8,0) 10x10 cx=5 cy=10` becomes
/// `dt v1 stage=layout.place spec=decision.trace place Client x=8 y=0 cx=5 cy=10`;
/// a single-celled node (`  Server @ (0,0) 10x4`, no cx/cy) maps to
/// `cx=1 cy=1`. The oracle's cx/cy are its cell spans — the same units
/// gently's trace emits — so the lines are directly comparable.
pub fn align_dump(dump: &str) -> String {
    let mut out = String::new();
    let mut in_seed0 = false;
    let mut in_invocation1 = false;
    let mut in_band0 = false;
    for line in dump.lines() {
        if line.starts_with("-- seed ") {
            in_seed0 = line == "-- seed 0";
            in_invocation1 = false;
            in_band0 = false;
            continue;
        }
        if !in_seed0 {
            continue;
        }
        if line.starts_with("== invocation ") {
            in_invocation1 = line.starts_with("== invocation 1 ");
            in_band0 = false;
            continue;
        }
        if !in_invocation1 {
            continue;
        }
        if line.starts_with("band y=") {
            in_band0 = line.starts_with("band y=0 ");
            continue;
        }
        if in_band0 && line.starts_with("  ") {
            if let Some((name, x, y, cx, cy)) = parse_dump_node(line.trim_start()) {
                out.push_str(&format!(
                    "dt v1 stage=layout.place spec=decision.trace place {name} x={x} y={y} cx={cx} cy={cy}\n"
                ));
            }
        }
    }
    out
}

/// Parse one aligned dump node line: `Name @ (x,y) WxH cx=C cy=R` (the
/// cx/cy tail optional). Returns (name, x, y, cx, cy).
fn parse_dump_node(line: &str) -> Option<(String, i64, i64, u64, u64)> {
    let (name, rest) = line.split_once(" @ (")?;
    let (xy, dims_tail) = rest.split_once(") ")?;
    let (x, y) = xy.split_once(',')?;
    let cx = attr(dims_tail, "cx=").unwrap_or(1);
    let cy = attr(dims_tail, "cy=").unwrap_or(1);
    Some((
        name.trim().to_string(),
        x.parse().ok()?,
        y.parse().ok()?,
        cx,
        cy,
    ))
}

fn attr(tail: &str, prefix: &str) -> Option<u64> {
    tail.split_whitespace()
        .find_map(|t| t.strip_prefix(prefix))
        .and_then(|v| v.parse().ok())
}
