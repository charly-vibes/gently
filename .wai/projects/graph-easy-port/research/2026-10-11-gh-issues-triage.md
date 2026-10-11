# GitHub issue mining triage — ironcamel/Graph-Easy (gently-ni9.3)

**Date:** 2026-10-11 · **Oracle pin:** Graph::Easy v0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499
**Scope:** all issues on `ironcamel/Graph-Easy` (state=all, per_page=100 — the tracker holds exactly 8). The older rt.cpan.org queue was unreachable from this environment (empty response); skipped per brief — GitHub is the target.

## Triage table

| Issue | State | Input shape | Oracle v0.69 verdict | gently verdict | Action |
|-------|-------|-------------|----------------------|----------------|--------|
| [#9](https://github.com/ironcamel/Graph-Easy/issues/9) | open | spam (Thai movie links) — no graph input | n/a | n/a | skip — not a bug report |
| [#7](https://github.com/ironcamel/Graph-Easy/issues/7) | open | Module::Signature verification failure — no graph input | n/a | n/a | skip — packaging/signing, not parse/render |
| [#6](https://github.com/ironcamel/Graph-Easy/issues/6) | open | macOS install failure — no graph input | n/a | n/a | skip — install environment, not parse/render |
| [#5](https://github.com/ironcamel/Graph-Easy/issues/5) | open | Module::Signature verification failure — no graph input | n/a | n/a | skip — packaging/signing, not parse/render |
| [#4](https://github.com/ironcamel/Graph-Easy/issues/4) | closed | **DOT**: nodes declared before `{foo bar} -> baz`, plus the no-declaration variant | **still reproduces, bug-for-bug**: input a renders the spurious self-edge `[ baz ] --> [ baz ]` and the reported wrong drawing; input b renders the correct 3-box row. Hash-stable across seeds 0/1. | **diverges at parse time**: both inputs fail with `expected a statement` — gently's dot parser has (i) no graph-scope attribute statement form (`rankdir="LR"`), (ii) no bare-scope-as-edge-endpoint form (`{foo bar} -> baz`); `digraph { { a } }` parses but `digraph { { a } -> b }` does not | probe `gh-4-nodes-first` (`.observed` evidence recorded); divergences filed as beads **gently-a0k** (graph-scope attrs) and **gently-2n5** (bare-scope endpoint) |
| [#3](https://github.com/ironcamel/Graph-Easy/issues/3) | open | feature request (color in as_ascii) — no graph input | n/a | n/a | skip — feature request; no reproducible input. Note: gently is byte-compat with the oracle, which does not emit color in as_ascii either |
| [#2](https://github.com/ironcamel/Graph-Easy/issues/2) | open | Module::Signature verification failure — no graph input | n/a | n/a | skip — packaging/signing, not parse/render |
| [#1](https://github.com/ironcamel/Graph-Easy/issues/1) | open | feature request (max columns for ascii output) — no graph input | n/a | n/a | skip — feature request; no reproducible input |

## Outcome summary

- **Issues triaged:** 8/8 (the tracker's full state=all set).
- **Reproducible graph inputs:** 1 issue (#4, two inputs).
- **Oracle verdict:** the reported bug **still reproduces against the pinned oracle** — the oracle is the byte-compat contract, so the spurious `[ baz ] --> [ baz ]` self-edge and the "wrong drawing" are the *correct* gently target, not a fix.
- **gently verdict:** diverges before rendering — both inputs are parse errors. Two divergence classes isolated:
  1. graph-scope attribute statements (`rankdir="LR"`) — bead `gently-a0k`;
  2. bare scope as an edge-chain endpoint (`{a b} -> c`) — bead `gently-2n5`.
- **Claim probe:** `gh-4-nodes-first` → `tests/repro/claims/gh-4-nodes-first.observed` (evidence recorded under the pin). Not a corpus fixture: tier-1 byte-admission is unreachable while gently cannot parse the input, and the probe comment records that.
- **Zero unverified claims published** — every verdict above is grounded in a run (oracle snippet runs for #4; the rest have no input to verify anything against).