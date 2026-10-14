# decision-trace — delta spec

> OpenSpec delta for the `decision-trace` capability (new). The
> authoritative four-layer specodelic corpus file (`specs/decision-trace.md`)
> is authored during implementation (task 1.1); this delta fixes the
> requirement set it must encode.

## ADDED Requirements

### Requirement: Versioned decision trace schema with stage vocabularies

gently SHALL emit a versioned decision trace describing its pipeline
decisions (parse, graph model, rank/col, bands, spans, route
classification, bends, glyphs), where every event carries its governing
spec id, a `schema_version` field is present, and emission is disabled
unless explicitly enabled via `GENTLY_TRACE`/`GENTLY_TRACE_FILE`.

#### Scenario: trace-off corpus byte-identity

- **WHEN** no `GENTLY_TRACE` variable is set
- **THEN** every corpus fixture renders byte-identical to the committed
  expectations (E1-verified property, pinned as a regression test)

#### Scenario: stage-attributed events

- **WHEN** a trace is captured for a fixture
- **THEN** each event names its pipeline stage, the stage's
  sub-vocabulary, and the spec id governing that stage

### Requirement: Oracle dumps are committed, seed-pinned, observability-marked

The repro harness SHALL provide a dump mode emitting oracle-side
decision traces (starting with `_prepare_layout`, per the E2 pilot)
where each dump carries the oracle pin header, the hash seed as an
explicit field, and per-stage observability marks; dumps are committed
only for hash-stable fixture classes, and partially-observed traces are
never reported as divergences.

#### Scenario: seed-explicit dump

- **WHEN** a dump is recorded
- **THEN** the dump states the oracle pin and the `PERL_HASH_SEED`
  value used, so the recording is reproducible

#### Scenario: tier-2 exclusion

- **WHEN** a fixture's oracle output varies across hash seeds
- **THEN** the fixture is excluded from the committed-dump corpus

### Requirement: Normalization is a committed, tested table

The oracle→trace normalization SHALL be an explicit committed table
with a pairwise faithfulness test: for every pair of corpus fixtures
whose oracle dumps differ at stage S, the normalized traces differ in
S's image; and per fixture, canonically-ordered state snapshots
normalize to identical traces regardless of dump ordering.

#### Scenario: computability gate

- **WHEN** the normalization table is authored
- **THEN** the pairwise criterion is demonstrated computable on real
  committed dumps (experiment E3) before the table is trusted beyond
  the pilot fixtures

### Requirement: trace-diff reports minimal cuts, byte-gated

The trace-diff tool SHALL report divergence as the minimal antichain
(cut set) over the decision dependency poset — never a linear "first
divergent decision" — and SHALL run only on byte-divergent fixtures,
downgrading decision-only diffs to informational; byte equality remains
the CI gate.

#### Scenario: antichain, not "first"

- **WHEN** trace-diff compares a divergent fixture's traces
- **THEN** the report is a minimal cut set (E1 case: awd's
  placement-event antichain), not an ordered "first"

#### Scenario: byte-identical no-op

- **WHEN** the compared fixtures render byte-identically
- **THEN** trace-diff does not report a divergence

### Requirement: Adjudication registry with a pre-registered checklist

When trace-diff reports a cut element, the project SHALL classify it
(gently diverges / oracle defect / underdetermined) against the
pre-registered five-point (b)-checklist, recording every adjudication
as a wai decision record; the registry ships only if experiment E4
reaches class (b) with evidence already in hand.

#### Scenario: checklist gates the (b) class

- **WHEN** a cut element is proposed as an oracle defect
- **THEN** all five checklist points hold before the classification is
  recorded, else the element drops to (a) or (c)
