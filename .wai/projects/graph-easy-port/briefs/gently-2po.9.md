# Subagent brief: gently-2po.9 — tb.cli: end-to-end pipeline over the tracer shape

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-2po.9` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real gates —
never claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-2po.9`; read `specs/cli.md` (c3, c4 are the binding
  constraints for this slice) and `specs/ge-ascii_render.md` (intro only).
- Existing code: `crates/gently-core/src/{graph.rs,layout.rs,render/txt.rs}`.
  You will add a minimal parser + ascii renderer and rewrite the CLI stub.
- genesis-vibes: spec cli.c5 requires crate >= 0.12 — `genesis-vibes 0.12.1`
  is on crates.io (confirmed). Study its API from the vendored source after
  `cargo add`/build (`~/.cargo/registry/src/*/genesis-vibes-0.12.1/`) —
  Guide dispatch, `Output::emit`. Sibling repos using genesis (dont,
  dulce-de-leche at 0.10) show real usage patterns; adapt to 0.12 APIs.

## What to build

The tracer end-to-end slice: input text → parse → model → layout → ascii
render → stdout, exit 0.

**1. Minimal text parser (`gently_core::parse::text` or similar):**
- Parses the tracer text form: `[ a ] --> [ b ]` (also accept `->`), with
  flexible whitespace, returning a `Graph` with two nodes and one edge.
- Anything it cannot parse → a typed parse error (no panics). Full
  `ge.text_parser` capability lands with gently-bzx.

**2. Minimal ascii renderer (`gently_core::render::ascii`):**
- Consumes the `Layout` (gently-2po.8) and draws the grid.
- **Byte-identical to the pinned oracle for the tracer shape** (recorded
  from Graph::Easy v0.69 @ ededa3d7):

```
+---+     +---+
| a | --> | b |
+---+     +---+
```

(exactly: node boxes `+---+` / `| a |` / `+---+`, 5-char gap columns with
` --> ` on the middle row, trailing newline). Derive the char-grid mapping
from the layout cells; keep the mapping explicit and deterministic.

**3. CLI (`gently-cli`):**
- Name the binary `gently` (`[[bin]] name = "gently"`).
- Built on genesis-vibes 0.12: command dispatch via `Guide`, output via
  `Output::emit`. Verbose/-q flags and `init`/`doctor` subcommands are NOT
  in this slice (gently-ef5 deepens; note them as stubs only if genesis
  forces structure).
- Input: file argument or stdin.
- `--format <fmt>`: `ascii` is the only valid format in this slice.
  Unknown format → diagnostic on stderr naming the requested format AND the
  valid formats, exit code 2 (cli.c4).
- Human mode: rendered output to stdout as raw bytes; diagnostics to
  stderr; exit 0 on success; nonzero on parse/render errors (cli.c3).
  No `--json` yet (ef5).

**Exact test shape** — add to `crates/gently-cli/tests/cli.rs` (integration
test using `env!("CARGO_BIN_EXE_gently")`):

```rust
#[test]
fn tracer_end_to_end_ascii_matches_oracle() {
    // stdin: "[ a ] --> [ b ]\n" → stdout byte-identical to the pinned
    // oracle ascii block above, exit 0, empty stderr
}
#[test]
fn unknown_format_exits_2_naming_formats() { /* cli.c4 */ }
#[test]
fn unparseable_input_fails_nonzero_with_diagnostic() { /* cli.c3 */ }
```

**RED→GREEN→REFACTOR (mandatory order):**
1. RED: tests first; `cargo test --workspace`; record honest failures.
2. GREEN: minimum implementation until green.
3. REFACTOR: only if genuinely untidy; separate commit.

## Out of scope (do NOT do)

- No `--json`/Envelope, no verbosity flags, no `init`/`doctor`, no DOT
  input, no other render formats — gently-ef5 / capability beads deepen.
- No openspec change, no fixture files (gently-2po.10 records those), no
  edits to justfile/lefthook/CI/.github.

## Hard scope guard

- Allowed files: `crates/gently-core/src/**`,
  `crates/gently-core/tests/scenarios.rs`, `crates/gently-cli/**`
  (including its Cargo.toml for the genesis dependency + bin name).
- Never edit: `openspec/`, `specs/`, `.espectacular/`, `.wai/**`,
  `justfile`, `lefthook.yml`, `.github/`, root `Cargo.toml` unless adding
  nothing (workspace deps stay as-is; add genesis-vibes in gently-cli's
  own Cargo.toml).

## Repo facts (boilerplate — applies to every ticket)

- **Commit hygiene**: `git status --short` before ANY commit; stage only
  files YOU authored; attribute only what the diff contains.
- **Pre-commit gates run automatically** (lefthook) — dirty foreign files
  fail them. Cargo.lock will change when you add genesis-vibes: commit it.
- **Do NOT push**; **do NOT `bd close` / `wai close`** — orchestrator owns.
- **New source files** need Purpose/Responsibilities/Rationale headers.
- **Non-interactive shells**: `-f`/`-rf`/`-y` flags.
- Commits reference the ticket: `(gently-2po.9)`.

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
