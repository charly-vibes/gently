# Subagent brief: gently-q1x — implement ge.oracle (scenario tests p1–p6 + contract filters; closes dk2 + mki)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core`, edition 2021). An orchestrator has
claimed ticket `gently-q1x` and is hands-off: you own the implementation
end-to-end. This run also closes `gently-dk2` (stale ge-oracle toml
filters) and `gently-mki` (missing ge-oracle scenario tests) — after
your gates you may close both with `bd close gently-dk2` /
`bd close gently-mki` (NOT gently-q1x itself — the orchestrator owns
ship for the parent ticket).

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-q1x` (and `bd show gently-dk2`, `bd show gently-mki`)
- Read `specs/ge-oracle.md` IN FULL — c1–c7 + properties p1–p7.
  IMPORTANT: ikm (3d78481) amended c1/c3/c4/c7 (perl version, pinned
  seed 0, PERL5LIB checkout contract); liz (6c9ece9) split c4's
  remediation. The current rows are the contract.
- Existing scenario arm: `crates/gently-core/tests/scenarios/ge_oracle.rs`
  already implements p7 (tier-1 admission manifest check). Your work:
  **p1–p6** in the same module + fix the toml flags.
- `.espectacular/ge-oracle/p1.toml`…`p6.toml` flags are stale
  (`scenarios::ge_oracle::pN` — should be `ge_oracle::pN`; p7.toml is
  already correct — mirror it). That staleness is the dk2 ticket.
- Tooling under contract: `tools/oracle.pl` (record/verify/claims
  modes — read its arg parsing + error messages), justfile
  `oracle-record`/`oracle-verify` recipes, fixture corpus
  `tests/fixtures/graph-easy/` (8 inputs + companions + pin headers —
  read README.md for the layout).
- The oracle toolchain (perl + pinned checkout at
  `/var/tmp/ge0.69/Graph-Easy-0.69/lib`) EXISTS on this machine but
  NOT in CI — every test must degrade honestly: when the toolchain is
  absent, the oracle-executing tests (p3/p4/p5 oracle-run arms) SKIP
  (print a reason + early return) and the static arms (p1/p2/p6) still
  run. Never let CI-relevant tests hang on missing perl.

## What to build

Extend `tests/scenarios/ge_oracle.rs` (same module, alongside p7):

- **p1 (c1):** static — every fixture companion's pin header names
  version `0.69` + commit `ededa3d7…`; recording metadata lines name
  perl version + seed; no foreign pin.
- **p2 (c2):** static — every input has exactly one recorded expected
  output per supported format; no orphan companions.
- **p3 (c3):** oracle-executing — if the pinned checkout + perl are
  available, re-render every fixture input via the oracle
  (`tools/oracle.pl verify` or the justfile recipe path) and assert
  byte-identical (report input/format/first-differing-line on
  mismatch — the tool already does; assert its exit status). Skip
  with reason otherwise. KEEP IT FAST — 8 fixtures is fine.
- **p4 (c4):** oracle-executing — in a tempdir (never the repo tree),
  induce each drift case (perl missing is not inducible — fake PATH
  without perl; version drift via a fake checkout with wrong version;
  missing checkout path; missing PERL_HASH_SEED on a record run; stale
  pin header) and assert each yields a typed error naming the missing
  piece + remediation. Skip if perl absent.
- **p5 (c5):** oracle-executing — record a NEW tiny input into a
  tempdir corpus (matching pins → recorded + header updated), then
  attempt re-record with a stale/absent pin → rejected. Skip if perl
  absent.
- **p6 (c6):** static — coverage matrix over the recorded corpus:
  every edge style, border style, node shape, group form, flow
  direction, label placement class appears in ≥1 fixture (the classes
  as the spec row lists them; derive the present classes from fixture
  inputs/attributes).

Fix `.espectacular/ge-oracle/p1–p6.toml` flags to `ge_oracle::pN`.

## TDD sequence (red → green → refactor commits)

1. **RED:** add the p1–p6 tests + toml flags — honestly failing
   (static arms fail on real gaps; oracle arms skip if toolchain
   missing — if everything skips on this machine, note it; the
   toolchain IS present here so they should run).
2. **GREEN:** make them pass (fix what the tests expose in
   tools/oracle.pl / justfile / fixtures ONLY as the tests dictate —
   no spec edits).
3. **REFACTOR:** shared helpers (fixture-path, toolchain-detection)
   tidied; separate commit.

## Hard scope guard

- Allowed: `crates/gently-core/tests/scenarios/ge_oracle.rs`,
  `tools/oracle.pl` (only as the tests dictate),
  `justfile` (oracle recipes only, only as the tests dictate),
  `tests/fixtures/graph-easy/**` (only to fix a pin header the tests
  expose as wrong — never re-render bytes),
  `.espectacular/ge-oracle/p1.toml`…`p7.toml` (flags).
- Never edit: `specs/*.md`, `openspec/**`, `crates/gently-core/src/**`,
  other scenario files, `.wai/**`, CI workflows, `deny.toml`.
- Do NOT push. Close gently-dk2 and gently-mki ONLY after all gates
  are green; do NOT close gently-q1x; do NOT run `wai close`.
- Commit subjects end with `(gently-q1x)`. File headers required.

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
spk lint specs/
ah check
ah check --run-tests   # ge-oracle p1–p7 must ALL execute (no-tests-ran → 0)
just gates
```

## Report format (end with this)

## Report

**Commits** (hash + one line each)

**Test arms** (per p1–p6: static or oracle-executing; what CI does)

**Gates run** (the six lines; state no-tests-ran count)

**Beads closed** (dk2/mki or not)

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, ge-oracle.md, tools/oracle.pl
  arg surface, existing ge_oracle.rs p7. Max 8 tool calls.
- **Minutes 5–12:** RED p1–p6 + toml flags.
- **Minutes 12–24:** GREEN (static arms first, oracle arms second).
- **Minutes 24–28:** gates; close dk2/mki if green.
- **Minutes 28–30:** report.
Never re-read a file you have already read. A committed RED beats an
uncommitted almost.