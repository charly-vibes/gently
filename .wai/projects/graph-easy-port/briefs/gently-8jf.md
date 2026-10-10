# Subagent brief: gently-8jf — txt_render: emit group membership so it round-trips

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core`, edition 2021). An orchestrator has
claimed ticket `gently-8jf` and is hands-off: you own the implementation
end-to-end.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-8jf`
- Read `specs/ge-txt_render.md` c3 + p3 IN FULL — including the
  gently-0kg re-derivation note (current contract excepts group
  membership: group sections emit the empty `( name )` form so members
  do not round-trip; your job is to CLOSE that gap and tighten the rows
  to full equality).
- Code: `crates/gently-core/src/render/txt/mod.rs` — `group_pass()`
  (line ~107) and `group_line()` (line ~216). Upstream shape evidence:
  `tests/repro/claims/group-edge-graphviz.observed` (upstream `as_txt`
  emits members as `( A\n  [ x ]\n)` — group section carries the
  member node declarations inside it).
- Parser side (the round-trip counterpart):
  `crates/gently-core/src/parse/text/node.rs` / `group handling` —
  check how the text parser reads group sections; the emitter change
  must produce text the parser already handles (group membership via
  node lines inside a group scope). Do NOT change the parser unless
  the probe forces it.
- Tests: `tests/scenarios/ge_txt_render.rs` — `p3` (round-trip
  property, line ~207) and the existing
  `group_sections_render_sorted_with_attributes` unit test in
  `src/render/txt/mod.rs`.

## What to build

1. **Emitter change**: `group_pass()` emits each group's member node
   declarations INSIDE the group section (the upstream shape —
   `( A:\n  [ x ]\n)`), not the empty `( name )` form. Members come
   from the model graph's group membership. Careful: a node can belong
   to exactly one group (text_parser rule) — if a node is in a group,
   emit its declaration inside that group's section and NOT in the
   bare node pass (avoid double declaration); verify what the parser
   does with duplicates and pick the deterministic form the parser
   round-trips correctly.
2. **Tests**: extend `p3` to assert group MEMBERSHIP round-trips
   (named group with members → emit → parse → same membership).
   Update `group_sections_render_sorted_with_attributes` to the new
   emission shape. Keep every other test green.
3. **Spec tightening** (only where dictated): amend c3 + p3 in
   `specs/ge-txt_render.md` to full model equality (drop the group
   membership carve-out; keep the group-endpoint-edge exclusion).
   `spk lint specs/`; deploy the openspec mirror and commit the spec
   edit + mirror in the SAME commit (spec-drift-gate). Use
   `tools/deploy_specs.py` (read its help first).

## TDD sequence (red → green → refactor commits)

1. **RED:** extend p3 to membership round-trip (failing).
2. **GREEN:** emitter change until tests pass; commit.
3. **REFACTOR:** spec amendment + mirror deploy (same commit); tidy
   emitter helpers; separate commit.

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/txt/**`,
  `crates/gently-core/src/render/txt.rs` (module docs),
  `crates/gently-core/tests/scenarios/ge_txt_render.rs`,
  `specs/ge-txt_render.md` + deployed `openspec/` mirror,
  `.espectacular/ge-txt_render/p*.toml` (flags only, if needed),
  `crates/gently-core/src/parse/text/**` ONLY if the probe proves the
  parser cannot read the emitter's output (record the evidence).
- Never edit: other specs, `crates/gently-cli/**`, `.wai/**`,
  `Cargo.toml`.
- Do NOT push, do NOT close the bead, do NOT run `wai close` — the
  orchestrator owns ship.
- Commit subjects end with `(gently-8jf)`. File headers required.

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
spk lint specs/
ah check
ah check --run-tests
just gates
```

## Repo facts

- CI is green at all 4 jobs — keep it that way.

## Report format (end with this)

## Report

**Commits** (hash + one line each)

**Emitter shape chosen** (member emission inside group section; duplicate
handling resolution)

**Spec rows amended** (c3/p3 or "none")

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, spec c3/p3, group_pass(),
  group-edge-graphviz.observed. Max 8 tool calls.
- **Minutes 5–9:** RED (p3 membership round-trip assert).
- **Minutes 9–20:** GREEN emitter change.
- **Minutes 20–26:** spec amendment + mirror deploy (same commit),
  gates.
- **Minutes 26–30:** report.
Never re-read a file you have already read. A committed RED beats an
uncommitted almost.
