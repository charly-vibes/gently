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
- **TDD / Tidy First**: every capability gets red→green→refactor cycles;
  refactors are separate changes from features (tracked in beads).
- **No implementation before approval**: openspec changes carry
  dual-format deltas; implementation happens only in the apply stage.

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
| `ge.cli` | `gently` command-line pipeline |
