# Subagent brief: gently-2po.10 — tb.oracle: first differential fixture proves the tracer

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-2po.10` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real gates —
never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-2po.10`; read `specs/ge-oracle.md` fully (c1–c6; the slice
  binds c1 pin, c2 recorded outputs, c3 byte-verify, c4 typed drift error,
  c5 record discipline) and skim `specs/ge-txt_render.md` c4.
- Existing pipeline: `gently` CLI (gently-2po.9) reads tracer text → prints
  oracle-identical ascii. `just oracle-record` / `just oracle-verify` in the
  justfile currently fail honestly ("runner spec'd") — you make them real.

## What to build

**1. Fixture corpus seed — `tests/fixtures/graph-easy/`:**
- `tracer.txt` — the input: `[ a ] --> [ b ]`
- Recorded companions from the pinned oracle: `tracer.txt.expected` (as_txt
  → `[ a ] --> [ b ]\n`) and `tracer.ascii.expected` (as_ascii → the 3-line
  box drawing). Record them from the REAL pinned oracle, not from gently.
- Each companion carries a pin header naming Graph::Easy v0.69 @
  ededa3d787ad89ac532c578c06390e8a7b270499 (ge.oracle.c1). Pick a header
  format (e.g. leading `# oracle: ...` comment lines) and document it in a
  `tests/fixtures/graph-easy/README.md`.

**Oracle environment gotcha (already hit by the orchestrator):** system perl
has Graph::Easy **0.76** — the pin check WILL fail typed. To record with the
pinned revision, fetch it isolated (ephemeral, fine — recording is rare and
deliberate):

```bash
mkdir -p /var/tmp/ge069 && cd /var/tmp/ge069
curl -fsSLO https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-0.69.tar.gz
tar xzf Graph-Easy-0.69.tar.gz
PERL5LIB=/var/tmp/ge069/Graph-Easy-0.69/lib perl -MGraph::Easy -e 'print Graph::Easy->VERSION'  # 0.69
```

**2. Oracle runner — `tools/oracle.pl` (or similar), wired into justfile:**
- `just oracle-record`: for each `tests/fixtures/graph-easy/*.txt` input
  without a companion (or with a stale pin header), run the pinned oracle
  (`as_txt` + `as_ascii`) and write companions with pin headers in one pass
  (ge.oracle.c5). Refuse to overwrite a companion whose pin header already
  matches the current pin (re-recording without a pin change is rejected).
- `just oracle-verify`: for every input, run the **gently binary**
  (`cargo run -q -p gently-cli -- --format txt|ascii < input` or the built
  binary) and compare byte-identically to the recorded companions; any
  mismatch fails naming the input, the format, and the first differing line
  (ge.oracle.c3).
- Drift/missing environment → typed error naming what is missing and the
  remediation (ge.oracle.c4): perl absent, Graph::Easy version ≠ 0.69
  (remediation names `cpanm Graph::Easy==0.69` or the isolated-PERL5LIB
  recipe), stale pin header.
- Keep the existing justfile recipe structure; replace the honest-failure
  stubs with real runners. `--human`-style error lines are fine.

**3. Differential proof as a cargo test (CI-runnable, no perl needed):**
- `scenarios::tb::oracle` in `crates/gently-core/tests/scenarios.rs` (or a
  `crates/gently-cli/tests/` integration test — your call, keep filters
  `scenarios::tb::oracle::...` compatible): read `tracer.txt` + companions,
  run the pipeline (parse → render for txt; full CLI for ascii), assert
  byte-identity against the committed expected bytes. This pins the oracle
  forever in-repo; the just recipes re-verify against live perl locally.

**Exact test shape** (minimum):

```rust
#[test]
fn tracer_fixture_matches_recorded_oracle_txt() { /* read fixture + expected, compare */ }
#[test]
fn tracer_fixture_matches_recorded_oracle_ascii() { /* via the pipeline/CLI */ }
#[test]
fn stale_pin_header_is_rejected() { /* recorder/verifier unit path */ }
```

**RED→GREEN→REFACTOR (mandatory order):**
1. RED: tests first (fixture bytes may already exist — the tests target the
   runner/verify logic; record the honest failure).
2. GREEN: implement runner + recipes until green.
3. REFACTOR: only if genuinely untidy; separate commit.

## Out of scope (do NOT do)

- No corpus-wide coverage matrix (ge.oracle.c6), no oracle doctor, no
  format matrix beyond txt+ascii — gently-q1x deepens.
- No changes to the CLI's behavior; no new render formats.
- No openspec change; no edits to `.github/`, `lefthook.yml`.

## Hard scope guard

- Allowed files: `justfile` (the two oracle recipes only),
  `tools/**`, `tests/fixtures/graph-easy/**`,
  `crates/gently-core/tests/scenarios.rs`, `crates/gently-cli/tests/**`
- Never edit: `openspec/`, `specs/`, `.espectacular/`, `.wai/**`,
  `lefthook.yml`, `.github/`, `crates/*/src/**` unless a test-support
  accessor is strictly needed (prefer not to touch src at all).

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before ANY commit; stage only
  files YOU authored; attribute only what the diff contains.
- **Pre-commit gates run automatically** (lefthook) — dirty foreign files
  fail them.
- **Do NOT push**; **do NOT `bd close` / `wai close`** — orchestrator owns.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: `-f`/`-rf`/`-y` flags.
- Commits reference the ticket: `(gently-2po.10)`.

## Gates you must leave green

- `cargo test --workspace` (all prior tests stay green)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `ah check` (0 findings)
- `just oracle-verify` green locally (with the pinned PERL5LIB or system
  0.69 — document which in the report)

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo test --workspace` → <result>
- `cargo clippy ...` → <result>
- `ah check` → <result>
- `just oracle-verify` → <result + how the pin was satisfied>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
