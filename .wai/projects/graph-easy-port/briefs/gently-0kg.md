# Subagent brief: gently-0kg — re-derive round-trip claims (graphviz_render c2/c4, txt_render c3)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-0kg` and is hands-off: you own
the re-derivation end-to-end. This is a SPEC-RE-DERIVATION ticket: your
deliverable is amended spec rows + evidence, and only code changes if the
spec verdict forces them (see scope guard).

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-0kg`
- The claims to re-derive (ticket origin — a review report's finding 5):
  - `specs/ge-graphviz_render.md` c2 (line 19): edge arrow `->`/`--`
    matches the model edge direction.
  - `specs/ge-graphviz_render.md` c4 (line 21): feeding the emitted DOT
    through ge.dot_parser yields a model isomorphic to the source.
  - `specs/ge-txt_render.md` c3 (line 21... row id c3): parsing emitted
    text reproduces the same model (the group case is the divergence).
- The probed oracle divergences (from gently-dcp review report, probed
  against the pinned oracle):
  1. `as_graphviz` always emits `digraph` + `->` — an undirected or
     bidirectional model edge comes back DIRECTED (c2's faithful-arrow
     claim is not what upstream does).
  2. Named groups round-trip as `cluster0, cluster1, ...` (names lost).
  3. `as_graphviz` CRASHES on an edge whose endpoint is a group
     (probe evidence: `tests/repro/claims/group-edge.observed` —
     group-to-group edges parse fine upstream, but as_graphviz dies on
     them).
- **The implementation landscape has CHANGED since the report:** b4v
  shipped `src/render/graphviz/` (with documented deviations: arrows
  mirror the model, named groups keep names, group endpoints emitted
  faithfully, global attr blocks omitted) and 89p shipped the dot
  parser. Re-derive against BOTH the pinned oracle and the shipped
  implementation.

## What to do

1. **Probe the current state** (use the existing probe harness):
   `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl claim <name>`
   — read `tests/repro/probes.pl` first to see existing claims. Add
   claims ONLY where a genuine behavior question needs pinning (record
   each in your report).
2. **Verify the shipped graphviz_render round-trip** — write a quick
   Rust test scratch (or use the existing
   `tests/scenarios/ge_graphviz_render.rs::p4`) exercising:
   undirected edge → emit → re-parse; named-group graph → emit →
   re-parse (does group membership survive? does the name?);
   group-endpoint edge → emit → re-parse. Record exact behavior.
3. **Verdict per claim** — for each of c2/c4/graphviz and txt c3:
   - If the claim holds for the shipped implementation: leave the row.
   - If the row overclaims (e.g. c4 isomorphism cannot hold when a
     group-endpoint edge is in the graph, or upstream's always-directed
     emission makes a byte-compat c2 impossible): amend the row with a
     re-derivation note per the established pattern (see
     `specs/ge-dot_parser.md` and `specs/ge-graph_model.md` r22
     amendments for the note style: `<!-- re-derived ... -->` style or
     the wording pattern used there — READ one amended row first).
   - Amend ONLY the rows the ticket names (graphviz_render c2/c4,
     txt_render c3), plus their deriving property predicates if the
     constraint's meaning changes.
4. **Spec workflow**: edit `specs/*.md` directly (this is the
   established amendment path from r22/13f/6j0), run `spk lint specs/`
   and `tools/deploy_specs.py` equivalents via `just gates` —
   spec-drift-gate requires the openspec/ mirrors be deployed in the
   SAME commit as the corpus change: run the deploy tool, then
   `git add` specs + deployed mirrors together.
5. **Code changes only if forced**: if an amended row makes an existing
   test wrong, adjust that test to the amended contract — but do NOT
   implement new renderer/parser features; file a follow-up bead
   instead (see below).

## Hard scope guard

- Allowed files: `specs/ge-graphviz_render.md`, `specs/ge-txt_render.md`
  (+ deployed mirrors under `openspec/` via the deploy tool),
  `tests/repro/**` (new probes/claims with .observed),
  `crates/gently-core/tests/**` ONLY to realign an existing test with
  an amended row,
  `.espectacular/ge-graphviz_render/*.toml`, `.espectacular/ge-txt_render/*.toml` (flags only, if needed).
- NEVER edit: `crates/gently-core/src/**`, `crates/gently-cli/**`,
  other specs, `.wai/**`.
- File follow-up beads (`bd create`) for any implementation work your
  verdicts demand (title prefix the capability, P2, description names
  the spec row + evidence). Do NOT close other beads; do NOT close
  gently-0kg; do NOT push.
- File headers: every source file carries the
  Purpose/Responsibilities/Rationale header. Commit subjects end with
  `(gently-0kg)`.

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

**Verdicts**
- graphviz_render c2: <holds|amended> — <one-line reason + evidence>
- graphviz_render c4: <holds|amended> — <...>
- txt_render c3: <holds|amended> — <...>

**Commits**
- `<hash>` <message> — <what>

**Probes added** (name → .observed path, or "none")

**Follow-up beads filed** (ids + titles, or "none")

**Gates run** — <same six lines as impl briefs>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, the two spec files, one amended
  row example from ge-dot_parser.md. Max 8 tool calls.
- **Minutes 5–12:** probe/verify the three divergence cases.
- **Minutes 12–20:** verdicts + spec row amendments + deploy mirrors.
- **Minutes 20–27:** gates, follow-up beads.
- **Minutes 27–30:** report.
Never re-read a file you have already read. A committed partial verdict
beats an uncommitted complete one.
