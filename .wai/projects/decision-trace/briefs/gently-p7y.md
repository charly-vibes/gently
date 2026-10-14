# Subagent brief: gently-p7y — trace emission (D1 promotion + p1 contract)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace, edition 2021). An orchestrator has claimed ticket
`gently-p7y` and is hands-off: you own the work end-to-end (RED, GREEN,
refactor, commits). Do NOT push and do NOT `bd close` — the orchestrator
owns ship.

## TIME BUDGET (binding): 30 minutes.

## Orientation (max 10 tool calls)

- `export WAI_PROJECT=decision-trace`
- `bd show gently-p7y`
- `specs/decision-trace.md` — c1 is YOUR row (versioned schema, per-stage
  vocabulary, governing spec id per event); p1 is the contract you execute
  (`.espectacular/decision-trace/p1.toml`, cargo flag `decision_trace::p1`).
- `crates/gently-core/src/trace.rs` — the env-gated E1 prototype you promote
  (stages `layout.place` + `render.route`, wired in `layout.rs:227-250`).
- The design doc D1 section:
  `.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md`.
- Contract-test wiring precedent: `tests/scenarios/` + the registration
  pattern in `tests/scenarios.rs` (see how `ge_oracle` or `ge_layout`
  modules are wired; `ah check --run-tests` executes them via the tomls).

## Desired outcome

`trace.rs` is the real D1 layer, not the deletable prototype:
1. A `version` field (start at 1) and an explicit stage-vocabulary set
   covering c1's stages (parse, graph model, rank/col, bands, spans,
   route classification, bends, glyphs). Emission points must exist for at
   least the two stages the E1 prototype already materializes
   (`layout.place`, `render.route`); add emission for other stages ONLY
   where the decision is already materialized in reachable code (e.g.
   parse events in the text parser's statement loop) — do not refactor
   distant code to create emission points; leave those stages declared but
   unemitted for follow-up beads, recorded in a doc-comment.
2. Every trace line carries: `version`, stage, event, and the governing
   spec id (`decision.trace`).
3. Env gating UNCHANGED: `GENTLY_TRACE=<stage>`, output to
   `GENTLY_TRACE_FILE` (append) or stderr. c1 says nothing else.
4. Contract `decision_trace::p1` executes: `tests/scenarios/decision_trace.rs`
   (registered in `tests/scenarios.rs`) asserting the p1 predicate from
   specs/decision-trace.md (events emitted across the pipeline carry the
   version + governing spec id; trace-off = silent + byte-identical corpus).

## RED → GREEN → REFACTOR

- RED commit: the scenario file with p1 assertions that FAIL against the
  current prototype (no version field, no spec id). Commit message:
  `test(decision-trace): p1 contract RED — trace events lack version + spec id (gently-p7y)`.
- GREEN commit: the trace.rs promotion making p1 pass.
  `feat(decision-trace): promote trace.rs to the D1 layer — versioned stage vocabulary + governing spec ids (gently-p7y)`.
- REFACTOR commit only if needed (separate commit).
- Regression property (in the scenario file): with `GENTLY_TRACE` unset,
  rendering a small fixture produces no trace output (no stderr capture
  needed — assert the API-level gate: `trace::enabled` false and emit() is
  a no-op), and the existing pinned corpus tests already gate bytes.

## Gates (run before reporting)

- `cargo test` green (expect 218+ existing + your new p1 scenario tests;
  count the delta).
- `cargo clippy --all-targets` clean — NOTE: `single_match` is DENIED by
  pre-push, use `if let` instead of `match x { A => .., _ => .. }`.
- `ah check --run-tests`: 78 passed (77 + p1), no-tests-ran drops 6 → 5.
- `just gates` green. If `spk compile` regenerates specodelic artifacts,
  `rustfmt --edition 2021 specodelic/*.rs` (specodelic#15) — but you should
  NOT be amending the spec in this slice; stage any accidental regen
  separately and mention it in your report.

## Out of scope

- `tests/repro/probes.pl`, `tests/repro/trace/`, dumps (h1b/9nn own those).
- p2–p6 contracts (stay in the no-tests-ran backlog).
- Spec amendments, docs rows, push, `bd close`.

## Follow-up threshold

If the D1 promotion would require touching >6 files or restructuring
layout.rs, STOP at the minimal promotion (version + spec id + declared
vocabulary) and record the rest as follow-up in your report.

## Report back (≤200 tokens)

- commit shas, files touched, contract status (p1 executing, backlog 5),
  gates numbers, deviations.
