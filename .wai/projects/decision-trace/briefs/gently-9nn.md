# Subagent brief: gently-9nn — trace-diff (minimal-cut driver + EXCL-001 property)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace, edition 2021). An orchestrator has claimed ticket
`gently-9nn` and is hands-off: you own the work end-to-end. Do NOT push
and do NOT `bd close` — the orchestrator owns ship.

## TIME BUDGET (binding): 30 minutes.

## Orientation (max 10 tool calls)

- `export WAI_PROJECT=decision-trace`
- `bd show gently-9nn`
- `tests/repro/trace/e1.sh` — the E1 prototype driver you promote (it
  builds throwaway worktrees at fix^ and fix, injects the OLD trace
  prototype, diffs traces). NOTE: the current `trace.rs` is now the real
  D1 layer (gently-p7y: versioned, `dt v1 stage=… spec=decision.trace`
  line format) — e1.sh's prototype-injection hack is OBSOLETE. The new
  trace-diff must run against the library trace as committed at any rev.
- `specs/decision-trace.md` c3 + p3 — the contract you execute:
  divergence is a set over the decision dependency poset; trace-diff
  reports the MINIMAL ANTICHAIN (cut set), never a unique "first";
  byte-gate: refuses on byte-identical fixtures (decision-only diffs are
  informational); p3's cargo flag is `decision_trace::p3`
  (`.espectacular/decision-trace/p3.toml`).
- Design doc D3/D6 sections:
  `.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md`.
- `tests/repro/dumps/` — 27 committed oracle dumps (E2 five + corpus
  extension, gently-h1b), each with pin header + seed field +
  stage-observability marks.
- EXCL-001 (design doc): every byte-divergent fixture in the corpus
  yields a nonempty trace diff against its dump.

## Desired outcome

1. **trace-diff implementation** (5.1): a real driver (Rust scenario or
   bash+perl runner under `tests/repro/trace/` — your call, but it must
   be invocable from the scenario test) that:
   - runs gently on a fixture at two revisions (or gently vs the oracle
     dump for the E2 cases), emits traces for the pre-registered stage,
   - diffs them over the stage vocabulary and reports the minimal
     antichain of differing events (a first-differing linear guess is
     PRESENTATION; the report must contain the full antichain — for the
     E1 cases the antichain is exactly the pre-registered cut),
   - BYTE-GATE (5.2): refuses (typed error, non-zero exit) when the two
     fixtures' rendered bytes are identical and the traces differ
     ("decision-only diff — informational, run with --informational to
     see it"); byte equality remains the CI gate.
2. **EXCL-001 corpus property test** (5.3): a scenario test iterating
   the byte-divergent corpus (the known-hard-cases rows / the
   `so-flowchart-size` class — the fixtures whose pinned bytes differ
   from gently) asserting a nonempty trace diff against the committed
   dump for each. Scope it to the fixture classes with committed dumps
   (the 27) + at least one byte-divergent fixture class; do not try to
   hook every corpus row.
3. **Contract `decision_trace::p3` executes** in
   `tests/scenarios/decision_trace.rs` (extend the existing module — do
   NOT create a second file for it).

## RED → GREEN → REFACTOR

- RED commit: p3 scenario assertions failing (no trace-diff exists).
  `test(decision-trace): p3 contract RED — trace-diff missing (gently-9nn)`.
- GREEN commit: the driver + EXCL-001 property.
  `feat(decision-trace): trace-diff minimal-antichain driver + EXCL-001 corpus property (gently-9nn)`.
- Keep e1.sh in place as historical record (it documents E1's method);
  add a header note that it is superseded by the real driver.

## Gates

- `ah check --run-tests`: 79 passed (78 + p3), no-tests-ran 5 → 4.
- `cargo test` + `cargo clippy --all-targets` clean (`single_match` is
  DENIED — use `if let`).
- `just gates` green; rustfmt any regenerated specodelic artifacts
  (specodelic#15).
- The trace-diff run must complete in seconds (E1 was ~8 s; budget < 60 s
  for the EXCL-001 property).

## Out of scope

- p2/p4/p5/p6 contracts. probes.pl changes. Spec amendments (unless p3's
  predicate text proves unimplementable — then STOP and report, do not
  weaken it silently). Push. bd close.

## Report back (≤200 tokens)

- commit shas, driver location + invocation, antichain output for the awd
  case (does it match the E1 pre-registered cut?), byte-gate behavior,
  EXCL-001 scope + pass status, gates numbers, deviations.
