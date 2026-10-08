# Subagent brief: gently-2po.8 — tb.model+layout: minimal model + one deterministic layout path

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-2po.8` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real gates —
never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-2po.8` — the ticket
- Read `specs/ge-graph_model.md` and `specs/ge-layout.md` (corpus specs —
  c1 determinism, c2 no-overlap/grid-containment are the binding constraints
  for this slice; their full property sets land with gently-4ht/gently-4nx).
- Existing code: `crates/gently-core/src/graph.rs` (tracer model from
  gently-2po.7), `src/render/txt.rs`. Extend, don't rewrite.

## What to build

Thin slices of `ge.graph_model` + `ge.layout` sufficient for the tracer
graph (`[a] -> [b]`):

**Model deepening (only what layout needs):**
- Whatever `graph.rs` needs so layout can consume it (e.g. node/edge access
  helpers). Do NOT add groups, attributes, labels, or styles yet — the
  existing `Node`/`Edge`/`Graph` shape stays.

**One deterministic layout path (`gently_core::layout`):**
- `pub fn layout(graph: &Graph) -> Layout` (or an equivalent clean type).
- `Layout` must expose, at minimum: each node's grid cell `(x, y)`, and the
  routed edge path as a list of grid cells (orthogonal segments only).
- Tracer semantics: rank(a) = 0, rank(b) = 1 with rank growing eastward;
  both nodes on the same row; edge routed orthogonally from a's east side
  to b's west side through the gap column(s); the path never enters a node
  cell.
- Constraint `ge.layout.c1`: the layout is a pure function — identical
  inputs produce identical output. No randomness, no hashmap iteration
  order dependence.
- Constraint `ge.layout.c2`: no two node cells overlap; every node fully
  inside the grid.

**Oracle provenance (recorded by the orchestrator, pinned Graph::Easy
v0.69 @ ededa3d7 — this is the geometry the eventual ascii render in
gently-2po.9/.10 must reproduce):**

```
perl -IGraph-Easy-0.69/lib -MGraph::Easy \
  -e 'my $g = Graph::Easy->new; $g->add_edge("a","b"); print $g->as_ascii'
# →
# +---+     +---+
# | a | --> | b |
# +---+     +---+
```

Choose grid coordinates consistent with this shape (a west of b, same row,
one-cell gap is fine) — the exact gap width gets pinned when the ascii
renderer lands; do not over-fit beyond "consistent with the oracle ascii".

**Exact test shape** (add to `crates/gently-core/tests/scenarios.rs` inside
`mod tb` as `mod layout`; filter-compatible path
`scenarios::tb::layout::...`):

```rust
#[test]
fn tracer_layout_is_deterministic_and_non_overlapping() {
    let g = Graph::tracer();
    let l1 = layout::layout(&g);
    let l2 = layout::layout(&g);
    assert_eq!(l1, l2);                       // c1 determinism
    // a west of b, same row; distinct cells   // c2
    // edge path orthogonal, touches neither node cell
}
```

(You may split into several focused tests; keep at least: determinism,
relative position/rank, no-overlap, edge-path orthogonality + no node-cell
entry.)

**RED→GREEN→REFACTOR (mandatory order):**
1. RED: tests first; `cargo test --workspace`; record the honest failure.
2. GREEN: minimum implementation until green.
3. REFACTOR: only if genuinely untidy; zero behavior change; separate commit.

## Out of scope (do NOT do)

- No groups/attributes/labels/styles in the model; no multi-rank or
  multi-edge layout — gently-4ht / gently-4nx deepen those.
- No openspec change (tb slice). No renderer changes (`render/txt.rs`
  untouched). No CLI work. No fixture files. No justfile/lefthook/CI edits.

## Hard scope guard

- Allowed files: `crates/gently-core/src/**`, `crates/gently-core/tests/scenarios.rs`
- Never edit: `openspec/`, `specs/`, `.espectacular/`, `.wai/resources/**`,
  `justfile`, `lefthook.yml`, `.github/`, `crates/gently-cli/**`, this brief.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: before ANY `git commit`, run `git status --short`;
  stage only files YOU authored (`git add <paths>`, never bare `git add`).
  Attribute the message only to what the diff contains.
- **Pre-commit gates run automatically** (lefthook) — a dirty foreign file
  fails them.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `bd close` / `wai close`** — the orchestrator owns those.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: `-f`/`-rf`/`-y` flags; aliases may prompt.
- Commits reference the ticket: `(gently-2po.8)`.

## Gates you must leave green

- `cargo test --workspace` (all prior tests stay green)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `ah check` (0 findings)

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo test --workspace` → <result>
- `cargo clippy ...` → <result>
- `ah check` → <result>

**Deviations** (or "none")
- <any scope/plan deviation and why>

**Next**
- <exact next action for the orchestrator, or "ticket complete">
