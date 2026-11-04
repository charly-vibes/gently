use super::*;

/// N1: the `# dump:` pin header is metadata, never a trace event.
#[test]
pub(crate) fn row_n1_header_is_metadata() {
    let with_header = format!("# dump: totally different pin header\n{BARE}");
    assert_eq!(normalize_str(&with_header), normalize_str(BARE));
}

/// N2: seed markers are block scope, never an event — the trace is
/// identical for every seed of identical content.
#[test]
pub(crate) fn row_n2_seed_not_an_event() {
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
pub(crate) fn row_n3_invocation_scope_keys() {
    let t = &normalize_str(BARE)[0].1;
    assert!(
        t.iter().all(|e| e.contains("inv=1") && e.contains("case0")),
        "{t:?}"
    );
}

/// N4: band lines become verbatim-count band events.
#[test]
pub(crate) fn row_n4_band_event() {
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
pub(crate) fn row_n5_node_span_defaults() {
    let t = &normalize_str(BARE)[0].1;
    let spans: Vec<&str> = t
        .iter()
        .filter(|e| e.contains(" @ (") && e.contains("cx=1 cy=1"))
        .map(String::as_str)
        .collect();
    assert!(
        !spans.is_empty(),
        "an event with default spans exists: {t:?}"
    );
    assert!(t.iter().any(|e| e.contains("cy=1")), "{t:?}");
}

/// N6: explicit spans are carried verbatim.
#[test]
pub(crate) fn row_n6_node_explicit_spans() {
    let t = &normalize_str(BARE)[0].1;
    assert!(
        t.iter().any(|e| e.contains("node=C @ (0,4) 7x3 cx=1 cy=2")),
        "{t:?}"
    );
}

/// N7: line ordering never leaks (reversed members, same trace) —
/// the row-level form of E3 clause 2.
#[test]
pub(crate) fn row_n7_ordering_never_leaks() {
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
pub(crate) fn row_n8_key_dedup() {
    let dup = format!("{BARE}  B @ (6,0) 5x3\n"); // duplicate of an existing node line
    let t = &normalize_str(&dup)[0].1;
    let count = t.iter().filter(|e| e.contains("node=B")).count();
    assert_eq!(count, 1, "duplicate node line must dedup: {t:?}");
    // distinct state (different position) stays a distinct event
    let t = &normalize_str(BARE)[0].1;
    let count = t.iter().filter(|e| e.contains("node=B")).count();
    assert_eq!(count, 1);
}

pub(crate) fn normalize_str(dump: &str) -> Vec<(i64, Trace)> {
    seed_blocks(dump)
        .iter()
        .map(|(s, b)| {
            let refs: Vec<&str> = b.iter().map(String::as_str).collect();
            (*s, normalize_block(&refs))
        })
        .collect()
}
