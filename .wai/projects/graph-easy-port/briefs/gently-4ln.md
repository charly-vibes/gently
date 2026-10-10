# Subagent brief: gently-4ln — implement ge.perf (budget scenario tests + just perf-check)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-4ln` and is hands-off: you own
the implementation end-to-end.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-4ln`
- Read `specs/ge-perf.md` IN FULL — c1–c5 + properties p1–p5 are the
  binding contract, re-scoped by gently-k4u (commit 23a7de6): c2/c5 are
  now Tier-3 gently-only budgets with MEASURED headroom (1000 nodes ≈
  13 ms / 4.4 MB release — enormous slack). c1: 20-node cold-start
  under 50 ms (debug-test runs must account for the slowness — see
  below). c4: typed diagnostic naming stage/budget/measured value,
  non-zero exit via `just perf-check`.
- Ground truth from k4u measurements: release builds do 1000 nodes in
  ~13 ms; the 50 ms / 2 s budgets have huge headroom.
- Existing: justfile `perf-check` recipe (line ~121) currently exits 1
  "implementation phase not started" — you REPLACE that stub. The
  `.espectacular/ge-perf/p1–p5.toml` flags are stale
  (`scenarios::ge-perf::pN` — cargo paths need underscores:
  `ge_perf::pN`, mirror the ge-dot_parser tomls).
- Scenario wiring pattern: `crates/gently-core/tests/scenarios.rs`
  (each module with `#[path = "scenarios/ge_X.rs"]`), contract tomls
  under `.espectacular/ge-perf/`.
- Test paths note (cargo does NOT include the file name in test paths):
  module must be named `ge_perf`; p1–p5 tests named p1()…p5().

## What to build

1. **`tests/scenarios/ge_perf.rs`** — one `#[test]` per property id:
   - **p1 (c1):** build a 20-node/30-edge graph, run
     parse→layout→ascii-render, assert wall time < 50 ms. CRITICAL:
     budget checks like this are timing-flaky under debug builds and
     CI load — measure with a headroom multiplier: run the pipeline
     once, assert < 50 ms × headroom (pick a documented multiplier,
     e.g. 20× for debug builds = 1 s, still trivially met given the
     release number is ~1 ms), and record the multiplier in a
     doc-comment. Never assert sub-50ms raw in debug.
   - **p2 (c2):** 1000-node/2000-edge graph rendered in every
     supported format (ascii, boxart, html, txt, graphviz), assert
     wall < 2 s × same headroom and peak memory accounted structurally
     (e.g. via allocator counters if cheap, else assert the measured
     profile from k4u: the pipeline holds one model + grid + buffer —
     a structural assertion on data size: total bytes of outputs is
     bounded by O(cells), not an RSS measurement, in unit tests).
   - **p3 (c3):** structural: stage outputs do not share mutable
     caches — assert that rendering twice produces identical results
     and that peak buffers are one grid + one buffer (code-level
     structural assertion; keep it honest and documented).
   - **p4 (c4):** the budget gate itself — a helper (lib function or
     test-local) that checks a measured duration against a budget and
     produces the typed diagnostic naming stage/budget/measured value;
     test the diagnostic format + non-zero exit semantics. If the
     gate helper belongs in gently-core (pub fn for the CLI/just
     recipe), put it in a small `src/perf.rs` module.
   - **p5 (c5):** sweep sizes (e.g. 100/500/1000/2000 nodes), assert
     measured times scale near-linearly within a documented bound
     (e.g. t(4n) <= 6×t(n)); report — no oracle comparison.
2. **justfile `perf-check`:** wire it to run the perf scenario tests
   (e.g. `cargo test -p gently-core --test scenarios ge_perf --release
   -- --nocapture` or a release build + the budget helper) and exit
   non-zero on budget violation per c4. Keep it CI-tolerant (CI runs
   debug — use the headroom multiplier).
3. **Contract wiring:** create `.espectacular` scenario registration
   `mod ge_perf;` in `tests/scenarios.rs` (with `#[path]` attr!) and
   fix `.espectacular/ge-perf/p1–p5.toml` flags to
   `ge_perf::pN`. After your change, the no-tests-ran backlog must
   shrink by the five ge-perf contracts.

## TDD sequence (red → green → refactor commits)

1. **RED:** scenario tests p1–p5 honestly failing (module registered,
   tomls fixed — the flags fix makes `ah check --run-tests` run
   them and fail).
2. **GREEN:** implement until tests pass; commit.
3. **REFACTOR:** justfile recipe + doc-comment tidy — separate commit.

## Hard scope guard

- Allowed: `crates/gently-core/tests/scenarios.rs`,
  `crates/gently-core/tests/scenarios/ge_perf.rs` (new),
  `crates/gently-core/src/perf.rs` (new, small),
  `crates/gently-core/src/lib.rs` (export only),
  `justfile` (perf-check recipe only),
  `.espectacular/ge-perf/p1.toml`…`p5.toml` (flags).
- Never edit: `specs/*.md`, `openspec/**`, other `src/**` modules,
  `crates/gently-cli/**`, `.wai/**`, CI workflows.
- Do NOT push, do NOT close the bead, do NOT run `wai close` — the
  orchestrator owns ship.
- Commit subjects end with `(gently-4ln)`. File headers required.

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
spk lint specs/
ah check
ah check --run-tests   # ge-perf p1–p5 must now execute
just gates
just perf-check        # must exit 0 on this machine
```

## Report format (end with this)

## Report

**Commits** (hash + one line each)

**Headroom multiplier chosen + rationale**

**Gates run** (the seven lines)

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, ge-perf.md, scenarios.rs
  pattern, one existing scenario file. Max 8 tool calls.
- **Minutes 5–10:** RED tests + toml flags.
- **Minutes 10–22:** GREEN + perf.rs helper.
- **Minutes 22–28:** justfile recipe + gates (incl. just perf-check).
- **Minutes 28–30:** report.
Never re-read a file you have already read. A committed RED beats an
uncommitted almost.