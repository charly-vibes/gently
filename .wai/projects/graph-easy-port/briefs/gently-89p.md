# Subagent brief: gently-89p — implement ge.dot_parser (post-13f amended spec)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-89p` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-89p`
- Read `specs/ge-dot_parser.md` IN FULL — it was re-derived to the pinned
  oracle in closed gently-13f (commits 84fe037/0dc2f36/5050b2f). Rows
  c1–c5 + model + properties p1–p5 are the binding contract. Every
  property row becomes a passing test named for its id.
- Read the probe evidence — binding observed behavior, captured from
  Graph::Easy v0.69 @ `ededa3d787ad89ac532c578c06390e8a7b270499`
  (`Graph::Easy::Parser::Graphviz`):
  - `tests/repro/claims/dot-direction.observed` (c1/c2: the EDGE OPERATOR
    decides direction, not the header — `graph { a -> b }` is directed,
    `digraph { a -- b }` undirected; header only sets model `type` attr)
  - `tests/repro/claims/subgraph-handling.observed` (c4: any named
    subgraph becomes a group verbatim; nested nodes → innermost group
    only; named subgraph as edge endpoint → edge to the GROUP object;
    nameless `subgraph { }` keyword form is a tokenizing error but bare
    `{ }` scope parses — with the pinned spurious-edge caveat in p4)
  - `tests/repro/claims/dot-records-ports.observed` (c5: records and
    HTML-like table labels autosplit into `name.N` part nodes, port
    markers stripped + edges reattached; unresolvable port ref →
    `Cannot find autosplit node for <base>:<port> on edge <id>`; malformed
    HTML / nameless `subgraph` → tokenizing errors; no partial graph)
- **POD divergence warning:** the 13f amendments overrode the spec's own
  earlier claims (header-direction, cluster_-only groups, records
  rejected). Follow the probes/observed bytes, never prose.
- Existing code: `crates/gently-core/src/parse/` (only `text/` grammar
  module + `parse.rs` module docs) and `crates/gently-core/src/graph/`
  (Node/Edge/Group/AttributeTable — graph_model rows amended in r22:
  anon nodes `#N` via shared object-id counter, store-layer unquote at
  assignment, border decomposition, edges may hold GROUP endpoints,
  del_node drops incident edges / add_edge re-creates bare nodes).
- Existing scenario wiring pattern: `crates/gently-core/tests/scenarios.rs`
  → `tests/scenarios/ge_text_parser.rs` (one test per property id) and
  `.espectacular/ge-dot_parser/p1.toml`…`p5.toml` (flags already use
  `scenarios::ge-dot_parser::pN` — cargo does NOT include the file name
  in test paths, so your test module must be named `ge_dot_parser` inside
  `tests/scenarios.rs`; verify each toml filter actually matches).
- The implementation is a plain module (no cargo `[features]` gate exists
  yet — do NOT invent one this bead; the spec's "optional feature"
  wording is aspirational packaging).
- `wai search "dot_parser"` for accumulated patterns.

## What to build

The full `ge.dot_parser` capability in `crates/gently-core`:

- **c1/p1** — header handling: `digraph`/`graph` headers set the model
  graph's `type` attribute per the amended row; direction comes from the
  operator.
- **c2/p2** — edge statements: one model edge per arrow, direction per
  operator (`->` directed, `--` undirected), chains share nodes.
- **c3/p3** — attribute lists `[k=v, k2=v2]` map onto model attributes;
  quoted values unescape exactly once (route through the model's
  store-layer unquote — see r22 findings, do not double-unquote).
- **c4/p4** — subgraphs: named → verbatim group (innermost-membership
  nesting), anonymous keyword form → tokenizing error, bare `{ }` scope
  parses with the pinned caveat; group objects as edge endpoints.
- **c5/p5** — records/HTML-like labels autosplit into part nodes with
  port reattachment; unresolvable port refs + malformed constructs →
  typed errors with source position; no partial graph on failure.

Module placement: `crates/gently-core/src/parse/dot.rs` or
`src/parse/dot/` split (mirror the `text/` grammar-module style —
the pretender gate enforces file line limits; split helpers into `dot/`
submodules when a file grows). Wire it into `src/parse.rs` module docs
and `src/lib.rs` exports only as needed.

**Contract wiring (critical):**
- Create `tests/scenarios/ge_dot_parser.rs` (one `#[test]` per property
  id, mirroring the ge_text_parser layout), register
  `mod ge_dot_parser;` in `tests/scenarios.rs`.
- Verify `.espectacular/ge-dot_parser/p1–p5.toml` filters match the real
  test paths — after your change, the `ah check --run-tests`
  no-tests-ran backlog must shrink by the five dot_parser contracts.

## TDD sequence (red → green → refactor commits)

1. **RED:** commit the scenario tests + probe-derived fixtures first —
   each property id one `#[test]`, honestly failing against the current
   (nonexistent) parser.
2. **GREEN:** implement the DOT grammar in `src/parse/dot*` (and the
   minimal model surface) until all tests pass. Keep functions small.
3. **REFACTOR:** tidy grammar helpers, align probe evidence naming, wire
   `.espectacular` flags, README/docs touch-ups — separate commit.

## Hard scope guard

- Allowed files: `crates/gently-core/src/parse/**`,
  `crates/gently-core/src/parse.rs` (module docs),
  `crates/gently-core/src/graph/**` (minimal parser-serving surface),
  `crates/gently-core/src/lib.rs` (exports only),
  `crates/gently-core/tests/scenarios.rs`,
  `crates/gently-core/tests/scenarios/ge_dot_parser.rs` (new),
  `.espectacular/ge-dot_parser/p1.toml`…`p5.toml` (flags only),
  `tests/repro/**` (only if a genuine behavior question forces a new
  probe — record it in your report, never amend spec rows).
- Never edit: other `specs/*.md`, `openspec/**`,
  `crates/gently-core/src/render/**`, `crates/gently-cli/**`, `.wai/**`.
- Do NOT push, do NOT close the bead, do NOT run `wai close` — the
  orchestrator owns ship.

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
spk lint specs/
ah check
ah check --run-tests   # dot_parser p1–p5 must now execute
just gates
```

## Repo facts (boilerplate)

- **Commit hygiene:** before ANY `git commit`, `git status --short`; stage
  only files YOU authored; end subjects with `(gently-89p)`.
- **File headers:** every source file carries the
  Purpose/Responsibilities/Rationale header.
- **Oracle pin:** any new observation runs under
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl …`
- If a probe contradicts the amended spec, STOP that row, record raw
  observations in your report — do not silently amend or guess.

## Follow-up threshold

If the full grammar does not fit the session (headers + edges +
attributes + subgraphs + records/HTML/ports is the floor), ship the
coherent subset that keeps every existing test green and file follow-up
beads listing exactly what remains — never leave the tree red.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo test --workspace` → <result>
- `cargo clippy ...` → <result>
- `spk lint specs/` → <result>
- `ah check` → <result>
- `ah check --run-tests` (dot_parser p1–p5) → <result>
- `just gates` → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">
