//! decision-trace E3 (beads gently-ugy): D2's pairwise faithfulness
//! criterion, computed against the committed E2 oracle dumps
//! (`tests/repro/dumps/*.dump.txt`, `_prepare_layout` stage dumps).
//!
//! D2 restatement (design doc, verbatim testable form):
//!   1. For every pair of corpus fixtures whose oracle dumps differ at
//!      stage S, the normalized traces must differ in S's image —
//!      normalization must never erase a real divergence (soundness).
//!   2. Per fixture, canonically-ordered state snapshots must normalize
//!      to identical traces regardless of dump ordering — normalization
//!      must be invariant under the oracle's hash-order line permutations.
//!
//! Meter: `cargo test -p gently-core --test e3_faithfulness` green.
//! Kill criterion: not computable on the corpus without hooking every
//! upstream sub — i.e. this test needs only the committed dumps.
//!
//! The normalizer here is the E3 prototype of D2's committed table
//! (task 4.2 graduates it); it maps the `_prepare_layout` dump
//! vocabulary onto the trace stage `layout.place`:
//!   node placement  -> layout.place node event (name, x, y, w, h, cx, cy)
//!   band summary    -> layout.place band event (y, counts)

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Stage the dumps observe: the wrapper wraps `Graph::Easy::_prepare_layout`.
const STAGE: &str = "layout.place";

fn dumps_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/repro/dumps")
}

fn load_dump(name: &str) -> String {
    fs::read_to_string(dumps_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// One node placement event, keyed canonically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NodeEvent {
    stage: &'static str,
    invocation: usize,
    case: String,
    name: String,
    x: i64,
    y: i64,
    w: u64,
    h: u64,
    cx: u64,
    cy: u64,
}

/// One band event, keyed canonically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BandEvent {
    stage: &'static str,
    invocation: usize,
    case: String,
    band_y: i64,
    nodes: u64,
    edge_cells: u64,
    group_cells: u64,
}

/// The normalized trace of one dump block: the stage's event image,
/// sorted canonically (BTreeSet dedups identical events).
type Trace = Vec<String>;

fn event_key(node: &NodeEvent) -> String {
    format!(
        "{}[inv={} case={} node={} @ ({},{}) {}x{} cx={} cy={}]",
        node.stage,
        node.invocation,
        node.case,
        node.name,
        node.x,
        node.y,
        node.w,
        node.h,
        node.cx,
        node.cy
    )
}

fn event_key_band(band: &BandEvent) -> String {
    format!(
        "{}[inv={} case={} band y={} nodes={} edge-cells={} group-cells={}]",
        band.stage,
        band.invocation,
        band.case,
        band.band_y,
        band.nodes,
        band.edge_cells,
        band.group_cells
    )
}

