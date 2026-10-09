# Subagent brief: gently-3hv — spec: implement ge.txt_render

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-3hv` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real gates —
never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-3hv`
- Read `specs/ge-txt_render.md` IN FULL — it is the binding contract:
  constraints c1–c4, model, properties p1–p4. Every property row becomes a
  passing test named for its id.
- Existing code: `crates/gently-core/src/render/txt.rs` (tracer-era
  serializer: tracer shape only, no class sections/styles/ordering) and
  `crates/gently-core/tests/scenarios.rs` `mod ge_txt_render` (capacity
  contracts landed by gently-2po.7 with unit tests p1–p4).
- Upstream reference: Graph::Easy 0.69's `lib/Graph/Easy/AsTxt.pm`
  (pinned source fetchable: `curl -fsSLO
  https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-0.69.tar.gz`).
  Its test corpus is authoritative; gently's txt form must match it.
- The tracer pipeline (parse → layout → render txt/ascii → CLI) depends on
  `txt::render` — all existing tests must stay green. Deepen; don't break.

## What to build

The full `ge.txt_render` capability in `crates/gently-core`:

**c1/p1 — class sections:** output starts with class attribute sections for
graph, node, edge, and group classes, emitted in sorted class order with
sorted attribute order. This is upstream `as_txt` behavior when
`class_attributes` / class attribute data exist.

**c2/p2 — object emission:** every node is emitted with its name and its
instance attributes; every edge as an operator chain whose operator matches
the edge style and direction. Each object appears exactly once with a
correct operator form. Study upstream `AsTxt.pm` for the operator/style
mapping (arrow directions for north/south/east/west ends, styles like
`-->`, `==>`, `..>`, undirected forms) and implement the equivalent for
the attribute set your model supports.

**c3/p3 — round-trip:** parsing the emitted text with `ge.text_parser`
reproduces a model with the same nodes, edges, styles, labels, directions,
and attributes as the source model. Re-parsing the canonical txt must
reproduce the source under the model-equality predicate. Coordinate with
the parser API as it currently exists (`crates/gently-core/src/parse/` —
read it, don't change its binding behavior); if the parser lacks a
feature your model round-trips, document the deviation and keep the test
to the supported feature set.

**c4/p4 — fixture corpus:** recorded upstream Graph::Easy fixtures, stored
under `tests/fixtures/graph-easy/` as input-text files with expected
canonical-text companions, captured from one pinned upstream revision,
render byte-identically. The fixture corpus is pinned to one upstream
revision; re-recording expectations is a deliberate, separately reviewed
change, never a side effect of code edits.

**API freedom:** you may reshape `txt::render` internals (signature stays
`render(&Graph) -> String` or a documented equivalent the pipeline can
call) as long as the tracer pipeline stays green — update internal call
sites as needed. Public tracer behavior (byte-identical oracle output) must
not change.

**Tests — contract-aligned naming (critical):**
- Contract tests live in `crates/gently-core/tests/scenarios.rs`
  `mod ge_txt_render` with `#[test] fn p1() ... p4()` — full paths
  `scenarios::ge_txt_render::p1` etc. The contracts landed by gently-2po.7
  already use this naming; verify the real test names match.
- Reconcile the espectacular contract flags with the real test names:
  edit `.espectacular/ge-txt_render/p1.toml` … `p4.toml`, changing
  `flags = "scenarios::ge-txt_render::pN"` (hyphen — unmatchable) to
  `flags = "scenarios::ge_txt_render::pN"` (underscore). This is the
  sanctioned spec-test correspondence fix, in scope for this bead.
- Additionally unit tests for internal invariants are welcome (separate
  names, not pN).

**RED→GREEN→REFACTOR (mandatory order):**
1. RED: p1–p4 tests first; `cargo test --workspace`; record failures.
2. GREEN: implement until green.
3. REFACTOR: separate commit, zero behavior change.

## Out of scope (do NOT do)

- Parser/renderer changes beyond adapting `ge.text_parser` call sites to
  the round-trip feature set (the binding parser contract is another
  spec's — do not chase it).
- Other specs' contracts or corpus edits.

## Hard scope guard

- Allowed files: `crates/gently-core/**`, `.espectacular/ge-txt_render/*.toml`
- Never edit: `openspec/`, `specs/`, other `.espectacular/*` dirs,
  `.wai/**`, `justfile`, `lefthook.yml`, `.github/`.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before ANY commit; stage only
  files YOU authored; attribute only what the diff contains.
- **Pre-commit gates run automatically** (lefthook).
- **Do NOT push**; **do NOT `bd close` / `wai close`** — orchestrator owns.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: `-f`/`-rf`/`-y` flags.
- Commits reference the ticket: `(gently-3hv)`.

## Gates you must leave green

- `cargo test --workspace` — all existing tests AND p1–p4
- `cargo clippy --workspace --all-targets -- -D warnings`
- `ah check` — 0 structural findings
- `ah check --run-tests` — the 4 ge-txt_render contracts EXECUTE and pass
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
- `ah check --run-tests` (ge-txt_render contracts) → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">
