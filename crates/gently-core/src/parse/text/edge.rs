//! Purpose: the edge-operator grammar of ge.text_parser (c2, c4, c9).
//! Responsibilities: match the four upstream operator alternatives —
//! directed (`->` family), inline-labeled with matching flanks, and the two
//! arrow-less forms — as an ordered backtracking search over unit tokens
//! (`= `, `=`, `- `, `-`, `..-`, `.-`, `.`, `~`), then map the match to
//! style, label, and bidirectionality. Rationale: the pinning evidence
//! (tests/repro/claims/operator-patterns.observed plus style probes under
//! the pinned oracle) shows the style comes from the upstream
//! `_edge_style` homogeneity tests — for directed operators effectively the
//! LAST unit, for arrow-less patterns the whole consumed text; the plain
//! `_edge_style` regexes (including its unescaped `(..-)+`) are replicated
//! literally so mixed arrow-less patterns inherit (solid) exactly as the
//! oracle does (`== [ b ]` is solid, `==[ b ]` is double).

use super::attrs::{collapse_ws, unescape};

/// Unit tokens in upstream priority order (longest/space forms first).
const UNITS: [&str; 8] = ["= ", "=", "- ", "-", "..-", ".-", ".", "~"];

/// The plain-unit set of the arrow-less alternatives (no `..-`/`.-`).
const PLAIN_UNITS: [&str; 6] = ["= ", "=", "- ", "-", ".", "~"];

/// Cap on left-flank candidates for the labeled search (labels and flanks
/// are short in practice; the candidate space is exponential in theory).
const MAX_FLANK_CANDIDATES: usize = 256;

/// A matched operator: its style (None = solid/inherit), optional inline
/// label, bidirectionality (`<`-prefixed directed operators), and whether
/// the operator carried an arrow (directed) or not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeOp {
    pub style: Option<&'static str>,
    pub label: Option<String>,
    pub bidirectional: bool,
    pub directed: bool,
}

/// A char that can start an edge operator (c2's operator table).
pub fn is_op_start(c: char) -> bool {
    matches!(c, '<' | '=' | '-' | '.' | '~')
}

/// The upstream `_edge_style` semantics (Parser.pm), replicated literally:
/// whole-pattern homogeneity tests in its exact order; `None` = inherit
/// (solid). Note `(..-)+` in upstream has UNESCAPED dots — any two chars
/// ending in `-` — hence the triple rule below.
pub fn edge_style(pattern: &str) -> Option<&'static str> {
    if is_reps_of(pattern, "= ") {
        Some("double-dash")
    } else if all_char(pattern, '=') {
        Some("double")
    } else if all_char(pattern, '.') {
        Some("dotted")
    } else if is_reps_of(pattern, "- ") {
        Some("dashed")
    } else if is_triples_ending_dash(pattern) {
        Some("dot-dot-dash")
    } else if is_reps_of(pattern, ".-") {
        Some("dot-dash")
    } else if all_char(pattern, '~') {
        Some("wave")
    } else {
        None
    }
}

fn all_char(s: &str, c: char) -> bool {
    !s.is_empty() && s.chars().all(|x| x == c)
}

/// True when `s` is one or more repetitions of the literal token `u`.
fn is_reps_of(s: &str, u: &str) -> bool {
    !s.is_empty()
        && s.len().is_multiple_of(u.len())
        && (0..s.len()).step_by(u.len()).all(|i| &s[i..i + u.len()] == u)
}

/// Upstream's unescaped `(..-)+\z`: length a multiple of 3 with every
/// third character `-` (matches `..-`, but also `- -`, `= =`, …).
fn is_triples_ending_dash(s: &str) -> bool {
    s.len() >= 3
        && s.len().is_multiple_of(3)
        && s.chars().enumerate().all(|(i, c)| i % 3 != 2 || c == '-')
}

/// Which operator alternative to match, in upstream order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    Directed,
    Labeled,
    DotArrowless,
    PlainArrowless,
}

/// Match one operator alternative at `pos`; `None` when it does not match.
/// Returns the position after the operator and the parsed `EdgeOp`. The
/// caller restores the cursor between attempts.
pub fn match_branch(chars: &[char], pos: usize, branch: Branch) -> Option<(usize, EdgeOp)> {
    match branch {
        Branch::Directed => try_directed(chars, pos),
        Branch::Labeled => try_labeled(chars, pos),
        Branch::DotArrowless => try_arrowless(chars, pos, &["..-", ".-"], 1),
        Branch::PlainArrowless => try_arrowless(chars, pos, &PLAIN_UNITS, 2),
    }
}

/// Branch 1: `(<?)` + one-or-more units + `>`; style from the last unit.
fn try_directed(chars: &[char], pos: usize) -> Option<(usize, EdgeOp)> {
    let (p, bidi) = skip_lt(chars, pos);
    let (gt, last) = dfs_to_gt(chars, p, None)?;
    let op = EdgeOp { style: edge_style(&last), label: None, bidirectional: bidi, directed: true };
    Some((gt + 1, op))
}

