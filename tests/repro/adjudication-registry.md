# Adjudication registry — the pre-registered (b)-checklist (D5)

> Decision-trace D5: the spec sketch is the arbiter; the pinned oracle is
> one model of it — a sufficient heuristic for byte-compat, never the
> definition of correct. When trace-diff reports a cut, each cut element
> is classified **(a)** gently diverges → fix gently (default), **(b)**
> oracle defect → gently is right(er) — gated on the checklist below —
> or **(c)** underdetermined → human decides.
>
> Shipped WITH the trace layer per E4 (gently-btf): the dry run on
> gently-58m reached class (b) with all five items holding in one
> session — the pre-registered kill criterion ("checklist stalls on
> missing evidence classes") did not trigger.
>
> Byte-compat stays the CI gate (D6); adjudicated improvements are
> promoted into specs (gently's behavior becomes a spec claim), so
> "better" is decided once, in review, with evidence — not re-litigated
> per session.

## The (b)-checklist (verbatim from the design doc — do not edit retroactively)

All five must hold for class (b); any failure drops the element to
(a) or (c).

1. **Deterministic probe reproduction.** A claim probe reproduces the
   upstream behavior deterministically — re-run under ≥2 hash seeds;
   nondeterministic ⇒ tier-2, not (b).
2. **No spec contradiction.** No existing spec constraint contradicts
   gently's chosen behavior — if one does, the conflict is a spec
   amendment question, not a (b).
3. **Upstream defect recorded and linked to the probe.** Filed on the
   upstream tracker when authorized; otherwise (user decision,
   2026-10-14, gently-58m) recorded as a beads ticket in this repo
   (`upstream-defect` label) carrying the repro, seeds, observed
   outputs, and expected behavior, naming the claim probe.
4. **gently renders, deterministically.** gently's behavior is
   renderable and deterministic under its own invariants
   (ge.layout c1 model-order determinism).
5. **No c7 scope-off.** The delta is not already covered by the
   ge-oracle c7 scope-off classes (hash-randomized inputs,
   beyond-envelope sizes, round-trip fidelity the oracle itself lacks).

## Adjudication-record template

Each adjudication is recorded as a wai decision record at
`.wai/projects/decision-trace/designs/<date>-decision-<slug>.md`:

```
# Decision: <case> classifies <(a)/(b)/(c)> [<slug>]

**Date:** <date>
**Case:** <bead / cut element, one line>
**Meter:** this record (per-item verdicts + final class)

| # | item | verdict | evidence |
|---|------|---------|----------|
| 1 | deterministic probe reproduction, ≥2 seeds | HOLDS / FAILS | <probe, seeds, observed file> |
| 2 | no spec constraint contradicts gently | HOLDS / FAILS | <constraint ids checked> |
| 3 | upstream defect recorded + linked | HOLDS / FAILS | <issue URL or beads id> |
| 4 | gently renderable + deterministic (ge.layout c1) | HOLDS / FAILS | <render + determinism check> |
| 5 | not covered by ge-oracle c7 scope-offs | HOLDS / FAILS | <which classes were checked> |

## Final class: <(a)/(b)/(c)>

<one-paragraph verdict>

## Consequences

<registry shipping, spec promotion, byte-compat gating>
```

## Registry

| date | case | class | record |
|------|------|-------|--------|
| 2026-10-14 | gently-58m (oracle as_txt omits Berlin, group-endpoint edge) | (b) | [E4 dry run record](../../.wai/projects/decision-trace/designs/2026-10-14-decision-e4-adjudication-dry-run-gently-58m.md) · defect: gently-278 |
