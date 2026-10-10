# Subagent brief: gently-a83 — text_parser: store-layer %XX entity decode in attribute values

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core`, edition 2021). An orchestrator has
claimed ticket `gently-a83` (bug) and is hands-off: you own the fix
end-to-end.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-a83`
- Evidence (binding, from the pinned oracle
  Graph::Easy v0.69 @ `ededa3d7...`):
  `tests/repro/claims/attr-quote-value.observed` — the upstream store
  layer (`Graph::Easy::Attributes::unquote_attribute`) ALSO decodes %XX
  entities (`%41` → `A`) and strips %00-%1f/%7f plus high-bit percent
  escapes (exploit filtering). gently-8ol landed quote-strip + #"';\
  unescape layers but deliberately left %XX out.
- Code: `crates/gently-core/src/graph/attributes.rs`
  (`unquote_attribute` / the store-layer unquote path — gently-8ol/r22
  work; also `set_attribute`), and the text-grammar unquote flow
  `crates/gently-core/src/parse/text/attrs.rs`.
- Probe harness: `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl
  tests/repro/probes.pl claim <name>` — `attr-quote-value` claim exists
  (12 cases). Read `tests/repro/probes.pl` first.
- Existing contracts: `specs/ge-text_parser.md` c10+p10 (attribute
  values) — the %XX behavior belongs in the same row family if the
  corpus/spec needs the extension.

## What to build

1. **First verify corpus impact** (the bead asks for this): grep the
   recorded fixture corpus (`tests/fixtures/graph-easy/*.txt`) for
   `%[0-9A-Fa-f]{2}` inside attribute values — report whether any
   fixture actually exercises %XX decode (this determines if corpus
   oracle bytes change; if no fixture hits it, the fix is probe-scoped
   only and no corpus re-record is needed).
2. **RED:** extend the `ge_text_parser::p10` scenario (or add a
   focused unit test in `src/parse/text/attrs.rs`/`tests/`) pinning:
   `%41` → `A` decode; %00-%1f/%7f and high-bit percent escapes
   stripped; a literal `%` NOT followed by two hex digits stays
   literal (follow the .observed evidence — if the evidence doesn't
   pin a case, run the oracle probe for it and record the .observed).
3. **GREEN:** implement the decode/filter layer in the store-layer
   unquote (mirror upstream's per-position rule) until green. Do NOT
   double-decode — the dot parser route must stay single-unescape (see
   r22 notes in `src/graph/mod.rs`/attrs docs).
4. **Spec:** if the c10/p10 wording needs the %XX layer named, amend
   those two rows only (re-derivation note style as used in 0kg
   amendments), deploy the openspec mirror in the SAME commit
   (`tools/deploy_specs.py`, read help first), and recompile props via
   `just gates`.

## TDD sequence (red → green → refactor commits)

1. **RED:** scenario/unit tests pinning the %XX behavior (failing).
2. **GREEN:** store-layer decode/filter; commit.
3. **REFACTOR:** spec row amendment + mirror + props (same commit);
   separate commit.

## Hard scope guard

- Allowed: `crates/gently-core/src/graph/attributes.rs`,
  `crates/gently-core/src/parse/text/attrs.rs` (route-through only),
  `crates/gently-core/tests/scenarios/ge_text_parser.rs` (p10
  extension),
  `tests/repro/**` (probes/.observed),
  `specs/ge-text_parser.md` + mirrors + regenerated props,
  `.espectacular/ge-text_parser/p10.toml` (flags only if needed).
- Never edit: other specs, `crates/gently-cli/**`, `.wai/**`,
  `Cargo.toml`, `crates/gently-core/src/parse/dot/**`.
- Do NOT push, do NOT close the bead, do NOT run `wai close`.
- Commit subjects end with `(gently-a83)`. File headers required.

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

**Corpus impact** (fixtures exercising %XX, or "none — probe-scoped")

**Commits** (hash + one line each)

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, attr-quote-value.observed,
  attributes.rs unquote path, corpus grep. Max 8 tool calls.
- **Minutes 5–9:** RED tests.
- **Minutes 9–20:** GREEN decode/filter layer.
- **Minutes 20–26:** spec/mirror/props if needed + gates.
- **Minutes 26–30:** report.
Never re-read a file you have already read. A committed RED beats an
uncommitted almost.
