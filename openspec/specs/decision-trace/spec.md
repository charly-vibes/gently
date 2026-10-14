---
id: decision.trace
kind: intent
statement: "WHEN a layout or render divergence is investigated against the pinned oracle THE decision trace SHALL localize it as a versioned, stage-structured artifact diffed by minimal cuts over a committed, tested oracle normalization."
---

# decision-trace

The decision-trace capability: the intermediate decision category `Dec`
between input and bytes is a first-class, versioned artifact, not a log
format (design D1,
`.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md`,
D7 go/no-go record 2026-10-14-decision-d7-go-no-go-openspec-proposal.md).
Experiments E1 (gently-asc), E2 (gently-8je), E3 (gently-ugy), and E4
(gently-btf) all PASSED and harden the decisions below: E1 proved
retrospective localization (both cuts at pre-registered stages, ~8 s
driver), E2 settled committed dumps for the pilot fixture class (5130 B,
one wrapped sub), E3 proved the pairwise faithfulness criterion
computable on the committed dumps, and E4 reached class (b) with
evidence in hand (gently-58m, oracle defect). Trace emission is
env-gated (`GENTLY_TRACE`/`GENTLY_TRACE_FILE`); with trace off the
corpus stays byte-identical (E1 regression property), and byte equality
remains the CI gate — traces are the debugging and adjudication
interface, never a rival gate (design D6).

## Purpose

The decision-trace capability: the intermediate decision category `Dec`
between input and bytes is a first-class, versioned artifact, not a log
format (design D1,
`.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md`,
D7 go/no-go record 2026-10-14-decision-d7-go-no-go-openspec-proposal.md).
Experiments E1 (gently-asc), E2 (gently-8je), E3 (gently-ugy), and E4
(gently-btf) all PASSED and harden the decisions below: E1 proved
retrospective localization (both cuts at pre-registered stages, ~8 s
driver), E2 settled committed dumps for the pilot fixture class (5130 B,
one wrapped sub), E3 proved the pairwise faithfulness criterion
computable on the committed dumps, and E4 reached class (b) with
evidence in hand (gently-58m, oracle defect). Trace emission is
env-gated (`GENTLY_TRACE`/`GENTLY_TRACE_FILE`); with trace off the
corpus stays byte-identical (E1 regression property), and byte equality
remains the CI gate — traces are the debugging and adjudication
interface, never a rival gate (design D6).

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | The decision trace is a versioned artifact — a `version` field is mandatory from day one — carrying one sub-vocabulary per pipeline stage (parse, graph model, rank/col, bands, spans, route classification, bends, glyphs), and every event carries its governing spec id; the seed vocabulary for route classification is the `Route` enum of `crates/gently-core/src/layout.rs` (`Selfloop`, `Grew`, `Parallel(usize)`, `Around`, `PortBelow`, `PortAbove`), and no stage vocabulary may invent route spellings that contradict it (design D1). | [[decision.trace]] | |
| c2 | invariant | The oracle→Dec normalization is the explicit committed table `tests/repro/trace/normalization-table.md` (rows N1–N8), never prose, each row enforced by a named subtest in `crates/gently-core/tests/e3_faithfulness.rs` (`row_nN_*`); the pairwise faithfulness property holds: for every pair of corpus fixtures whose oracle dumps differ at stage S, the normalized traces differ in S's image, and per fixture, canonically-ordered state snapshots normalize to an identical trace regardless of dump ordering (design D2 / CORR-001; E3 PASS gently-ugy). | [[decision.trace]] | [[decision.trace.c5]] |
| c3 | invariant | Divergence is a set over the decision dependency poset: trace-diff reports the minimal antichain (cut set), never a unique "first" — a linear presentation is presentation only — and trace-diff runs only on byte-divergent fixtures, with decision-only diffs on byte-equal fixtures downgraded to informational; byte equality remains the CI gate (design D3 / CORR-002, D6; E1 PASS gently-asc). | [[decision.trace]] | |
| c4 | invariant | Oracle dumps are committed companions carrying the oracle pin header AND the hash seed as an explicit field, and mark per-stage observability so partially-observed stages are never reported as divergences; tier-2 (hash-seed-dependent) fixture classes stay excluded from the committed-dump corpus, and the exclusion is by admission probe naming [[ge.oracle.c7]], not prose (design D4; E2 PASS gently-8je). | [[decision.trace]] | |
| c5 | effect | A committed dump that is invalid — missing the pin header, missing the explicit hash-seed field, missing per-stage observability marks, or a tier-2 (hash-seed-dependent) fixture present in the committed-dump corpus — produces a typed error naming the defect and the remediation (re-dump under the pinned seed per the README Recording recipe, or removal from the corpus via the admission probe [[ge.oracle.c7]]). | [[decision.trace]] | |
| c6 | invariant | Adjudication of every reported cut element is governed by the pre-registered (b)-checklist registry committed under `tests/repro/trace/` (design D5; E4 dry-run record gently-btf): (a) gently diverges → fix gently (default); (b) oracle defect → all five checklist items must hold (deterministic probe reproduction under ≥2 hash seeds, no existing spec constraint contradicts gently, the upstream defect is durably recorded and links the probe, gently's behavior is renderable and deterministic under its own invariants per [[ge.layout.c1]], the delta is not covered by the [[ge.oracle.c7]] scope-off classes), and any failure drops the element to (a) or (c) underdetermined → human decides; each adjudication is recorded as a wai decision record — precedent: gently-58m classed (b) with all five items holding (`.wai/projects/decision-trace/designs/2026-10-14-decision-e4-adjudication-dry-run-gently-58m.md`). | [[decision.trace]] | |

## Model

### States

- `aligned`
- `divergent`
- `adjudicated`
- `dump_invalid`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | aligned | divergent | [[decision.trace.c3]] |
| t2 | aligned | dump_invalid | [[decision.trace.c4]] |
| t3 | divergent | adjudicated | [[decision.trace.c6]] |
| t4 | adjudicated | aligned | [[decision.trace.c1]] |

### Failure terminal

`dump_invalid` is a failure terminal: it emits [[decision.trace.c5]].

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[decision.trace.c1]] | trace events emitted across the pipeline stages with trace gating on and off | every event carries the schema version and its governing spec id; route-classification events use only the `Route` seed vocabulary; with `GENTLY_TRACE` unset the full fixture corpus renders byte-identical (E1 regression pin) |
| p2 | unit | [[decision.trace.c2]] | the E3 faithfulness subtests over the committed `_prepare_layout` dumps | clause 1 (pairwise stage-difference preservation) and clause 2 (canonical-order insensitivity) hold; every normalization-table row N1–N8 is enforced by its named `row_nN_*` subtest |
| p3 | unit | [[decision.trace.c3]] | trace-diff runs over byte-divergent and byte-equal fixtures | every reported cut is a minimal antichain over the decision dependency poset with no unique "first"; byte-equal fixtures yield at most informational decision-only diffs; byte equality stays the CI gate |
| p4 | unit | [[decision.trace.c4]] | the committed dump corpus headers and the admission probe | every dump carries the pin header, an explicit hash-seed field, and per-stage observability marks; no tier-2 fixture is admitted, per [[ge.oracle.c7]] |
| p5 | unit | [[decision.trace.c5]] | dump environments with a missing pin header, a missing seed field, missing observability marks, and an admitted tier-2 fixture | each case yields a typed error naming the defect and the remediation |
| p6 | unit | [[decision.trace.c6]] | the recorded adjudication decision records | each classification carries per-item verdicts for all five checklist items; every (b) verdict has all five holding; the gently-58m precedent is recorded as a wai decision record |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** trace events emitted across the pipeline stages with trace gating on and off
- **THEN** every event carries the schema version and its governing spec id; route-classification events use only the `Route` seed vocabulary; with `GENTLY_TRACE` unset the full fixture corpus renders byte-identical (E1 regression pin)
- **VERIFIES** [[decision.trace.p1]]

