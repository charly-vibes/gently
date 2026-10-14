# Design: decision-trace layer + oracle adjudication for gently

Date: 2026-10-14 (revised 2026-10-14 after Rule-of-5/TypeSafe review, verdict NEEDS_REVISION) · Phase: research → design inputs
Companion research: this project, 2026-10-14 session audit.

> **Status: D1–D6 are PROVISIONAL, pending experiment results E1–E4 (D7).**
> The project decides by experiment, not by design fiat: which decisions
> harden into commitments is gated by the E-results and the D7 matrix.
> Terminology fixed by review (CLAR-001): the artifact is the
> **decision trace**; "trace schema" and "event stream" are its schema
> and serialization, not rival names.

## D1 — The trace schema is a first-class artifact, not a log format

Decision: define a versioned "decision trace" schema with one
sub-vocabulary per pipeline stage (parse, graph model, rank/col, bands,
spans, route classification, bends, glyphs), each event carrying its
governing spec id.

Rationale (CT): the schema IS the intermediate category Dec. Comparing
two implementations of the pipeline is only well-defined relative to a
shared Dec; byte comparison (Inp → Byte at the codomain) cannot localize
divergence. Route's existing enum (layout.rs) is the seed vocabulary.

Consequence: the schema gets its own spec file (specodelic-linted) and
version field from day one; no overfitting to today's five beads.

## D2 — Oracle normalization is a committed, tested artifact (CORR-001)

Provisional decision: the oracle→Dec normalization is an explicit table
in tests/repro (not prose), with a faithfulness test restated pairwise
over fixtures (the original "decision states" phrasing had no identity
or enumeration — unwritable as a test):

> For every pair of corpus fixtures whose oracle dumps differ at stage
> S, the normalized traces must differ in S's image; and per fixture,
> canonically-ordered state snapshots must normalize to identical
> traces regardless of dump ordering.

Rationale: upstream concepts don't map 1:1 onto Route; an unspecified
normalization makes trace-diff unsound (misses real divergences) or
noisy (invents them). Faithfulness is testable; unfaithfulness is a
schema bug, not a tool quirk. Computability of the pairwise form on
real dumps is itself under test (E3).

## D3 — trace-diff reports minimal cuts, not "the first" (CORR-002)

Decision: divergence is a set over the decision dependency poset; the
diff reports its minimal antichain (cut set). A linear "first" is
presentation only.

Rationale: decision divergence can reconverge (different placements,
equal downstream); a unique "first divergent decision" may not exist.

## D4 — Oracle dumps are committed companions, seed-pinned, stage-observable

Provisional decision: probes.pl gains a dump mode emitting the Dec
schema from the Perl side (wrapping _prepare_layout, _balance_sizes,
_near_places); dumps carry the oracle pin header AND the hash seed as an
explicit field, and mark per-stage observability so partially-observed
traces are never reported as divergences. **Committed-vs-on-demand is
NOT decided here** — the review flagged unbounded repo growth (71-file
claims corpus alone); E2 pilots the five bead fixtures, measures size,
and the D7 matrix settles storage policy from data. Tier-2
(hash-seed-dependent) fixtures are excluded from the committed-dump
corpus regardless of outcome.

Rationale: kills the per-session scratch-probe loop (486 probe
invocations, the lost-probes incident probes.pl already documents);
hash-seed dependence (4ga fan class) makes seed-blind dumps
unreproducible.

## D5 — Adjudication: the spec sketch is the arbiter, the oracle is a model

Provisional decision: when trace-diff reports a cut, each cut element is
classified:

- (a) gently diverges → fix gently (default).
- (b) oracle defect → gently is right(er). Gate on the pre-registered
  checklist below.
- (c) underdetermined → human decides.

**Pre-registered (b)-checklist** (written before any case is adjudicated,
to guard against wanting gently to be better — review CLAR-002; each
adjudication is recorded as a wai decision record):

1. A claim probe reproduces the upstream behavior deterministically
   (re-run under ≥2 hash seeds; nondeterministic ⇒ tier-2, not (b)).
2. No existing spec constraint contradicts gently's chosen behavior —
   if one does, the conflict is a spec amendment question, not a (b).
