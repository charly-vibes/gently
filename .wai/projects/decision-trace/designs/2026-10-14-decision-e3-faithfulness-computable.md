# Decision: E3 faithfulness criterion is computable — normalization may proceed (gently-ugy)

**Date:** 2026-10-14
**Gates:** D2 (normalization table, task 4.2) + the option-B overfit-risk row in the D7 matrix
**Meter (pre-registered):** `cargo test -p gently-core --test e3_faithfulness` green on the committed E2 dumps
**Kill criterion:** pairwise test not computable on the corpus without hooking every upstream sub

## Verdict: PASS — computable

D2's pairwise restated faithfulness criterion holds on the five committed
`_prepare_layout` dumps (`tests/repro/dumps/`), implemented as
`crates/gently-core/tests/e3_faithfulness.rs`:

1. **Pairwise soundness** (`e3_pairwise_faithfulness`): for every pair of
   fixtures whose raw oracle dumps differ at stage `layout.place` (all 10
   pairs do), the normalized traces differ too — normalization erases no
   real divergence. Also asserted at seed granularity: fan-layout's
   genuinely hash-seed-dependent case0 placement (C/D vs B/D flanks)
   stays distinct after normalization; the four seed-stable fixtures
   normalize identically across seeds.
2. **Ordering invariance** (`e3_normalization_is_order_insensitive`):
   per fixture, per seed, per invocation — reversing the node lines
   within every band (the hash-order permutation the dump producer
   already sorts away) yields the identical normalized trace. The
   normalizer keys events on (invocation, case, band y) / (.., name)
   and emits sorted, so ordering cannot leak into the trace.

Discrimination check: mutating the normalizer to drop node events fails
the seed clause (`fan-layout: seeds differ at layout.place but
normalization collapsed them`) — the criterion bites, it is not vacuous.

## Cost / computability evidence (kill-criterion check)

- Only inputs: the 5 committed dumps (5130 B total). Zero additional
  upstream hooks — `_prepare_layout` alone (E2's wrapper) suffices.
- Normalizer ≈ 150 lines of Rust, parser keyed on the dump grammar
  (`== invocation`, `band y=`, `name @ (x,y) WxH [cx= cy=]`).
- Kill criterion NOT met; D2 is writable as a committed, tested table.

## Consequences

- **Task 4.2 unblocked:** the normalization table may land as the
  committed artifact (graduating the E3 prototype out of the test file).
- **D7 matrix, option B overfit-risk row:** the "5 fixtures only — E3
  faithfulness gates scale-out" caveat resolves to *cleared*; storage
  scale-out beyond the five fixtures is no longer gated by computability
  (cost is the remaining constraint, D4).
- fan-layout's seed dependence is a feature of the data, not a bug: the
  pairwise clause uses it to prove normalization preserves seed noise.
  Tier-2 exclusion (task 3.3) still applies to *committed-dump corpus*
  membership decisions.