#### Scenario: p2

- **WHEN** the E3 faithfulness subtests over the committed `_prepare_layout` dumps
- **THEN** clause 1 (pairwise stage-difference preservation) and clause 2 (canonical-order insensitivity) hold; every normalization-table row N1–N8 is enforced by its named `row_nN_*` subtest
- **VERIFIES** [[decision.trace.p2]]

#### Scenario: p3

- **WHEN** trace-diff runs over byte-divergent and byte-equal fixtures
- **THEN** every reported cut is a minimal antichain over the decision dependency poset with no unique "first"; byte-equal fixtures yield at most informational decision-only diffs; byte equality stays the CI gate
- **VERIFIES** [[decision.trace.p3]]

#### Scenario: p4

- **WHEN** the committed dump corpus headers and the admission probe
- **THEN** every dump carries the pin header, an explicit hash-seed field, and per-stage observability marks; no tier-2 fixture is admitted, per [[ge.oracle.c7]]
- **VERIFIES** [[decision.trace.p4]]

#### Scenario: p5

- **WHEN** dump environments with a missing pin header, a missing seed field, missing observability marks, and an admitted tier-2 fixture
- **THEN** each case yields a typed error naming the defect and the remediation
- **VERIFIES** [[decision.trace.p5]]

#### Scenario: p6

- **WHEN** the recorded adjudication decision records
- **THEN** each classification carries per-item verdicts for all five checklist items; every (b) verdict has all five holding; the gently-58m precedent is recorded as a wai decision record
- **VERIFIES** [[decision.trace.p6]]

