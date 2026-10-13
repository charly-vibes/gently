# Changelog

All notable changes to gently are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
