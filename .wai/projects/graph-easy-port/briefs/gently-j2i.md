# Subagent brief: gently-j2i — graphviz_render: restore verbatim group names on the dot round trip

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core`, edition 2021). An orchestrator has
claimed ticket `gently-j2i` and is hands-off: you own the implementation
end-to-end.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-j2i`
- Read `specs/ge-graphviz_render.md` c3/c4 + p4 IN FULL — including the
  gently-0kg re-derivation note on c4 (the current contract allows
  group renaming up to `cluster_<name>` / `cluster<N>`; your job is to
  CLOSE that gap and tighten the rows to full identity).
- Code: `crates/gently-core/src/render/graphviz/mod.rs` (`cluster()`
  fn — named → `cluster_<name>`, anonymous → `cluster<N>`),
  `crates/gently-core/src/parse/dot/scope.rs` + `stmt.rs` (subgraph
  parsing — takes subgraph names verbatim as group names),
  `tests/scenarios/ge_graphviz_render.rs` (p3/p4 tests).
- Evidence: `tests/repro/claims/group-edge-graphviz.observed`
  (group-ENDPOINT edges stay OUT of scope — the amended c4 excludes
  them).

## What to build

Make the render→parse round trip preserve group identity:

1. **Parser side**: ge.dot_parser recognizes the renderer's cluster
   convention — a subgraph named `cluster_<name>` re-parses with group
   name `<name>` (verbatim restore); a subgraph named `cluster<N>`
   (bare digits after `cluster`, the anonymous form) re-parses as an
   ANONYMOUS group (empty name), restoring anonymity.
2. **Renderer/parser agreement**: if the prefix scheme cannot express
   some edge case (e.g. a real group actually NAMED `cluster_foo` —
   ambiguous), pick the deterministic resolution (single
   interpretation rule) and document it in the module header +
   report. Do not invent escape hatches beyond the convention.
3. **Tighten the spec** (you amend rows ONLY where the ticket
   dictates): after identity round-trips, amend c4 + p4 in
   `specs/ge-graphviz_render.md` to state full identity (drop the
   "up to group renaming" carve-out, keep the group-endpoint
   exclusion). Run `spk lint specs/`. Deploy the openspec mirror in
   the SAME commit as the spec edit (spec-drift-gate compares the
   worktree to the index — check `just gates` / the drift-gate hook;
   mirror lives under `openspec/` via `tools/deploy_specs.py` — read
   that tool's help before running).
4. **Tests**: strengthen `tests/scenarios/ge_graphviz_render.rs` p4
   (and p3 if needed) to assert verbatim names + anonymity restore.
   Keep p1/p2/p3 green.

## TDD sequence (red → green → refactor commits)

1. **RED:** extend p4 to assert identity round-trip (failing).
2. **GREEN:** parser-side cluster convention + any renderer-side fix;
   commit as soon as tests pass.
3. **REFACTOR:** spec rows amendment + mirror deploy in the same
   commit as the spec change; tidy — separate commit.

## Hard scope guard

- Allowed files: `crates/gently-core/src/parse/dot/**`,
  `crates/gently-core/src/render/graphviz/**` (only if agreement
  demands),
  `crates/gently-core/tests/scenarios/ge_graphviz_render.rs`,
  `crates/gently-core/tests/scenarios/ge_dot_parser.rs` (only if a dot
  scenario needs the convention pinned),
  `specs/ge-graphviz_render.md` + deployed `openspec/` mirror,
  `.espectacular/ge-graphviz_render/p*.toml` (flags only, if needed).
- Never edit: other specs, `crates/gently-cli/**`, `.wai/**`,
  `Cargo.toml`.
- Do NOT push, do NOT close the bead, do NOT run `wai close` — the
  orchestrator owns ship.
- Commit subjects end with `(gently-j2i)`. File headers required.

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
spk lint specs/
ah check
ah check --run-tests
just gates
```

## Report format (end with this)

## Report

**Commits** (hash + one line each)

**Convention chosen** (cluster_ prefix / anonymous cluster<N> resolution)

**Spec rows amended** (c4/p4 or "none")

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, spec c3/c4/p4, cluster() +
  scope.rs. Max 8 tool calls.
- **Minutes 5–9:** RED (extend p4 to identity round-trip).
- **Minutes 9–20:** GREEN parser-side cluster convention.
- **Minutes 20–26:** spec amendment + mirror deploy (same commit),
  gates.
- **Minutes 26–30:** report.
Never re-read a file you have already read. A committed RED beats an
uncommitted almost.