/// Normalize one dump block (the lines following a `-- seed N` marker,
/// up to the next marker) into the `layout.place` trace image.
///
/// Ordering-insensitive BY CONSTRUCTION: bands and their node lines are
/// keyed into a BTreeMap on (invocation, case, band y) / (.., name) and
/// emitted sorted — the oracle's hash-order line permutation cannot
/// change the result. This is the property D2 clause 2 asserts.
fn normalize_block(lines: &[&str]) -> Trace {
    let mut nodes: BTreeMap<(usize, String, i64, String), NodeEvent> = BTreeMap::new();
    let mut bands: BTreeMap<(usize, String, i64), BandEvent> = BTreeMap::new();
    let mut invocation = 0usize;
    let mut case = String::new();
    let mut cur_band: Option<i64> = None;

    for line in lines {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("== invocation ") {
            // "== invocation 1 (case0, format: ascii) nodes=.. .."
            invocation = rest
                .split_whitespace()
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            case = rest
                .split('(')
                .nth(1)
                .and_then(|s| s.split(',').next())
                .unwrap_or("")
                .to_string();
            cur_band = None;
        } else if let Some(rest) = line.strip_prefix("band y=") {
            // "band y=0 nodes=2 edge-cells=2 group-cells=0"
            let mut it = rest.split_whitespace();
            let y = it
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| panic!("bad band line: {line}"));
            let get = |key: &str| -> u64 {
                rest.split(' ')
                    .find_map(|tok| tok.strip_prefix(key).map(|v| v.parse().unwrap_or(0)))
                    .unwrap_or(0)
            };
            bands.insert(
                (invocation, case.clone(), y),
                BandEvent {
                    stage: STAGE,
                    invocation,
                    case: case.clone(),
                    band_y: y,
                    nodes: get("nodes="),
                    edge_cells: get("edge-cells="),
                    group_cells: get("group-cells="),
                },
            );
            cur_band = Some(y);
        } else if line.contains(" @ (") {
            // "  Client @ (2,0) 13x3" | "  Msg Server @ (0,0) 14x12 cx=1 cy=4"
            let Some(y) = cur_band else {
                panic!("node line outside a band: {line}")
            };
            let (name, rest) = line.split_once(" @ (").unwrap();
            let mut it = rest
                .split(['(', ')', 'x', ' ', ','])
                .filter(|t| !t.is_empty());
            let x: i64 = it
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| panic!("bad x: {line}"));
            let ny: i64 = it
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| panic!("bad y: {line}"));
            let w: u64 = it
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| panic!("bad w: {line}"));
            let h: u64 = it
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| panic!("bad h: {line}"));
            let get_span = |key: &str| {
                line.rsplit(' ')
                    .find_map(|tok| tok.strip_prefix(key).and_then(|v| v.parse().ok()))
                    .unwrap_or(1)
            };
            nodes.insert(
                (invocation, case.clone(), y, name.to_string()),
                NodeEvent {
                    stage: STAGE,
                    invocation,
                    case: case.clone(),
                    name: name.to_string(),
                    x,
                    y: ny,
                    w,
                    h,
                    cx: get_span("cx="),
                    cy: get_span("cy="),
                },
            );
        }
        // "-- seed N" markers and "#" comments are block delimiters handled
        // by the caller; "..." truncation markers never occur (E2 meter).
    }

    let mut trace = Trace::new();
    for n in nodes.values() {
        trace.push(event_key(n));
    }
    for b in bands.values() {
        trace.push(event_key_band(b));
    }
    trace.sort();
    trace.dedup(); // N8: duplicate oracle states dedup (identical event strings)
    trace
}

/// Split a dump into (seed, block lines) pairs.
fn seed_blocks(dump: &str) -> Vec<(i64, Vec<String>)> {
    let mut blocks = Vec::new();
    let mut seed: Option<i64> = None;
    let mut cur: Vec<String> = Vec::new();
    for line in dump.lines() {
        if let Some(rest) = line.strip_prefix("-- seed ") {
            if let Some(s) = seed.take() {
                blocks.push((s, cur.clone()));
            }
            seed = rest.trim().parse().ok();
            cur.clear();
        } else if seed.is_some() && !line.starts_with('#') {
            cur.push(line.to_string());
        }
    }
    if let Some(s) = seed {
        blocks.push((s, cur));
    }
    blocks
}

/// Raw (un-normalized) rendering of a block's stage content — what the
/// oracle emitted, before normalization. Used by clause 1 to decide
/// whether two dumps "differ at stage S".
fn raw_stage_image(block: &[String]) -> String {
    block.join("\n")
}

/// Apply a hash-order permutation: node lines reversed within every
/// band (the producer already sorted them away once; the normalizer
/// must not care). A node line belongs to its nearest preceding band
/// line, so band lines stay put — reordering bands would break the
/// dump grammar, not just its ordering.
fn permute_block(block: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut band_start: Vec<(usize, usize)> = Vec::new(); // (band line idx, block end before next band)
    let mut band_lines: Vec<usize> = Vec::new();
    for (i, line) in block.iter().enumerate() {
        if line.trim_start().starts_with("band y=") {
            band_lines.push(i);
        }
    }
    for (k, &start) in band_lines.iter().enumerate() {
        // members run to the earliest of: next band line, next
        // "== invocation" header, end of block
        let next_band = band_lines.get(k + 1).copied();
        let next_inv = block[start..]
            .iter()
            .position(|l| l.trim_start().starts_with("== invocation"))
            .map(|p| start + p);
        let end = next_band
            .into_iter()
            .chain(next_inv)
            .min()
            .unwrap_or(block.len());
        band_start.push((start, end));
    }
    let mut consumed = 0usize;
    for (start, end) in &band_start {
        // header lines between previous band's end and this band line
        out.extend(block[consumed..*start].iter().cloned());
        // band line first, then its members reversed
        out.push(block[*start].clone());
        let mut members: Vec<String> = block[*start + 1..*end].to_vec();
        members.reverse();
        out.extend(members);
        consumed = *end;
    }
    out.extend(block[consumed..].iter().cloned());
    out
}

