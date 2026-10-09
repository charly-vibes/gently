# Subagent brief: gently-bzx — implement ge.text_parser (post-6j0 amended spec)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-bzx` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-bzx`
- Read `specs/ge-text_parser.md` IN FULL — it was re-derived to the pinned
  oracle in closed gently-6j0 (commits dfb0ce3/8ffadc9/d46f5cc). Rows
  c1–c9 + model + properties p1–p9 are the binding contract. Every
  property row becomes a passing test named for its id.
- Read the probe evidence — binding observed behavior, captured from
  Graph::Easy v0.69 @ `ededa3d787ad89ac532c578c06390e8a7b270499`:
  - `tests/repro/claims/sharp-label.observed` (c7: in-string `#` truncates
    even inside quotes; `\#` escapes; hex colours auto-escape)
  - `tests/repro/claims/operator-patterns.observed` (c9: style follows the
    LAST unit token; `..-..-..>` is a VALID dotted edge; `.-`/`..-` valid
    single; plain units need 2+; missing endpoints are errors; `<`
    bidirectional family)
  - `tests/repro/claims/group-syntax.observed` (c6: colon joins the group
    name → `G:`; one-group membership, later declaration MOVES the node;
    nested groups: inner nodes belong only to the inner group; anonymous
    groups named `Group #N`)
  - `tests/repro/claims/anon-reference.observed` (c1: anon nodes are named
    `#N`, odd counter, reusable via `[ \#N ]`)
- **POD divergence warning:** upstream POD claims `..-..-..>` and `.-` are
  invalid — the CODE accepts both. Follow the probes, never the POD.
- Existing code: `crates/gently-core/src/parse/text.rs` (tracer-slice
  parser: edge lines + bare node lines only, 169 lines) and
  `crates/gently-core/src/graph/` (Node/Edge/Group-stub/AttributeTable).
- Existing scenario wiring pattern: `crates/gently-core/tests/scenarios.rs`
  → `tests/scenarios/ge_txt_render.rs` (one test per property id) and
  `.espectacular/ge-txt_render/p*.toml` (flags use real cargo test paths
  like `ge_txt_render::p1` — cargo does NOT include the file name in test
  paths, so a filter `scenarios::ge-txt_parser::p1` would never match).
- `wai search "text_parser"` for accumulated patterns.

## What to build

The full `ge.text_parser` capability in `crates/gently-core`:

- **c1/p1** — named tokens intern + reuse; bare `[ ]` creates an anon node
  named `#N` (odd counter) reusable via escaped `[ \#N ]`.
- **c2/p2** — the full operator table (directed + `<`-prefixed
  bidirectional family) → edge styles; missing-endpoint operators are
  parse errors; style-only styles via attribute, never as operators.
- **c3/p3** — chains produce one edge per adjacent pair, shared nodes.
- **c4/p4** — inline edge labels with matching flanks; arrow-less inline
  labels are errors.
- **c5/p5** — attribute blocks on nearest preceding object; class sections
  `graph/node/edge/group { ... }`.
- **c6/p6** — group blocks with colon-name, one-group membership with
  move-on-redeclare, nesting semantics, anonymous `Group #N`.
- **c7/p7** — `#` comment truncation everywhere (mid-line, in quotes),
  `\#` escape, hex-colour special case.
- **c8/p8** — typed errors with 1-based line + reason (keep the existing
  `ParseError` shape).
- **c9/p9** — unit-token operator grammar per the amended row.

Model surface: extend `crates/gently-core/src/graph/` ONLY as the parser
contract requires (e.g. real group membership on the Group stub). Do not
change renderer behavior; every existing test must stay green —
especially `ge_txt_render` (its p3 round-trip binds to the parser-supported
subset; deepening the parser must not regress it) and the tracer
pipeline.

Contract wiring: create `tests/scenarios/ge_text_parser.rs` (one test per
property id, mirroring the ge_txt_render layout), register it in
`tests/scenarios.rs`, and update `.espectacular/ge-text_parser/p1–p9.toml`
flags to the real cargo test paths so `ah check --run-tests` executes
them. After your change, the `ah check --run-tests` no-tests-ran backlog
must shrink by the nine text_parser contracts.

## TDD sequence (red → green → refactor commits)

1. **RED:** commit the scenario tests + probe-derived fixtures first —
   each property id one `#[test]`, honestly failing against the current
   tracer parser.
2. **GREEN:** implement the grammar in `src/parse/text.rs` (and the
   minimal model surface) until all tests pass. Keep functions small —
   the pretender ratchet enforces line limits on `src/` files (see the
   ddl-pretender-gate hook; it forced a `main()` split in gently-0h9).
3. **REFACTOR:** tidy grammar helpers, align probe evidence naming, wire
   `.espectacular` flags, README/docs touch-ups — separate commit.

## Hard scope guard

- Allowed files: `crates/gently-core/src/parse/text.rs`,
  `crates/gently-core/src/parse.rs` (module docs only),
  `crates/gently-core/src/graph/**` (minimal parser-serving surface),
  `crates/gently-core/tests/scenarios.rs`,
  `crates/gently-core/tests/scenarios/ge_text_parser.rs` (new),
  `.espectacular/ge-text_parser/p1.toml`…`p9.toml` (flags only),
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
ah check --run-tests   # text_parser p1–p9 must now execute
just gates
```

## Repo facts (boilerplate)

- **Commit hygiene:** before ANY `git commit`, `git status --short`; stage
  only files YOU authored; end subjects with `(gently-bzx)`.
- **File headers:** every source file carries the
  Purpose/Responsibilities/Rationale header; update `src/parse/text.rs`'s
  header to describe the full grammar, not the tracer slice.
- **Oracle pin:** any new observation runs under
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl …`
- If a probe contradicts the amended spec, STOP that row, record raw
  observations in your report — do not silently amend or guess.

## Follow-up threshold

If the full grammar does not fit the session (operators + groups +
attributes + classes is the floor), ship the coherent subset that keeps
every existing test green and file follow-up beads listing exactly what
remains — never leave the tree red.