/// Branch 2: `(<?)` + flank + `\s+` + label + `\s+` + SAME flank + `>`
/// (c4: both flanks must match; the label may not contain `>`/`[`/`{`).
fn try_labeled(chars: &[char], pos: usize) -> Option<(usize, EdgeOp)> {
    let (p, bidi) = skip_lt(chars, pos);
    for (fend, flank) in flank_candidates(chars, p) {
        if let Some((end, raw)) = labeled_tail(chars, fend, &flank) {
            let label = collapse_ws(unescape(&raw).trim());
            let label = (!label.is_empty()).then_some(label);
            let op = EdgeOp { style: edge_style(&flank), label, bidirectional: bidi, directed: true };
            return Some((end, op));
        }
    }
    None
}

/// The arrow-less branches: branch 3 consumes only `..-`/`.-` units (at
/// least one); branch 4 only plain units (at least two, c9).
fn try_arrowless(chars: &[char], pos: usize, units: &[&str], min_units: usize) -> Option<(usize, EdgeOp)> {
    let (end, count) = greedy_units(chars, pos, units);
    if count < min_units {
        return None;
    }
    let pattern: String = chars[pos..end].iter().collect();
    let op = EdgeOp { style: edge_style(&pattern), label: None, bidirectional: false, directed: false };
    Some((end, op))
}

/// Consume units greedily (priority order per position); returns the end
/// position and the unit count.
fn greedy_units(chars: &[char], mut pos: usize, units: &[&str]) -> (usize, usize) {
    let mut count = 0;
    'outer: loop {
        for u in units {
            if unit_at(chars, pos, u) {
                pos += u.len();
                count += 1;
                continue 'outer;
            }
        }
        return (pos, count);
    }
}

/// The optional `<` prefix; returns the position after it and whether it
/// was present (c2: `<` requires the closing `>` — enforced by the `>` the
/// branch matchers need).
fn skip_lt(chars: &[char], pos: usize) -> (usize, bool) {
    match chars.get(pos) {
        Some('<') => (pos + 1, true),
        _ => (pos, false),
    }
}

fn unit_at(chars: &[char], pos: usize, u: &str) -> bool {
    u.chars().enumerate().all(|(i, uc)| chars.get(pos + i) == Some(&uc))
}

/// Depth-first search for a unit sequence ending at `>`, alternatives in
/// priority order — the first success is the upstream greedy match. The
/// last consumed unit is returned (c9: the style follows it).
fn dfs_to_gt(chars: &[char], pos: usize, last: Option<&str>) -> Option<(usize, String)> {
    for u in UNITS {
        if unit_at(chars, pos, u) {
            if let Some(r) = dfs_to_gt(chars, pos + u.len(), Some(u)) {
                return Some(r);
            }
        }
    }
    match (last, chars.get(pos)) {
        (Some(u), Some('>')) => Some((pos, u.to_string())),
        _ => None,
    }
}

/// All left-flank candidates in upstream backtracking order: deeper unit
/// extensions first (the greedy `+` explores another unit before exiting),
/// then the shorter flank at the choice point.
fn flank_candidates(chars: &[char], pos: usize) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut flank = String::new();
    walk_flanks(chars, pos, &mut flank, &mut out);
    out
}

fn walk_flanks(chars: &[char], pos: usize, flank: &mut String, out: &mut Vec<(usize, String)>) {
    for u in UNITS {
        if unit_at(chars, pos, u) {
            flank.push_str(u);
            walk_flanks(chars, pos + u.len(), flank, out);
            flank.truncate(flank.len() - u.len());
        }
    }
    if !flank.is_empty() && out.len() < MAX_FLANK_CANDIDATES {
        out.push((pos, flank.clone()));
    }
}

/// Try the labeled tail after a left flank ending at `fend`: `\s+`, the
/// lazy label, `\s+`, the same flank, `>` (c4). Returns the full consumed
/// end and the raw label text.
fn labeled_tail(chars: &[char], fend: usize, flank: &str) -> Option<(usize, String)> {
    let ls = skip_ws_run(chars, fend)?;
    let mut le = ls;
    loop {
        if let Some(end) = tail_end(chars, le, flank) {
            let label: String = chars[ls..le].iter().collect();
            return Some((end, label));
        }
        le = label_char(chars, le)?;
    }
}

/// `\s+` then the flank then `>` (c4's `(\s+\5)>`); returns the position
/// after the `>`.
fn tail_end(chars: &[char], pos: usize, flank: &str) -> Option<usize> {
    let ws_end = skip_ws_run(chars, pos)?;
    let fc: Vec<char> = flank.chars().collect();
    let after = ws_end + fc.len();
    let closes = chars[ws_end..].starts_with(&fc) && chars.get(after) == Some(&'>');
    closes.then_some(after + 1)
}

/// One or more whitespace characters; returns the position after the run.
fn skip_ws_run(chars: &[char], mut pos: usize) -> Option<usize> {
    let start = pos;
    while matches!(chars.get(pos), Some(c) if c.is_whitespace()) {
        pos += 1;
    }
    (pos > start).then_some(pos)
}

/// One label character: an escape pair (`\X` — any `X`) or any char but
/// `>`, `[`, `{`. Returns the position after it.
fn label_char(chars: &[char], pos: usize) -> Option<usize> {
    match chars.get(pos) {
        Some('\\') => chars.get(pos + 1).map(|_| pos + 2),
        Some(&c) if !matches!(c, '>' | '[' | '{') => Some(pos + 1),
        _ => None,
    }
}
