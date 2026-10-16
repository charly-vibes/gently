# Changelog

All notable changes to gently are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-10-16

### Fixed

- **Fan-star routing family** (gently-n0d, gently-206) — labelled fan graphs
  (edges converging on or leaving one node) no longer panic or error:
  the `label_cells` subtract-with-overflow panic on zero-cell routed paths is
  gone, and the router now realizes valid orthogonal paths for the shapes
  that produced them — span-hosted west-going attachments, dodge columns
  between stacked nodes (no 180° backtrack), terminal-blocked walks closing
  to the face row, span rides honouring reserved gap rows, and east runs
  truncated at occupied cells with a face-adjacent approach. The ASCII
  renderer also classifies span west-side entries on continuation rows.
  Every edge of the specodelic corpus projection (337 edges, 333 labelled)
  now routes and renders in ascii and html; the pinned oracle renders are
  unchanged.

## [0.2.1] - 2026-10-16

### Fixed

- **`--help` no longer shows genesis library docs** — the flattened
  `CliVerbosity`/`CliFormat` args structs leaked their doc comments into
  the top of `gently --help` as mangled prose (clap flatten merges the
  child's doc-derived help; non-verbatim rustdoc collapses to a broken
  single-paragraph blob). Fixed upstream in genesis-vibes 0.13.2
  (genesis-hjy); the dependency pin is bumped from 0.12.1 and `--help`
  now leads with a one-line summary of the `--json`/`--human` flags.

## [0.2.0] - 2026-10-16

Second release: DOT input wired into the CLI, a layout-capability wave
(fan placement, indexed ports, multi-row stacking, grid-cell spans), and
the decision-trace spec corpus.

### Added

- **CLI accepts DOT input** — the DOT parser is wired into the CLI input
  path; a bare scope nested in a named subgraph joins the enclosing named
  group (gently-cij, gently-fb1).
- **Layout capability wave** — upstream's fan placement engine for the
  fan-in star (gently-a6g); indexed front/back port slots route, grow,
  and label byte-exactly (gently-byq); parallel channels stack as rows
  through multi-row boxes (gently-awd); `size: W,H` as a virtual
  attribute deriving columns/rows with txt emitting the merged size form
  (gently-awd slice 1); nodes occupy their grid-cell span via
  size/rows/columns spans (gently-awd); fan-stack vertical walks jog
  around occupied band rows (gently-4ga).
- **Decision-trace corpus** — D1–D4 hardened spec + contracts
  (gently-i6a); `trace.rs` promoted to the D1 layer with versioned stage
  vocabulary (gently-p7y); trace-diff minimal-antichain driver + EXCL-001
  corpus property (gently-9nn).
- **Repro tooling** — corpus-wide oracle dump mode with a tier-2
  admission guard (gently-h1b); oracle `_prepare_layout` dump pilot
  (gently-8je); env-gated decision-trace prototype + E1 driver
  (gently-asc).
- **CLI fail-fast guard** — terminal stdin with no file argument fails
  fast with a diagnostic naming stdin and the input file instead of
  blocking on an interactive read (gently-in2).

### Fixed

- **Pre-push cargo-gate hang under lefthook** — the pipeline unit test
  read the process's global stdin to EOF, which is an open TTY when run
  via `git push`; the reader is now injected so tests are hermetic
  (gently-ktv).
- BSD-compatible `script` invocation in the cli.p9 pty test for macOS
  runners.
- bd pre-push hook closes stdin (bd blocks waiting for EOF on the pipe
  git/lefthook keeps open); spec-drift-gate failure output names the
  remedy (gently-cgw).
- Restored the cli.p9 integration test lost from an earlier rebase.

## [Unreleased]

## [0.1.0] - 2026-10-13

First published release: the complete initial capability wave.

### Added

- **Compliance wave (ddl-standard parity)** — LICENSE (GPL-2.0-or-later) +
  matching workspace `license` field; ddl-standard justfile utility recipes
  (`install`, `build`, `build-release`, `run`, `clean`, `check`, `deps`,
  `update`, `test-verbose`, `test-one`, `tools`); rustfmt gate (`just fmt` /
  `fmt-check`, CI fmt-check step, lefthook fmt-check); lefthook bd hook chain
  + `pretender.toml` with ddl-convention advisory lease; this CHANGELOG;
  README CI + license badges; `release.yml` and `smoke.yml` workflows with
  crates.io + Release badges.

- **Capability wave (Graph::Easy port)** — graph model (nodes/edges/groups),
  Graph::Easy text parser and DOT parser, layout engine, and renderers for
  ascii, boxart, html, graphviz, and txt output — gated differentially
  against the original Perl tool via a pinned oracle (Graph::Easy v0.69,
  `just oracle-verify`), with performance budget gates (`just perf-check`,
  ge.perf), byte-verified published examples (`just examples-verify`), and a
  byte-verified visual gallery (`just gallery-verify`).

### Changed

- **License corrected from declared-MIT to GPL-2.0-or-later** — gently is a
  derivative work of Graph::Easy (which is GPL-2.0-or-later); the previous
  `license = "MIT"` declaration without a LICENSE file was not compliant.
  Divergence from dulce-de-leche's Apache-2.0 fleet standard is forced by
  upstream.
