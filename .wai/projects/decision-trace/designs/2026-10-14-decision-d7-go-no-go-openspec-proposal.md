# Decision: D7 go/no-go — proceed to the openspec proposal (gently-25r)

**Date:** 2026-10-14 · **Ticket:** gently-25r (P2) · **Status:** DECIDED — **GO**
**Gates:** gently-ugy (E3), gently-btf (E4) — both now unblocked/open
**Inputs:** gently-asc (E1, CLOSED, PASS), gently-8je (E2, CLOSED, PASS)

## Decision rule (from the design doc, D7)

> Proceed to the openspec proposal only if E1 hits its hypothesis AND E2
> stays under kill thresholds.

## E-results

### E1 — retrospective localization (gently-asc): PASS

- Both cuts belong to the pre-registered stage for their case:
  - **awd @ 8a0d56b:** minimal cut = per-node placement events (band rows
    + rank→col span accumulation) → `layout.place`, as pre-registered.
    Oracle-side cut vs the aligned E2 dump also all-place events
    (post-fix Client x=8 matches; y residual = known parallel-channel delta).
  - **4ga @ 280f9dc:** minimal cut = route e2 A→D (cells=2/bends=0 jump vs
    cells=5/bends=2 jog) → `render.route`, as pre-registered; the route
    trace observes the defect before the render-time adjacency error fires.
- Kill criteria NOT hit: no cut pointed elsewhere; cost ~8 s driver wall
  clock + 1 subagent session vs the original multi-session byte-diff /
  oracle-decode hunts.
- Anti-goal guard held: subagent preregistration was committed before any
  fix-diff read and independently agreed with the orchestrator's stages.

### E2 — oracle dump pilot (gently-8je): PASS

- 5130 B total dumps across the five E2 fixtures (< 50 KB PASS ceiling;
  kill was > 500 KB).
- 1 sub wrapped (`_prepare_layout` only; kill was > 5 subs per stage).
- ~1 s run.
- D4 storage question settled for this fixture class: **committed dumps
  are cheap**. Tier-2 (hash-seed-dependent) fixtures stay excluded from
  the committed-dump corpus regardless.

## Matrix verdict (scores now filled from E-results — see the design doc)

- Option D (status quo+): loses on localization and adjudication support.
- Option A (Rust trace + byte-gated diff): E1 alone confirms high
  localization power at low cost, but the awd oracle-side cut was only
  decidable against the aligned E2 dump — adjudication support stays low.
- **Option B (A + oracle dumps, D1–D6): selected.** High localization +
  high adjudication support, med cost, both measured.
- Option C (B + overlay/lens): not pursued — the matrix shows no
  E-evidence that the overlay is what makes adjudication tractable
  (D7's condition for pursuing C).

## Consequences

1. **GO:** the openspec proposal `add-decision-trace` is drafted
   (decision-trace schema D1, oracle dump mode D4, normalization table D2,
   trace-diff minimal-cut D3, adjudication registry D5).
2. **Registry timing still gated:** E4 (gently-btf) decides whether the
   "deliberate improvements" registry ships with the trace layer or
   later; the proposal marks that task pending E4.
3. **E3 (gently-ugy)** remains the scale-out gate: the pairwise
   faithfulness criterion (D2) must be computable on real dumps before
   the normalization table is trusted beyond the five fixtures.
4. D1–D6 harden from PROVISIONAL to proposal-backed; per-decision
   hardening happens through proposal review, not by fiat.
