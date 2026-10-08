# Subagent brief: gently-2po.7 — tb.txt-render: minimal txt serialization for the tracer shape

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-2po.7` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real gates —
never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-2po.7` — the ticket
- Read `specs/ge-txt_render.md` (the corpus spec; c1–c4, p1–p4) and
  `specs/ge-oracle.md` (the pin: Graph::Easy v0.69 @ ededa3d7).
- `wai search "txt render tracer"` — accumulated patterns.

## What to build

Thin slice of `ge.txt_render` (tracer epic gently-2po): serialize ONLY the
tracer graph shape — two nodes `a`, `b`, one directed edge — to the
Graph::Easy canonical txt form, byte-identical to the pinned oracle.

**Oracle evidence (recorded by the orchestrator, pinned revision):**

```bash
# Graph::Easy v0.69 @ ededa3d7 (isolated lib at /var/tmp/ge069, ephemeral):
perl -IGraph-Easy-0.69/lib -MGraph::Easy \
  -e 'my $g = Graph::Easy->new; $g->add_edge("a","b"); print $g->as_txt'
# → "[ a ] --> [ b ]\n"
```

**Desired outcome:**
- `gently_core::graph` — minimal model: `Node { name }`, `Edge { from, to }`
  (indices into `Graph::nodes`), `Graph { nodes, edges }` with a
  `Graph::tracer()` constructor returning exactly the shape above.
- `gently_core::render::txt::render(&Graph) -> String` — tracer-slice
  serializer: each edge emits `[ <from> ] --> [ <to> ]\n`; a node in no
  edge emits `[ <name> ]\n`. Nothing more — no class sections, no styles,
  no ordering logic beyond the above (deepened by gently-3hv).
- Re-export `graph` and `render` from `crates/gently-core/src/lib.rs`.

**Exact test shape** (add to `crates/gently-core/tests/scenarios.rs` inside
`mod tb` as `mod txt_render`; the filter-compatible path is
`scenarios::tb::txt_render::...`):

```rust
#[test]
fn tracer_shape_matches_oracle() {
    let g = Graph::tracer();
    assert_eq!(txt::render(&g), "[ a ] --> [ b ]\n");
}
```

with the doc comment naming the oracle recording command (provenance).

**RED→GREEN→REFACTOR (mandatory order):**
1. RED: write the test first; run `cargo test --workspace`; record the
   failure (unresolved imports = honest red).
2. GREEN: implement the minimum above until `cargo test --workspace` is green.
3. REFACTOR: only if something is genuinely untidy; zero behavior change;
   separate commit if you refactor.

## Out of scope (do NOT do)

- No openspec change — this is a tb slice; the full `ge.txt_render`
  contracts (p1–p4) land with capability bead gently-3hv.
- No fixture corpus files (`tests/fixtures/`) — that is gently-2po.10.
- No CLI work (`gently-cli` untouched) — gently-2po.9.
- No layout code — gently-2po.8.
- No edits to justfile/lefthook.yml/CI — already wired by gently-2po.1.

## Hard scope guard

- Allowed files: `crates/gently-core/**`, `crates/gently-core/tests/scenarios.rs`
- Never edit: `openspec/`, `specs/` (the corpus), `.espectacular/`,
  `.wai/resources/**`, `justfile`, `lefthook.yml`, `.github/`, this brief.

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene** (standing, every commit): before ANY `git commit`, run
  `git status --short` and stage only files YOU authored
  (`git add <paths>`, never bare `git add`); unstage foreign files.
  Attribute the message only to what the diff contains.
- **Pre-commit gates run automatically** (lefthook: ah check, spk lint,
  spec-drift-gate, specodelic lint) — a dirty foreign file will fail them.
- **Do NOT push** — the orchestrator verifies and pushes.
- **Do NOT run `bd close`** — the orchestrator owns close.
- **Do NOT run `wai close`** — the orchestrator owns session close.
- **New source files** need Purpose/Responsibilities/Rationale headers
  (file-headers convention).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- Commits should reference the ticket: `(gently-2po.7)`.

## Gates you must leave green

- `cargo test --workspace` (include the unmodified
  `scenarios::tb::workspace_smoke`)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `ah check` (structural, 0 findings)

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