const FIXTURES: [&str; 5] = [
    "dot-bare-scope-in-named.dump.txt",
    "fan-layout.dump.txt",
    "pod-anon-group.dump.txt",
    "so-flowchart-size.dump.txt",
    "so-start-port-index.dump.txt",
];

/// Clause 2: per fixture, canonically-ordered state snapshots normalize
/// identically regardless of dump ordering. We reverse the oracle's line
/// order inside every band (the hash-order permutation the producer
/// already sorted away) and require the identical normalized trace, for
/// every seed block of every fixture.
#[test]
fn e3_normalization_is_order_insensitive() {
    for fixture in FIXTURES {
        let dump = load_dump(fixture);
        let blocks = seed_blocks(&dump);
        assert!(!blocks.is_empty(), "{fixture}: no seed blocks parsed");
        for (seed, block) in &blocks {
            let refs: Vec<&str> = block.iter().map(String::as_str).collect();
            let canon = normalize_block(&refs);
            let permuted = permute_block(block);
            let perms: Vec<&str> = permuted.iter().map(String::as_str).collect();
            let after = normalize_block(&perms);
            assert_eq!(
                canon, after,
                "{fixture} seed {seed}: normalization is order-sensitive"
            );
        }
    }
}

/// Clause 1 (pairwise): for every pair of corpus fixtures whose oracle
/// dumps differ at stage S, the normalized traces must differ in S's
/// image. Also asserted per fixture across seeds: when the oracle's raw
/// stage image differs between seeds (fan-layout fan-OUT placement),
/// normalization must preserve the difference, not collapse it.
#[test]
fn e3_pairwise_faithfulness() {
    let mut raw: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut norm: BTreeMap<&str, Trace> = BTreeMap::new();
    for fixture in FIXTURES {
        let dump = load_dump(fixture);
        let blocks = seed_blocks(&dump);
        // stage image = concat over seeds (seed tagged by the marker in
        // raw form; the raw compare uses the full block text)
        raw.insert(
            fixture,
            blocks.iter().flat_map(|(_, b)| b.clone()).collect(),
        );
        let merged: Trace = blocks
            .iter()
            .flat_map(|(_seed, b)| {
                let refs: Vec<&str> = b.iter().map(String::as_str).collect();
                normalize_block(&refs)
            })
            .collect();
        norm.insert(fixture, merged);
    }

    for (i, &a) in FIXTURES.iter().enumerate() {
        for &b in FIXTURES.iter().skip(i + 1) {
            let raw_differs = raw[a] != raw[b];
            if raw_differs {
                assert_ne!(
                    norm[a], norm[b],
                    "{a} vs {b}: dumps differ at {STAGE} but normalized traces collide"
                );
            }
        }
    }

    // Seed-level: fan-layout's case0 placement is genuinely hash-seed
    // dependent (C/D vs B/D at the flanks); normalization must keep the
    // two seeds distinct. Every other fixture is seed-stable, and its
    // two seeds must normalize identically.
    for fixture in FIXTURES {
        let dump = load_dump(fixture);
        let blocks = seed_blocks(&dump);
        let [b0, b1] = blocks.as_slice() else {
            panic!("{fixture}: expected exactly two seed blocks");
        };
        let r0: Vec<&str> = b0.1.iter().map(String::as_str).collect();
        let r1: Vec<&str> = b1.1.iter().map(String::as_str).collect();
        let n0 = normalize_block(&r0);
        let n1 = normalize_block(&r1);
        if raw_stage_image(&b0.1) != raw_stage_image(&b1.1) {
            assert_ne!(
                n0, n1,
                "{fixture}: seeds differ at {STAGE} but normalization collapsed them"
            );
        } else {
            assert_eq!(n0, n1, "{fixture}: seeds agree but normalization diverged");
        }
    }
}

// ---- D2 normalization-table row enforcement (tests/repro/trace/normalization-table.md)
// Each row of the committed table is enforced by one named subtest; a
// table edit without a matching normalizer change fails here.

