# gently

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)

A Rust port of Perl's [Graph::Easy](https://github.com/ironcamel/Graph-Easy):
build graphs (nodes/edges/groups), lay them out deterministically on a
flat surface, and render them as ASCII art, boxart, HTML, or Graphviz
DOT — plus parse graphs from the Graph::Easy text format and from DOT.

**Status: initial capability wave complete.** The full pipeline ships —
parse → model → layout → render (ASCII art, boxart, HTML, text, Graphviz
DOT) → CLI — with the capability corpus in `specs/` (Specodelic
four-layer format). Compatibility is gated differentially against the
original Perl tool via a pinned oracle (v0.69, `just oracle-verify`),
and performance via budget gates (`just perf-check`).

```bash
just lint            # all specs must be lint-clean
just gates           # compile + model-check the corpus
just oracle-verify   # differential compat gate (needs fixtures; spec ge.oracle)
just perf-check      # performance budgets (spec ge.perf)
```
