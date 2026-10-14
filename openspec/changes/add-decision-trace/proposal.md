# Proposal: add-decision-trace

## Why

Debugging render/layout divergences against the pinned oracle is a
per-session scratch-probe loop: each divergence hunt re-decodes the
relevant upstream stage by hand (486 probe invocations are already
documented in `tests/repro/probes.pl`; the lost-probes incident is
recorded there too). Two pre-registered experiments proved a decision
trace layer fixes this class of cost:

- **E1 (gently-asc, PASS):** retrospective localization on two
  already-fixed divergences (awd, 4ga) reported minimal cuts that belong
  to the pre-registered stage for each case, at ~8 s driver wall clock
  vs multi-session byte-diff hunts.
- **E2 (gently-8je, PASS):** oracle `_prepare_layout` dumps for the five
  bead fixtures total 5130 B (kill threshold 500 KB) with one sub
  wrapped (~1 s run) — committed dumps are cheap for this fixture class.

The D7 decision rule is satisfied; the go/no-go decision record is
`.wai/projects/decision-trace/designs/2026-10-14-decision-d7-go-no-go-openspec-proposal.md`.

## What Changes

Add a **decision-trace** capability (new spec, new tooling, no behavior
change to gently's parse/render output):

1. **Decision trace schema (D1):** a versioned schema with one
   sub-vocabulary per pipeline stage (parse, graph model, rank/col,
   bands, spans, route classification, bends, glyphs); every event
   carries its governing spec id. Emission is env-gated
   (`GENTLY_TRACE`/`GENTLY_TRACE_FILE`); with trace off the corpus stays
   byte-identical (E1-verified property).
2. **Oracle dump mode (D4):** `tests/repro/probes.pl` gains a `dump`
   mode emitting the trace schema from the Perl side (wrapping
   `_prepare_layout` first, per the E2 pilot); dumps carry the oracle
   pin header, the hash seed as an explicit field, and per-stage
   observability marks so partially-observed traces are never reported
   as divergences. Committed dumps for hash-stable (tier-1) fixtures;
   tier-2 fixtures are excluded.
3. **Normalization table (D2):** the oracle→trace normalization is an
   explicit committed, tested table — not prose — with the pairwise
   faithfulness criterion restated from the design doc and gated on E3
   computability.
4. **trace-diff (D3 + D6):** divergence is a set over the decision
   dependency poset; the diff reports the minimal antichain (cut set),
   never a linear "first". trace-diff runs only on byte-divergent
   fixtures (or downgrades decision-only diffs to informational);
   byte equality remains the CI gate.
5. **Adjudication registry (D5):** the pre-registered (b)-checklist
   (deterministic probe reproduction, no spec contradiction, upstream
   issue filed, gently determinism, no c7 scope-off) with each
   adjudication recorded as a wai decision record. **Timing gated on
   E4 (gently-btf):** ships with the trace layer only if the dry run
   reaches class (b) with evidence already in hand.

## Impact

- **Affected specs:** new capability `decision-trace`; no existing
  capability's requirements change. `ge-oracle` c7 scope-offs are
  referenced, not amended.
- **Affected code:** `tests/repro/probes.pl` (dump mode extension of the
  E2 pilot), `tests/repro/trace/` (driver, already prototyped in E1),
  `crates/gently-core/src/trace.rs` (harden the E1 prototype), plus the
  new spec corpus file + deployed mirror + specodelic artifacts for the
  capability (per the spec-first convention).
- **Not in scope:** changing any renderer/layout output, re-pinning the
  oracle revision, per-fixture committed dumps for tier-2 classes.
