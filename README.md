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
just examples-verify # published examples stay byte-identical to expected output
```

## Quickstart

Build the CLI and render a graph to ASCII art — the Graph::Easy text
format goes in on stdin, the layout comes out:

```sh
cargo build -p gently-cli
echo '[ Bonn ] --> [ Berlin ] --> [ Hamburg ]' | target/debug/gently --as ascii
```

```
+------+     +--------+     +---------+
| Bonn | --> | Berlin | --> | Hamburg |
+------+     +--------+     +---------+
```

`--as` selects the output format (so does the output filename's
extension, as in upstream). Rendering from a file argument instead of
stdin:

```sh
target/debug/gently --as ascii examples/simple-chain/input.gv
```

Other upstream formats (boxart, HTML, Graphviz DOT) land in later slices
— the CLI advertises `valid formats: ascii` until their renderers are
wired.

More worked examples (chain, edge styles, diamonds, self-loops, …) with
committed, byte-verified output live in [`examples/`](examples/README.md).
Where gently still differs from the pinned Graph::Easy oracle — and which
classes are deliberately outside byte-compat — see
[docs/known-hard-cases.md](docs/known-hard-cases.md).
