# examples

Worked Graph::Easy text-format examples, each a directory with:

- `input.gv` — the graph source (Graph::Easy text format)
- `expected.txt` — the committed, byte-verified ASCII-art output

Every `expected.txt` is the recorded output of the pinned oracle
(**Graph::Easy v0.69** @ `ededa3d7…`, `PERL_HASH_SEED=0`) — mined from the
differential fixture corpus in `tests/fixtures/graph-easy/`, pin header
stripped — and `just examples-verify` re-renders each `input.gv` with the
gently CLI and fails on any byte mismatch. The oracle's ASCII art is also
gently's, so what you see in `expected.txt` is what the CLI prints.

| example | one-liner |
|---------|-----------|
| `simple-chain/` | the smallest useful graph — a linear chain `a --> b --> c` |
| `edge-styles/` | edge style attributes: solid `------>` and double `========>` arrows |
| `edge-attributes/` | inline edge attributes `{ style: bold; label: foo; }` on an edge |
| `diamond/` | branching and re-joining: fan-out from `a`, fan-in to `d` (the diamond layout) |
| `self-loop/` | an edge from a node back to itself |
| `parallel-edges/` | two separate edges between the same pair of nodes |
| `isolated-node/` | an isolated node laid out next to an edge-connected pair |

Build the CLI first (see the main README quickstart), then run any example
through it:

```sh
cargo build -p gently-cli
gently --as ascii < examples/simple-chain/input.gv
```

Known rendering divergences from the oracle are tracked separately — see
[docs/known-hard-cases.md](../docs/known-hard-cases.md). Examples are only
published for inputs where gently is byte-identical to the oracle.
