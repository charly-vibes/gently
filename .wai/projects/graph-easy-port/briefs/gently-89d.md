# Subagent brief: gently-89d — layout spec re-derivation (c3/p3, c1, c4)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-89d` and is hands-off: you own
the re-derivation end-to-end. SPEC-RE-DERIVATION ticket — deliverable is
amended rows + probe evidence.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-89d`
- Read `specs/ge-layout.md` IN FULL — c1–c6, model, properties p1–p5.
- Grounded findings (review report, probed against the pinned oracle):
  - **c3/p3** (source strictly earlier than target along the flow axis;
    only self-loops exempt): any cycle (A→B→A) or a per-edge `flow:`
    override places an edge AGAINST the configured flow — c3 has no
    consistent reading; p3 cannot be satisfied. Probe:
    `tests/repro/claims/layout-flow-direction.observed` (cycle arrows
    wrap: oracle draws the back-edge around).
  - **c1** (pure function, identical inputs identical outputs): the
    oracle has a default 5-second `alarm()` timeout — at ≥200 nodes it
    dies before producing output, so "pure function" is unverifiable
    by the oracle at that scale (and the oracle is also
    hash-seed-dependent at some scales per the ikm determinism work).
    The DETERMINISM claim itself is still true of gently's own
    implementation — re-scope what c1 can assert and how it is
    verified (Tier-3 raw-scaling per ge.oracle c7 for out-of-envelope
    sizes).
  - **c4** (every edge routed through distinct cells, parallel edges
    distinct): false at 50 nodes — the oracle silently drops edges at
    scale (probed; see report finding 3). Re-scope: c4 can only be a
    within-envelope claim; large-scale fidelity is oracle-scoped-off.
- Note the gently-4nx scope note (in
  `crates/gently-core/tests/scenarios/ge_layout.rs` doc comments): c3
  strict-earlier was already falsified by the oracle for cycles and
  shared-target shapes — p3's generator excludes those shapes; 89d
  owns the re-derivation (you).
- The implementation (src/layout.rs, src/layout/flow.rs,
  placement.rs) shipped in 4nx follows the recorded oracle behavior on
  the corpus.

## What to do

1. **Re-probe the three findings** cheaply (reuse existing probes —
   `tests/repro/claims/layout-flow-direction.observed` and the 50-node
   silent-drop evidence; add a probe only if a specific case is
   unpinned). Confirm: cycle/per-edge-flow behavior, alarm timeout
   scale, silent edge drop at 50 nodes.
2. **Amend rows** with re-derivation notes (established style — read
   specs/ge-graphviz_render.md c4 for the pattern):
   - c3: re-scope to the verifiable core — source-before-target holds
     for acyclic, uniform-flow inputs; cycles and per-edge flow
     overrides are pinned as documented divergences (the oracle wraps
     back-edges; gently's placement follows the recorded corpus).
     Adjust p3's generator/predicate accordingly.
   - c1: determinism of gently's own algorithm stays; verification is
     tiered — oracle-comparable within the recorded envelope,
     gently-only (deterministic re-render) beyond it.
   - c4: qualify to within-envelope inputs; note the oracle's
     silent-drop at scale makes large-graph edge-count fidelity
     oracle-scoped-off (gently's own invariant: every model edge gets
     a path — verified structurally).
   - Only touch c3/c1/c4 and their deriving properties p1/p3/p4 (and
     the model guards if a transition cites them).
3. Deploy openspec mirrors in the SAME commit (`tools/deploy_specs.py`),
   recompile props via `just gates`.
4. No implementation changes; if an amended row exposes an
   implementation gap, file a follow-up bead.

## Hard scope guard

- Allowed: `specs/ge-layout.md` + openspec mirror + regenerated props,
  `tests/repro/**` (probes only if needed),
  `crates/gently-core/tests/scenarios/ge_layout.rs` ONLY to realign a
  test doc-comment with an amended row (no test logic changes).
- NEVER edit: other specs, `src/**`, `crates/gently-cli/**`, `.wai/**`.
- Do NOT push, do NOT close the bead, do NOT run `wai close`.
- Commit subjects end with `(gently-89d)`. File headers required.

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

**Probe evidence** (each finding confirmed/corrected)

**Spec rows amended** (which rows, one line each)

**Commits** (hash + one line)

**Follow-up beads filed** (or "none")

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, ge-layout.md, the probes,
  4nx scope note. Max 8 tool calls.
- **Minutes 5–12:** re-probe/verify the three findings.
- **Minutes 12–22:** row amendments + mirror + props.
- **Minutes 22–27:** gates, follow-up beads.
- **Minutes 27–30:** report.
Never re-read a file you have already read. A committed partial verdict
beats an uncommitted complete one.