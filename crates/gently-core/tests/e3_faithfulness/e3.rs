use super::rows::*;
use super::*;

/// Clause 2: per fixture, canonically-ordered state snapshots normalize
/// identically regardless of dump ordering. We reverse the oracle's line
/// order inside every band (the hash-order permutation the producer
/// already sorted away) and require the identical normalized trace, for
/// every seed block of every fixture.
#[test]
pub(crate) fn e3_normalization_is_order_insensitive() {
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
pub(crate) fn e3_pairwise_faithfulness() {
    let fixtures = FIXTURES.len();
    let (raw, norm) = build_traces();
    assert_eq!(raw.len(), fixtures, "every fixture yields a raw trace");
    assert_pairwise_distinct(&raw, &norm);
    assert_seed_stability();
}

/// Load every fixture's raw seed blocks and normalized traces.
fn build_traces() -> (
    BTreeMap<&'static str, Vec<String>>,
    BTreeMap<&'static str, Trace>,
) {
    let mut raw: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut norm: BTreeMap<&str, Trace> = BTreeMap::new();
    for fixture in FIXTURES {
        let dump = load_dump(fixture);
        let blocks = seed_blocks(&dump);
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
    (raw, norm)
}

/// Fixtures whose raw dumps differ must not normalize to the same trace.
fn assert_pairwise_distinct(
    raw: &BTreeMap<&'static str, Vec<String>>,
    norm: &BTreeMap<&'static str, Trace>,
) {
    let pairs = FIXTURES
        .iter()
        .enumerate()
        .flat_map(|(i, &a)| FIXTURES.iter().skip(i + 1).map(move |&b| (a, b)));
    for (a, b) in pairs {
        if raw[a] != raw[b] {
            assert_ne!(
                norm[a], norm[b],
                "{a} vs {b}: dumps differ at {STAGE} but normalized traces collide"
            );
        }
    }
}

/// Seed-level: fan-layout's case0 placement is genuinely hash-seed
/// dependent (C/D vs B/D at the flanks); normalization must keep the
/// two seeds distinct. Every other fixture is seed-stable, and its
/// two seeds must normalize identically.
fn assert_seed_stability() {
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
