> *"Everything is connected!"*
> — Dirk Gently

# Introduction

**gently** is a Rust port of Perl's
[Graph::Easy](https://github.com/ironcamel/Graph-Easy): build graphs
(nodes, edges, groups), lay them out deterministically on a flat surface,
and render them as ASCII art, HTML, or Graphviz DOT — plus parse graphs
from the Graph::Easy text format and from DOT.

Compatibility is gated differentially against the original Perl tool via
a pinned oracle (Graph::Easy v0.69, `just oracle-verify`): an output only
ships when it is byte-identical to what the oracle produces. Where gently
still differs, the [known hard cases](known-hard-cases.md) page is the
authoritative status.

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

## Formats

| Stage                   | Formats                                  |
|-------------------------|------------------------------------------|
| Parse                   | Graph::Easy text, Graphviz DOT           |
| Render via CLI (`--as`) | `ascii`, `html`, `txt`, `dot`/`graphviz` |
| Render via library      | boxart (CLI wiring pending)              |

Asking for an unwired format (e.g. `--as boxart`, `--as svg`) errors with
`gently: unknown output format` and exit 255 until a renderer is wired to
the CLI.

## Where to go next

- **[Examples](examples.md)** — worked, byte-verified input/output pairs.
- **[Gallery](gallery.md)** — every example as ASCII art *and* SVG.
- **[Release status](release.md)** — version and commit this book was
  built from (generated; the repository is authoritative).
- The [repository README](https://github.com/charly-vibes/gently) carries
  the contributor gates and the full compatibility methodology.
