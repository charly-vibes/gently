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

Every example also appears as a visual gallery entry — CLI ASCII art plus an
SVG rendering from gently's DOT output — in
[`docs/gallery.md`](../docs/gallery.md).

Build the CLI first (see the main README quickstart), then run any example
through it:

```sh
cargo build -p gently-cli
gently --as ascii < examples/simple-chain/input.gv
```

Known rendering divergences from the oracle are tracked separately — see
[docs/known-hard-cases.md](../docs/known-hard-cases.md). Examples are only
published for inputs where gently is byte-identical to the oracle.

## Adding an example

Every example is byte-verified against the pinned oracle before it is
published. The workflow (guided first, then explore):

1. **Mine the oracle bytes** — add your input as
   `tests/fixtures/graph-easy/<name>.txt`, then run `just oracle-record`
   (records with Graph::Easy v0.69, `PERL_HASH_SEED=0`; see
   `tests/fixtures/graph-easy/README.md` for the recording setup).
2. **Commit the example** — create `examples/<name>/` containing
   `input.gv` (your graph source) and `expected.txt` = the recorded
   `<name>.ascii.expected` companion with its leading `# oracle: ` pin
   header stripped (see the README paragraph above for what `expected.txt`
   is).
3. **Verify** — run `just examples-verify`; it re-renders every
   `input.gv` with the gently CLI and fails on any byte mismatch. Publish
   the example only when this is green (byte-identical).
4. **If it is NOT byte-identical** — do not publish an example; instead add
   a divergence row to [docs/known-hard-cases.md](../docs/known-hard-cases.md)
   (the admission rule) and track it in the issue tracker.

For further exploration: the fixture corpus lives in
`tests/fixtures/graph-easy/` (with its own README on recording and pin
headers), and `just repro-claims` lists the claim probes behind the gates.
