# Investigation: hard cases, pi sessions, and the missing semantic layer

Date: 2026-10-14 · Source session: gently pi session review (87 sessions)
Related project: graph-easy-port (byte-compat port); this project investigates
the *observability* layer that the port's debugging loop lacks.

## 1. Evidence from the pi sessions

87 gently sessions, ~9k tool calls (terapia over ~/.pi/agent/sessions):

- 7,267 bash · 751 `cargo test` · **486 ad-hoc Perl probe invocations** ·
  **678 greps over the pinned upstream source** (`/var/tmp/ge0.69`) ·
  372 touches of `.observed` claim files.
- The recurring hard-case loop: run fixture → byte diff → grep upstream
  Perl for the responsible sub (`_near_places`, `_balance_sizes`,
  `_draw_label`) → write a throwaway `/tmp/probeN.pl` wrapping the sub to
  dump internal state → hand-translate into Rust → re-run.
- Session 2026-10-13T22-07 (gently-awd channel work) rewrote probe4.pl —
  a cell-dump wrapper over `_prepare_layout` — from scratch; an earlier
  session wrote the same idea and lost it. probes.pl's own header
  admits it: "the original probe scripts were scratch files lost with
  their scratch directory".
- Error profile: exit_nonzero dominant (path/fn drift), the agent
  navigates by grep because there is nothing to *query*.

## 2. The hard cases are placement-engine semantics

All five open beads in docs/known-hard-cases.md are placement/routing
decisions the oracle models and gently does not (yet):

- gently-4ga — `_near_places` slot rings over grown clusters (fan placement;
  hash-seed-dependent upstream → tier-2)
- gently-awd — parallel-edge channel-as-grid-rows (boxes stretch to row spans)
- gently-byq — indexed ports `{ start: front, N }` (variable node heights)
- gently-fb1 — group_stack surviving bare scopes inside named subgraphs
- gently-58m — group-endpoint edge semantics in as_txt

None are glyph/table/parser problems anymore.

## 3. The gap: the semantic layer exists but is invisible

- gently already has a decision IR: `Layout` / `Placement` / `Route`
  (crates/gently-core/src/layout.rs) — `Grew`, `Parallel(n)`, `Around`,
  `PortBelow`, … But it is `pub(super)`, emitted nowhere.
- The oracle side has no equivalent at all: `.observed` files record
  *bytes only* (tools/oracle.pl, tests/repro/probes.pl).
- Therefore every divergence is diagnosed at the lowest resolution —
  final bytes — and the decision space is reverse-engineered per session.

## 4. Proposal (reviewed Rule-of-5, machine-verified via TypeSafe)

1. Typed decision trace (gently side): opt-in event stream over pipeline
   stages, spec-id-tagged (`ge.layout.c4` …), JSON via genesis CliFormat.
2. Oracle decision dumps: probes.pl `dump` mode wrapping
   `_prepare_layout` / `_balance_sizes` / `_near_places`, committed as
   companions next to byte companions.
3. trace-diff: align by entity, report divergence.
4. Overlay rendering: paint route/span/port decisions on the canvas.

### Review verdict: NEEDS_REVISION — two findings gate the design

- **CORR-001 (CRITICAL, verified 0.72 < gate → REVIEW_REQUIRED):** the
  oracle→schema normalization is the mathematical crux, not a risk
  footnote. Must be an explicit, committed normalization table with a
  testable faithfulness criterion: N is faithful iff oracle decision
  states that differ on any corpus output map to distinct traces.
- **CORR-002 (HIGH, verified 0.75 < gate → REVIEW_REQUIRED):** "first
  divergent decision" presumes a total order that doesn't exist —
  decision divergence can reconverge. The well-defined object is the
  *minimal antichain (cut set)* of the divergence set under the decision
  dependency poset, not a unique "first".
- Dropped (measured FPs, 2/5 = 40% → escalated): CORR-003 (byte-equal
  gate policy — survives as a design note: trace-diff gated on byte
  divergence), EDGE-001 (seed parameterization — survives as schema
  requirement, grounded in known-hard-cases hash-seed evidence).
- Stated guarantee to property-test: if N is faithful and the trace is
  complete over render's dependency set, every byte-divergent fixture
  has a nonempty minimal cut, and trace-diff computes it.

## 5. Category-theory formulation (annex of the review)

- gently / oracle = functors F, G : Inp → Byte; byte-compat compares
  them only at the codomain.
- The trace layer inserts the intermediate category Dec (the schema):
  F = render_G ∘ dec_G, G = render_O ∘ dec_O, N : Dec_O → Dec.
- render : Dec → Byte is quotient-like: equal decisions ⇒ equal bytes;
  Dec *refines* the byte-equality relation.
- trace-diff = equalizer computation of dec_G and N ∘ dec_O; output =
  minimal cut in the dependency poset.
- specs/*.md = a sketch; implementations are models; spec-id-tagged
  events are interpretations of sketch operations, checked sort-by-sort.
- Overlay = put direction of a lens Dec ⇄ Byte.
- Event log = Grothendieck construction ∫T of the staged decision functor.
- One line: **byte-compat compares two functors only at their codomain;
  the trace layer inserts the intermediate category that turns the
  comparison into an equalizer computation instead of a squint at
  final bytes.**

## 6. Oracle adjudication — the oracle is a reference model, not the theory

The byte-compat scope contract (specs/ge-oracle.md c7) already admits
classes where oracle equality is *unsatisfiable by construction*
(round-trip gaps the oracle itself has) or *meaningless* (hash-seed
dependence). Adjudication generalizes this from classes to instances:

When trace-diff reports a divergence cut, classify each cut element:

- **a — gently diverges:** fix gently (the current default assumption).
- **b — oracle defect:** gently is right(er). Requirements: claim probe
  reproducing the upstream behavior, upstream issue filed, spec amendment
  recording gently's chosen behavior, entry on the known-hard-cases page
  (or a new "deliberate improvements" registry).
- **c — underdetermined:** both defensible; human decides.

Key formal move: correctness = satisfaction of the spec sketch, NOT
oracle equality. The pinned oracle is one model of the sketch — a
sufficient heuristic for byte-compat, never the definition of correct.
Adjudication is the process that moves a divergence from "model
mismatch vs oracle" to "amendment of the sketch", with evidence, and it
needs the trace layer to be cheap: you cannot adjudicate a decision you
cannot see.

Existing precedent to build on: the scope-off registry, probes.pl claim
mode, and beads-tracked divergences on the hard-cases page.

