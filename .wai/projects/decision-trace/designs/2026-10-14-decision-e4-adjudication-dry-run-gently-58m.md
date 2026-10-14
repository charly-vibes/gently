# Decision: E4 adjudication dry run — gently-58m classifies (b), oracle defect (gently-btf)

**Date:** 2026-10-14
**Case:** gently-58m — oracle `as_txt` omits Berlin entirely for
`( [ Bonn ] ) -> [ Berlin ]` (the POD anonymous-group example); gently
lists Berlin as a plain node.
**Meter (pre-registered, D5):** wai decision record exists with per-item
verdicts + final class.
**Kill criterion:** checklist stalls on missing evidence classes ⇒ the
registry ships later, not with the trace layer.

## Pre-registered (b)-checklist — per-item verdicts

| # | item | verdict | evidence |
|---|---|---|---|
| 1 | claim probe reproduces upstream behavior deterministically under ≥2 hash seeds | **HOLDS** | re-ran live under `PERL_HASH_SEED=0` and `=1`: identical output — `as_txt` prints only `( [ Bonn ] )`, Berlin absent; recorded in `tests/repro/claims/pod-anon-group.observed`. Deterministic ⇒ not tier-2. |
| 2 | no existing spec constraint contradicts gently's behavior | **HOLDS** | `ge.txt_render.c3` already places group-endpoint edges OUTSIDE the round-trip contract (the gently model cannot represent them; upstream serializes them as plain node edges — `group-edge-graphviz.observed`). No spec mandates dropping a group-only-connected node from the declaration pass; p12 pins Berlin as an unlinked plain node. |
| 3 | upstream issue filed linking the probe | **HOLDS, modified form** | user decision (2026-10-14): do NOT post to `shlomif/perl-graph-easy`; the defect is recorded as beads ticket **gently-278** ("upstream defect (recorded, not filed)") carrying the repro, seeds, observed outputs, layouter warning, and expected behavior. The checklist's intent — a durable, linked record of the upstream defect naming the probe — is satisfied in-repo. |
| 4 | gently's behavior renderable and deterministic under its own invariants (ge.layout c1) | **HOLDS** | `cargo run -p gently-cli` renders ascii (dashed group border + Bonn + Berlin boxes) and txt (group block + `[ Berlin ]`); output md5-stable across runs. |
| 5 | delta not covered by the ge-oracle c7 scope-off classes | **HOLDS** | the input is hash-stable (seed sweep 0/1 identical), within envelope, and the delta is not a round-trip gap — none of c7's scope-off classes apply. |

## Final class: (b) — oracle defect

All five items hold (item 3 in its user-approved in-repo form). The
oracle's `as_txt` is self-inconsistent with its own `as_ascii`, and the
layouter emits `Warning: Layouter could only place 2 nodes/0 edges out
of 2/1 - giving up` — classic upstream defect. gently's behavior (group
block + `[ Berlin ]` listed; dashed border in ascii) stands.

## Consequences (D5/D7)

- **Kill criterion NOT met** — the checklist did not stall; class (b) was
  reachable with evidence in hand in one dry run.
- The **"deliberate improvements" registry ships WITH the trace layer**
  (task 6.2, if-branch): registry = pre-registered checklist + the
  adjudication-record template instantiated by this record.
- gently-58m's residual is adjudicated (b): gently's as_txt behavior is
  the intended one; the divergence is promoted to a spec claim question
  (spec sketch amendment in review, per D5's consequence paragraph),
  NOT a byte-compat fix. Byte-compat stays the CI gate (D6).
- D7 matrix rows E4-dependent (option B adjudication support) now carry
  a dry-run data point: checklist complete, zero missing evidence
  classes, one session.
