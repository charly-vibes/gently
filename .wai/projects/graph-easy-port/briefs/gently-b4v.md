# Subagent brief: gently-b4v — implement ge.graphviz_render

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-b4v` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## Orientation (do this first — max 10 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-b4v`
- Read `specs/ge-graphviz_render.md` IN FULL — rows c1–c4 + model +
  properties p1–p4 are the binding contract. Every property row becomes
  a passing test named for its id.
- Read the probe evidence — binding observed behavior, captured from
  Graph::Easy v0.69 @ `ededa3d787ad89ac532c578c06390e8a7b270499`
  (`Graph::Easy::Parser::Graphviz`):
  - `tests/repro/claims/graphviz-round-trip.observed` (the oracle's
    `as_graphviz` output: `digraph GRAPH_0 { ... }` header, generated-by
    comment, global `edge [ arrowhead=open ]` / `graph [ rankdir=LR ]` /
    `node [ fontsize=11, shape=box, style=filled, fillcolor=white ]`
    defaults, per-edge/per-node attrs after them)
- Also skim: `tests/repro/claims/dot-direction.observed`,
  `tests/repro/claims/group-edge.observed`,
  `tests/repro/claims/subgraph-handling.observed` (your round-trip
  counterpart is ge.dot_parser, shipped in gently-89p).
- Existing code: `crates/gently-core/src/render/{txt,html,ascii,boxart}/`
  (mirror the html/ or txt/ module-split style — the pretender gate
  enforces file line limits; split into `graphviz/` submodules when a
  file grows). Model surface: `crates/gently-core/src/graph/`
  (Node/Edge/Group/AttributeTable; note r22: edges may hold GROUP
  endpoints).
- `wai search "graphviz_render"` for accumulated patterns.

## What to build

The full `ge.graphviz_render` capability in `crates/gently-core`:

- **c1/p1** — node emission: exactly one DOT node statement per node,
  safely quoted names (spaces, unicode, quotes, `#N` anon ids), node
  attributes mapped to DOT counterparts (use graphviz-round-trip.observed
  as the shape oracle: name `a -> b [ color="#000000" ]` etc.).
- **c2/p2** — edge emission: one edge statement per model edge, arrow
  `->` vs `--` per model edge direction, edge style attributes mapped.
  Known 0kg oracle facts (already probed, use them): the oracle ALWAYS
  emits `digraph` + `->` — an undirected model edge still comes out
  directed in upstream output; group-endpoint edges CRASH upstream
  as_graphviz. For gently: emit from the model faithfully (arrow matches
  the model edge direction per c2 — the spec row wins over the oracle's
  lossy behavior); for group endpoints do NOT crash — pick the sanest
  faithful emission and record the deviation in your report.
- **c3/p3** — group emission: named subgraph clusters containing exactly
  member nodes, group label as cluster label; anonymous groups emit
  clusters named `cluster<N>` after internal id. NOTE the 0kg finding:
  upstream names clusters `cluster0..N` in emission order — follow the
  spec row: NAMED groups keep their name, ANONYMOUS get `cluster<N>`.
  If a probe contradicts, STOP the row, record raw observations in the
  report — never silently amend spec rows.
- **c4/p4** — round-trip: feeding the emitted DOT through
  `gently_core`'s dot_parser (src/parse/dot*, shipped by gently-89p)
  yields a model isomorphic to the source model (same nodes, edges,
  directions, group membership).

Module placement: `crates/gently-core/src/render/graphviz.rs` or
`src/render/graphviz/` split (mirror the txt/html style). Wire into
`src/render/mod.rs` and `src/lib.rs` exports only as needed.

**Contract wiring (critical):**
- Create `tests/scenarios/ge_graphviz_render.rs` (one `#[test]` per
  property id, mirroring the ge_dot_parser layout: `pub fn p1()`-style
  test names `p1`–`p4` inside module `ge_graphviz_render`), register in
  `tests/scenarios.rs` WITH the `#[path = "scenarios/ge_graphviz_render.rs"]`
  attribute (a previous RED commit shipped a bare `mod ge_dot_parser;`
  and broke the build — always carry the #[path] attr).
- Fix `.espectacular/ge-graphviz_render/p1–p4.toml` cargo flags from the
  broken `scenarios::ge-graphviz_render::pN` to
  `ge_graphviz_render::pN` (mirror the ge-dot_parser tomls). After your
  change, `ah check --run-tests`'s no-tests-ran backlog must shrink by
  the four graphviz_render contracts.

## TDD sequence (red → green → refactor commits)

1. **RED:** commit the scenario tests first — each property id one
   `#[test]`, honestly failing against the current (nonexistent)
   renderer.
2. **GREEN:** implement the renderer until all tests pass. Commit GREEN
   as soon as tests pass.
3. **REFACTOR:** tidy helpers, contract-flag wiring, README/docs
   touch-ups — separate commit.

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/**`,
  `crates/gently-core/src/render.rs` (if module docs live there),
  `crates/gently-core/src/graph/**` (minimal renderer-serving surface),
  `crates/gently-core/src/lib.rs` (exports only),
  `crates/gently-core/tests/scenarios.rs`,
  `crates/gently-core/tests/scenarios/ge_graphviz_render.rs` (new),
  `.espectacular/ge-graphviz_render/p1.toml`…`p4.toml` (flags only),
  `tests/repro/**` (only if a genuine behavior question forces a new
  probe — record it in your report, never amend spec rows).
- Never edit: other `specs/*.md`, `openspec/**`,
  `crates/gently-core/src/parse/**`, `crates/gently-cli/**`, `.wai/**`.
- Do NOT push, do NOT close the bead, do NOT run `wai close` — the
  orchestrator owns ship.

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
spk lint specs/
ah check
ah check --run-tests   # graphviz_render p1–p4 must now execute
just gates
```

## Repo facts (boilerplate)

- **Commit hygiene:** before ANY `git commit`, `git status --short`; stage
  only files YOU authored; end subjects with `(gently-b4v)`.
- **File headers:** every source file carries the
  Purpose/Responsibilities/Rationale header.
- **Oracle pin:** any new observation runs under
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl …`
- If a probe contradicts the spec, STOP that row, record raw
  observations in your report — do not silently amend or guess.

## Follow-up threshold

If the full capability does not fit the session (c1–c4 is the floor),
ship the coherent subset that keeps every existing test green and file
follow-up beads listing exactly what remains — never leave the tree red.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo test --workspace` → <result>
- `cargo clippy ...` → <result>
- `spk lint specs/` → <result>
- `ah check` → <result>
- `ah check --run-tests` (graphviz_render p1–p4) → <result>
- `just gates` → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — read this brief, the spec, and
  `graphviz-round-trip.observed`. That is ALL. Max 10 tool calls.
- **Minutes 5–10:** write the RED scenario file
  (`tests/scenarios/ge_graphviz_render.rs`) + register module (with
  #[path] attr); commit RED.
- **Minutes 10–22:** GREEN implementation in `src/render/graphviz*`;
  commit GREEN as soon as tests pass.
- **Minutes 22–28:** contract-flag wiring + gates.
- **Minutes 28–30:** report.
Never re-read a file you have already read. Never cat an entire
directory. If RED+GREEN cannot both land, ship RED-commit + partial
GREEN and say so in the report — a committed RED beats an uncommitted
almost.