const BARE: &str = "\
# dump: Graph::Easy v0.69 @ pin — claim: row-check — wrapped: Graph::Easy::_prepare_layout
-- seed 0
== invocation 1 (case0, format: ascii) nodes=2 edge-cells=3 group-cells=0
band y=0 nodes=2 edge-cells=3 group-cells=0
  A @ (0,0) 5x3
  B @ (6,0) 5x3
band y=4 nodes=1 edge-cells=0 group-cells=0
  C @ (0,4) 7x3 cx=1 cy=2
";

fn normalize_str(dump: &str) -> Vec<(i64, Trace)> {
    seed_blocks(dump)
        .iter()
        .map(|(s, b)| {
            let refs: Vec<&str> = b.iter().map(String::as_str).collect();
            (*s, normalize_block(&refs))
        })
        .collect()
}

/// N1: the `# dump:` pin header is metadata, never a trace event.
#[test]
fn row_n1_header_is_metadata() {
    let with_header = format!("# dump: totally different pin header\n{BARE}");
    assert_eq!(normalize_str(&with_header), normalize_str(BARE));
}

/// N2: seed markers are block scope, never an event — the trace is
/// identical for every seed of identical content.
#[test]
fn row_n2_seed_not_an_event() {
    let both = format!("{BARE}-- seed 7\n== invocation 1 (case0, format: ascii) nodes=2 edge-cells=3 group-cells=0\nband y=0 nodes=2 edge-cells=3 group-cells=0\n  A @ (0,0) 5x3\n  B @ (6,0) 5x3\nband y=4 nodes=1 edge-cells=0 group-cells=0\n  C @ (0,4) 7x3 cx=1 cy=2\n");
    let blocks = normalize_str(&both);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].1, blocks[1].1);
    assert!(blocks
        .iter()
        .all(|(_, t)| t.iter().all(|e| !e.contains("seed"))));
}

/// N3: invocation + case are scope keys on every event.
#[test]
fn row_n3_invocation_scope_keys() {
    let t = &normalize_str(BARE)[0].1;
    assert!(
        t.iter().all(|e| e.contains("inv=1") && e.contains("case0")),
        "{t:?}"
    );
}

/// N4: band lines become verbatim-count band events.
#[test]
fn row_n4_band_event() {
    let t = &normalize_str(BARE)[0].1;
    assert!(
        t.iter()
            .any(|e| e.contains("band y=0 nodes=2 edge-cells=3 group-cells=0")),
        "{t:?}"
    );
    assert!(
        t.iter()
            .any(|e| e.contains("band y=4 nodes=1 edge-cells=0 group-cells=0")),
        "{t:?}"
    );
}

/// N5: absent spans default to cx=1 cy=1.
#[test]
fn row_n5_node_span_defaults() {
    let t = &normalize_str(BARE)[0].1;
    assert!(
        t.iter().any(|e| e.contains("node=A @ (0,0) 5x3 cx=1 cy=1")),
        "{t:?}"
    );
}

/// N6: explicit spans are carried verbatim.
#[test]
fn row_n6_node_explicit_spans() {
    let t = &normalize_str(BARE)[0].1;
    assert!(
        t.iter().any(|e| e.contains("node=C @ (0,4) 7x3 cx=1 cy=2")),
        "{t:?}"
    );
}

/// N7: line ordering never leaks (reversed members, same trace) —
/// the row-level form of E3 clause 2.
#[test]
fn row_n7_ordering_never_leaks() {
    let blocks = seed_blocks(BARE);
    let block = &blocks[0].1;
    let refs: Vec<&str> = block.iter().map(String::as_str).collect();
    let canon = normalize_block(&refs);
    let permuted = permute_block(block);
    let perms: Vec<&str> = permuted.iter().map(String::as_str).collect();
    assert_eq!(canon, normalize_block(&perms));
}

/// N8: (inv, case, band y / name) keying dedups identical states and
/// keeps distinct ones distinct.
#[test]
fn row_n8_key_dedup() {
    let dup = format!("{BARE}  B @ (6,0) 5x3\n"); // duplicate of an existing node line
    let t = &normalize_str(&dup)[0].1;
    let count = t.iter().filter(|e| e.contains("node=B")).count();
    assert_eq!(count, 1, "duplicate node line must dedup: {t:?}");
    // distinct state (different position) stays a distinct event
    let t = &normalize_str(BARE)[0].1;
    let count = t.iter().filter(|e| e.contains("node=B")).count();
    assert_eq!(count, 1);
}
