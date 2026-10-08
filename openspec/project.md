# Project Context

## Purpose

`gently` is a Rust port of Perl's [Graph::Easy](https://github.com/ironcamel/Graph-Easy)
(Tels, 2004–2008). Graph::Easy lets you build directed/undirected graphs
(nodes connected by edges), lay them out on a flat surface, and render
them as ASCII art, boxart, HTML, or Graphviz DOT — plus parse graphs from
its own human-readable text format and from DOT.

The port aims to be a faithful, dependency-light Rust implementation with
the same observable behavior, validated differentially against the
upstream test corpus.

## Tech Stack

- Rust (edition 2021, workspace with focused crates)
- **genesis-vibes** (crate >= 0.12) as the CLI foundation/orchestrator:
  Guide dispatch, `Output::emit` envelopes, `CliVerbosity`/`CliFormat`,
  `DoctorRunner`, suggestions, and the `fixture` module (see `cli`)
- Proptest for property-based verification (compiled from the spec corpus)
- Specodelic four-layer specs under `specs/` (lint/compile/model-check gates)
- OpenSpec for change management (`openspec/changes/`)

## Project Conventions

- **Spec-first**: capabilities are specified in `specs/*.md` (one file,
  one spec, four layers: Intent/Constraints/Model/Properties) before any
  implementation. Gates: `specodelic lint`, `specodelic compile`,
  `specodelic model-check` — all must pass with zero findings.
- **Golden tests**: the `txt` renderer (canonical text form) and the
  `ascii` renderer are the differential-testing surface; fixtures come
  from the upstream Graph::Easy `t/` corpus.
- **Pinned upstream revision**: all differential fixtures and glyph
  tables are captured against Graph::Easy **v0.69**, upstream master
  commit `ededa3d787ad89ac532c578c06390e8a7b270499` (2010-10-22).
  Changing the pin is a deliberate, separately reviewed change.
- **TDD / Tidy First**: every capability gets red→green→refactor cycles;
  refactors are separate changes from features (tracked in beads).
- **No implementation before approval**: openspec changes carry
  dual-format deltas; implementation happens only in the apply stage.
- **Autonomous implementation loop** (one bead = one capability): claim the
  ready bead (`bd ready` -> `bd update <id> --claim`) -> openspec proposal
  with dual-format deltas (real ids per the specodelic naming law,
  Revision 18 — no `id: spec`) -> `openspec validate --all --strict` ->
  TDD red→green→refactor until every Properties-table row is a passing
  test -> gates (`just lint`, `just gates`, `spk lint openspec`,
  `cargo test`/`clippy`, `ah check`) -> `openspec archive` -> `bd close`.
  Every property has a stubbed espectacular contract under `.espectacular/`
  (`ah init`); the implementer adds `[tests]` entries per contract as the
  capability's tests land — `ah check` must exit 0 (structural
  no-tests-declared findings are fine until the capability is implemented).
  The corpus
  `specs/` is the source of truth; `openspec/specs/` is deployed via
  `tools/deploy_specs.py` and drift-gated on pre-commit — never edit it
  by hand.

## Capability Map

| Spec | Capability |
|------|------------|
| `ge.graph_model` | Core model: nodes, edges, groups, class-scoped attributes |
| `ge.text_parser` | Graph::Easy text format → model |
| `ge.dot_parser` | Graphviz DOT subset → model (optional feature) |
| `ge.layout` | Deterministic rank/order/position + orthogonal edge routing |
| `ge.ascii_render` | Classic ASCII art output |
| `ge.boxart_render` | Unicode box-drawing output |
| `ge.html_render` | Table-based HTML output |
| `ge.graphviz_render` | DOT output (round-trips through `ge.dot_parser`) |
| `ge.txt_render` | Canonical text serialization (golden-test lingua franca) |
| `cli` | `gently` command-line pipeline, built on genesis-vibes |
| `ge.oracle` | Differential oracle gating against upstream Graph::Easy v0.69 (`just oracle-verify`) |
| `ge.perf` | Performance budgets: latency, memory, scaling (`just perf-check`) |
