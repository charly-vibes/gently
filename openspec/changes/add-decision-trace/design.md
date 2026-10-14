# Design: add-decision-trace

The authoritative design is the experiment-gated design document:

`.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md`

with the go/no-go decision record:

`.wai/projects/decision-trace/designs/2026-10-14-decision-d7-go-no-go-openspec-proposal.md`

Decisions carried into this change (D1–D6, hardened from PROVISIONAL by
E1 PASS + E2 PASS):

| id | decision | experiment evidence |
|---|---|---|
| D1 | trace schema is a first-class versioned artifact, one sub-vocabulary per stage, spec id per event | E1: cuts name stages; schema matches the pre-registered vocabulary |
| D2 | oracle→trace normalization is a committed, tested table; faithfulness restated pairwise | E3 pending — computability is itself under test |
| D3 | trace-diff reports the minimal antichain, never "the first" | E1: awd cut is a placement-event antichain (no unique first) |
| D4 | oracle dumps committed for tier-1 fixtures, pin header + seed field + observability marks | E2: 5130 B total, 1 sub, ~1 s — committed is cheap |
| D5 | spec sketch is the arbiter, the oracle is one model; pre-registered (b)-checklist | E4 pending — registry timing gated |
| D6 | trace-diff runs only on byte-divergent fixtures; byte equality stays the CI gate | E1: corpus byte-identical with trace off |

Open items this change must NOT pre-decide:

- Overlay/lens (option C) is not pursued — no E-evidence motivates it.
- Storage scale-out beyond the five fixtures waits on E3's outcome.
