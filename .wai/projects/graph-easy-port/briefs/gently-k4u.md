# Subagent brief: gently-k4u — perf spec: re-scope budgets to the oracle-verifiable envelope

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-k4u` and is hands-off: you own
the re-derivation end-to-end. SPEC-RE-DERIVATION ticket — deliverable is
amended rows + recorded measurement evidence.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-k4u`
- Read `specs/ge-perf.md` IN FULL — c1–c5, model, properties.
- Evidence (pinned oracle, recorded): `tests/repro/claims/size-envelope.observed`
  — upstream at nodes=50/edges=141: 0.597 s; nodes=100/edges=291:
  2.744 s; nodes=200: ERROR (alarm timeout, no output). Superlinear
  growth. The 1000-node/2s and near-linear c2/c5 claims cannot be
  byte-compat AND oracle-comparable at that scale.
- The scope contract: `specs/ge-oracle.md` c7 (three tiers —
  byte-compat / oracle-scoped-off / raw-scaling) and the decision
  record `.wai/projects/graph-easy-port/designs/2026-10-09-decision-byte-compat-scope.md`
  — it explicitly notes "k4u owns the perf re-scope: 1000-node budgets
  are Tier-3 gently-only".
- Also check the ikm outcome: `specs/ge-oracle.md` c1 now pins seed 0 +
  perl version — perf measurement must respect determinism.

## What to do

1. **Measure the ORACLE envelope** (already probed — re-verify one
   point if cheap: `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl
   tests/repro/probes.pl claim size-envelope` or reuse the recorded
   .observed; do NOT run a 200-node oracle run that hangs — the alarm
   timeout already recorded it).
2. **Measure the GENTLY envelope** on the same fixture classes
   (50/100 nodes) — a quick release-build timing via the existing
   corpus harness or a `cargo build --release` + timed run. Record
   numbers. (This grounds the re-scoped budgets in BOTH engines.)
3. **Amend `specs/ge-perf.md`**:
   - c2: re-scope the heavy budget to what the oracle can verify
     (e.g. a 100-node/291-edge class with the measured oracle envelope
     as reference, and/or move 1000-node to a Tier-3 gently-only
     budget with a generous deterministic ceiling and no oracle
     comparison — follow the c7 tier language).
   - c5: rewrite the near-linear advisory honestly — upstream grows
     superlinearly and dies at 200 nodes; state the gently-only
     scaling expectation (e.g. scaling measured within the
     raw-scaling tier, regression reporting retained).
   - c1/c3/c4 stay unless measurement forces a change.
   - Update the property predicates accordingly. Add a re-derivation
     note per the established style (see amended rows in
     ge-graphviz_render.md / ge-txt_render.md).
   - Deploy the openspec mirror in the SAME commit
     (`tools/deploy_specs.py`, read help first), recompile props via
     `just gates`.
4. No implementation changes; if gently fails a re-scoped budget,
   measure, record the number in the report, and file a follow-up
   bead — do NOT optimize inline.

## Hard scope guard

- Allowed: `specs/ge-perf.md` + openspec mirror + regenerated props,
  `tests/repro/**` (probes/.observed only).
- NEVER edit: `crates/**`, other specs, `.wai/**` (except reading),
  `.github/**`, `justfile` (perf-check recipe changes belong to
  gently-4ln).
- Do NOT push, do NOT close the bead, do NOT run `wai close`.
- Commit subjects end with `(gently-k4u)`. File headers required.

## Exact gates (all green before you report done)

```sh
spk lint specs/
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
ah check
ah check --run-tests
just gates
```

## Report format (end with this)

## Report

**Measurements** (oracle envelope; gently envelope)

**Spec rows amended** (which rows, one line each)

**Commits** (hash + one line)

**Follow-up beads filed** (or "none")

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, ge-perf.md, c7 tiers,
  size-envelope.observed. Max 8 tool calls.
- **Minutes 5–12:** gently envelope measurement (release build).
- **Minutes 12–22:** row amendments + mirror + props.
- **Minutes 22–27:** gates.
- **Minutes 27–30:** report.
Never re-read a file you have already read. A committed partial verdict
beats an uncommitted complete one.