3. The upstream issue is filed and links the probe.
4. gently's behavior is renderable and deterministic under its own
   invariants (ge.layout c1 model-order determinism).
5. The delta is not already covered by the ge-oracle c7 scope-off
   classes.

All five must hold for (b); any failure drops the element to (a) or (c).

Rationale (CT): correctness = satisfaction of the spec sketch; the
pinned oracle is one model of the sketch — a sufficient heuristic for
byte-compat, never the definition of correct. Adjudication moves a
divergence from "model mismatch" to "sketch amendment" with evidence.
Precedent exists: c7 already scopes off round-trip gaps the oracle
itself has; probes.pl claim mode already records upstream-defect
evidence.

Consequence: byte-compat stays the CI gate; adjudicated improvements are
promoted into specs (gently's behavior becomes a spec claim), so
"better" is decided once, in review, with evidence — not re-litigated
per session.

## D6 — Gating policy

Decision: trace-diff runs only on byte-divergent fixtures (or downgrades
decision-only diffs to informational). Byte equality remains the CI
gate; traces are the debugging and adjudication interface.

Rationale: render is many-to-one on decisions; byte-equal outputs can
come from different traces.

## Property to property-test (EXCL-001)

If N (D2) is faithful and traces are complete over render's dependency
set, every byte-divergent fixture yields a nonempty minimal cut, and
trace-diff computes it. Corpus-level property test. **Completeness is
itself under test (E1)**: the retrospective check confronts the property
with already-fixed divergences before it is trusted on open ones.

## D7 — Experiment plan and decision matrix (review EXCL-001; gates D1–D6)

The best way is decided by experiment. Four pre-registered experiments,
each with hypothesis and kill criteria; the matrix below is populated
ONLY from E-results and its rows feed wai decision records.

**E1 — Retrospective localization.** Take two already-fixed divergences
(gently-awd span slice @ commit 8a0d56b; gently-4ga render-error fix).
Check out the pre-fix revision, build the decision trace for the stage
the fix touched, run diff. *Hypothesis:* the reported cut cites that
stage. *Kill:* cuts point elsewhere, or reaching them cost more than the
original byte-diff hunt.

**E2 — Oracle dump pilot.** Instrument `_prepare_layout` only, on the
five bead fixtures. *Hypothesis:* <50 KB total dumps, <1 day of work.
*Kill:* >500 KB, or needs hooking >5 subs per stage ⇒ dumps go
on-demand, not committed (settles D4's storage question with data).

**E3 — Faithfulness computability.** Implement D2's pairwise restated
criterion against E2's dumps. *Kill:* the pairwise test is not
computable on the corpus without hooking every upstream sub.

**E4 — Adjudication dry run.** Run the D5 pre-registered checklist on
gently-58m (the oracle's `as_txt` omits Berlin entirely — plausible
upstream defect). *Hypothesis:* class (b) is reachable with evidence
already in hand. *Kill:* checklist stalls on missing evidence classes,
⇒ the registry ships later, not with the trace layer.

**Decision rule:** proceed to the openspec proposal only if E1 hits its
hypothesis AND E2 stays under kill thresholds. E4's outcome decides
whether the "deliberate improvements" registry ships with the trace
layer or later. Option C is pursued only if the matrix shows the overlay
is what makes adjudication tractable.

**Options × criteria matrix** (scores filled from E-results; no scoring
without a run):

| option | localization power | adjudication support | cost | drift resilience | overfit risk |
|---|---|---|---|---|---|
| D: status quo+ (package old probes) | low | none | lowest | low | low |
| A: Rust trace + byte-gated diff | med | low | low | high | med |
| B: A + oracle dumps (D1–D6) | high | high | med | med | med |
| C: B + overlay/lens | high | high | high | med | med |

## Open questions (carried)

- Which stage first: route classification + cell placement (4/5 open
  beads live there) — proposed, unconfirmed; E1's stage choice doubles
  as the probe of this.
- Does the trace belong in spk-compiled assertions (spec corpus) or stay
  a debug tool? Lean: both — spec ids on events make the trace the
  corpus's runtime witness.

