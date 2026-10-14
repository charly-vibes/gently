# Pre-registered (b)-checklist registry (decision.trace c6, design D5)

The registry governing adjudication of every reported cut element,
committed per `specs/decision-trace.md` c6 (design D5; instantiated by the
E4 dry run, `.wai/projects/decision-trace/designs/2026-10-14-decision-e4-adjudication-dry-run-gently-58m.md`,
precedent gently-58m). This file is the checklist + the record template;
the per-case verdicts live in wai decision records under
`.wai/projects/decision-trace/designs/` and are enforced by
`scenarios/decision_trace.rs::p6`.

## Class taxonomy

- **(a)** gently diverges → fix gently (the default).
- **(b)** oracle defect → ALL FIVE checklist items must hold; any failure
  drops the element to (a) or (c).
- **(c)** underdetermined → human decides.

## The five checklist items (all must HOLD for a (b) classification)

| # | item |
|---|------|
| 1 | deterministic probe reproduction under ≥2 hash seeds (tier-2 excluded per [[ge.oracle.c7]]) |
| 2 | no existing spec constraint contradicts gently's behavior |
| 3 | the upstream defect is durably recorded and links the probe (in-repo bead is the user-approved form; no upstream posting) |
| 4 | gently's behavior is renderable and deterministic under its own invariants (per [[ge.layout.c1]]) |
| 5 | the delta is not covered by the [[ge.oracle.c7]] scope-off classes |

## Adjudication record template (a wai decision record per adjudication)

```markdown
# Decision: E<N> adjudication — <case-id> classifies <(a)|(b)|(c)>

**Date:** … **Case:** … **Meter (pre-registered, D5):** … **Kill
criterion:** …

## Pre-registered (b)-checklist — per-item verdicts

| # | item | verdict | evidence |
|---|------|---------|----------|
| 1..5 | <registry item> | **HOLDS** / **FAILS** | … |

## Final class: <(a)|(b)|(c)>

## Consequences (D5/D7)
```
