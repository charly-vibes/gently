# Subagent brief: gently-0of — restore oracle-verify: layout+ascii serve the recorded corpus

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-0of` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real gates —
never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-0of`
- THE FINDING: `just oracle-verify` fails at HEAD. gently-3hv grew
  `tests/fixtures/graph-easy/` to 8 inputs with ascii companions recorded
  from the pinned oracle (Graph::Easy v0.69 @ ededa3d7), but the tracer-era
  layout (`crates/gently-core/src/layout.rs`, gently-2po.8) and ascii
  renderer (`crates/gently-core/src/render/ascii.rs`, gently-2po.9) serve
  only the two-node tracer shape. Example failure (mixed_isolated):
  gently emits `| a | --> | b | -->  -->  --> | c |`; oracle expects a
  3-box chain plus an isolated d box below.
- Read `specs/ge-layout.md` (c1 determinism, c2 no-overlap/grid-containment)
  and `specs/ge-ascii_render.md` (byte-compatible with Graph::Easy classic
  ascii). The full property contracts for those specs belong to
  gently-4nx / gently-0cq — you implement the corpus-serving slice ONLY;
  do not chase their pN contracts.
- The recorded companions are AUTHORITATIVE expected outputs (recorded from
  the real pinned oracle, pin headers present). Your job is to make gently
  produce exactly those bytes for all 8 fixtures.

## What to build

Extend `gently_core::layout` + `gently_core::render::ascii` so every
fixture renders byte-identically through the CLI:

- `chain` (a→b→c): three boxes on one row.
- `mixed_isolated` (chain + isolated d): chain row; isolated node rendered
  as its own box below (oracle: d box after the chain block, left-aligned).
- `parallel` (a→b twice): second edge bends around (oracle shows the
  elbow: `+---------+` / `|         v` above the boxes).
- `selfloop` (a→a): oracle shows the loop elbow above the box with `v`
  arrowhead.
- `txt-diamond` (a→b, a→c, b→d, c→d): ranks 0/1/2; c sits below rank 1;
  vertical edge segments + arrowheads exactly as the oracle ascii shows.
- `txt-isolated`, `txt-shared-target`, `tracer`: already-passing shapes
  must stay byte-identical.

Upstream reference for layout/ascii mechanics: Graph::Easy 0.69
`lib/Graph/Easy/Layout.pm` (+ rank/order/position/path submodules) and
`lib/Graph/Easy/As_ascii.pm`; pinned source fetchable:
`curl -fsSLO https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-0.69.tar.gz`.
Port the minimal deterministic subset that serves these shapes — do NOT
port the full engine. ge.layout.c1 (pure function of model+options) and
c2 (no overlapping node cells, nodes inside grid) must hold.

**Test shape:**
- Scenario tests under `mod tb` (filter-compatible paths, e.g.
  `tb::corpus::chain_renders_oracle_ascii`): for each fixture, parse →
  layout → ascii render and assert byte-identity with the recorded
  companion (strip the pin header line).
- Keep all existing tests green (49 across the workspace).
- RED→GREEN→REFACTOR: red tests first (current renderer output vs
  oracle), then extend, then tidy in a separate commit.

## Out of scope (do NOT do)

- Do NOT modify fixtures or re-record companions (corpus is deliberate;
  it stays exactly as recorded).
- Do NOT chase gently-4nx/gently-0cq pN contracts or touch their
  `.espectacular/` files.
- No parser changes beyond what the corpus inputs already exercise
  (selfloop/parallel/diamond parse fine today).
- No txt-renderer changes (txt companions pass).

## Hard scope guard

- Allowed files: `crates/gently-core/src/**`, `crates/gently-core/tests/scenarios.rs`
- Never edit: `openspec/`, `specs/`, `.espectacular/**`, `tests/fixtures/**`,
  `.wai/**`, `justfile`, `lefthook.yml`, `.github/`, `crates/gently-cli/**`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before ANY commit; stage only
  files YOU authored; attribute only what the diff contains.
- **Pre-commit gates run automatically** (lefthook).
- **Do NOT push**; **do NOT `bd close` / `wai close`** — orchestrator owns.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: `-f`/`-rf`/`-y` flags.
- Commits reference the ticket: `(gently-0of)`.

## Gates you must leave green

- `just oracle-verify` — GREEN, all 8 fixtures byte-identical (the point
  of this ticket)
- `cargo test --workspace` — all existing tests + new corpus tests
- `cargo clippy --workspace --all-targets -- -D warnings`
- `ah check` — 0 structural findings

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `just oracle-verify` → <result>
- `cargo test --workspace` → <result>
- `cargo clippy ...` → <result>
- `ah check` → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">
