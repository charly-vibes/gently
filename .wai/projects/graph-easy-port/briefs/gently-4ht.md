# Subagent brief: gently-4ht — spec: implement ge.graph_model

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-4ht` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real gates —
never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-4ht`
- Read `specs/ge-graph_model.md` IN FULL — it is the binding contract:
  constraints c1–c5, model, properties p1–p5. Every property row becomes a
  passing test named for its id.
- Existing code: `crates/gently-core/src/graph.rs` (tracer-era model:
  `Node { name }`, `Edge { from, to }` as indices, `Graph { nodes, edges }`).
  The tracer pipeline (parse → layout → render txt/ascii → CLI) depends on
  this API — all 34 existing tests must stay green. Deepen; don't break.

## What to build

The full `ge.graph_model` capability in `crates/gently-core`:

**c1/p1 — node identity:** named nodes unique per graph; duplicate-name
insertion merges into the existing node (upstream add_node semantics:
existing node returned, no second node created). Anonymous nodes allowed,
unreferencable by name after creation.

**c2/p2 — attributes:** per-object (node/edge/graph) AND class-scoped
attribute tables; values stored verbatim (no loss); assignment to one scope
leaves reads on other scopes unchanged. Derived border components (style,
width, color) are computed AT ASSIGNMENT TIME exactly as upstream Graph::Easy
does — study upstream `Attribute`/`Border` parsing (the pinned source is
fetchable: `curl -fsSLO https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-0.69.tar.gz`,
see `lib/Graph/Easy/Attribute.pm` / `Node.pm` border handling) and implement
the equivalent for the attributes you support. Keep the supported attribute
set explicit and documented in the module header.

**c3/p3 — edge integrity:** every edge references exactly two live nodes
(source, target); self-loops legal; no dangling edge ever escapes a mutation
(e.g. node removal drops or rewires incident edges — pick upstream-faithful
behavior and document it).

**c4/p4 — direction:** each edge directed or undirected, with per-end
arrow-head presence, preserved bit-exactly through every model round-trip
(accessors, clones, whatever round-trip means in your API).

**c5/p5 — published contract:** downstream consumers (layout, renderers —
they exist) read the model through public accessors/iterators; p5's test
asserts consumer-visible reads match direct model inspection.

**API freedom:** you may reshape `Graph`/`Node`/`Edge` internals (e.g. slab
indices instead of bare `Vec` positions) as long as the tracer pipeline
stays green — update the internal call sites (parse/layout/render) as
needed. Public tracer behavior (byte-identical oracle output) must not
change.

**Tests — contract-aligned naming (critical):**
- Add `mod ge_graph_model` inside `crates/gently-core/tests/scenarios.rs`
  with `#[test] fn p1() ... p5()` — full paths `scenarios::ge_graph_model::p1` etc.
- Reconcile the espectacular contract flags with the real test names:
  edit `.espectacular/ge-graph_model/p1.toml` … `p5.toml`, changing
  `flags = "scenarios::ge-graph_model::pN"` (hyphen — unmatchable) to
  `flags = "scenarios::ge_graph_model::pN"`. This is the sanctioned
  spec-test correspondence fix, in scope for this bead.
- Additionally unit tests for internal invariants are welcome (separate
  names, not pN).

**RED→GREEN→REFACTOR (mandatory order):**
1. RED: p1–p5 tests first; `cargo test --workspace`; record failures.
2. GREEN: implement until green.
3. REFACTOR: separate commit, zero behavior change.

## Out of scope (do NOT do)

- Groups (c-scope lists them in the intent but no property exercises them
  yet — leave a documented stub if the model shape wants one).
- Parser/renderer changes beyond adapting call sites to the new model API.
- Other specs' contracts or corpus edits.

## Hard scope guard

- Allowed files: `crates/gently-core/**`, `.espectacular/ge-graph_model/*.toml`
- Never edit: `openspec/`, `specs/`, other `.espectacular/*` dirs,
  `.wai/**`, `justfile`, `lefthook.yml`, `.github/`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before ANY commit; stage only
  files YOU authored; attribute only what the diff contains.
- **Pre-commit gates run automatically** (lefthook).
- **Do NOT push**; **do NOT `bd close` / `wai close`** — orchestrator owns.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: `-f`/`-rf`/`-y` flags.
- Commits reference the ticket: `(gently-4ht)`.

## Gates you must leave green

- `cargo test --workspace` — all existing tests AND p1–p5
- `cargo clippy --workspace --all-targets -- -D warnings`
- `ah check` — 0 structural findings
- `ah check --run-tests` — the 5 ge-graph_model contracts EXECUTE and pass
  (other specs' `no-tests-ran` findings are pre-existing epic state; do not
  chase them)

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo test --workspace` → <result>
- `cargo clippy ...` → <result>
- `ah check` → <result>
- `ah check --run-tests` (ge-graph_model contracts) → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">
