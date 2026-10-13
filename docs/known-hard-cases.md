# Known hard cases — compatibility status

gently is gated byte-identically against the pinned Graph::Easy oracle
(**v0.69** @ `ededa3d7…`). The cases below are where the two currently
differ. This is a *compatibility status page*, not a to-do list: each row
is tracked in the issue tracker (beads), and gently is deliberately
conservative — an example is only published in [`examples/`](../examples/)
when it is byte-identical to the oracle.

## Divergences (open)

### ASCII rendering

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-51d` | wide/broad edge styles — oracle renders `##>` double-line arrowhead, gently renders plain `-->` |
| `gently-527` | literal `\n` in an edge label — oracle splits into a two-row label, gently prints it raw |
| `gently-5vf` | vertical stem column — gently indents 2 spaces where the oracle uses 3 (SO 71860) |
| `gently-7qz` | `<--` bidirectional edges — oracle draws both arrowheads and places inline labels above, gently drops the start arrowhead and inlines the label |
| `gently-7rx` | node groups — oracle draws the dashed `+ - - -+` border box with the group title, gently draws no border |
| `gently-b0z` | anonymous nodes — oracle draws them boxless/unnamed, gently draws boxed `#N` |
| `gently-bpq` | arrowless `--` edges — oracle draws no arrowhead, gently draws a target arrowhead |
| `gently-bvb` | Unicode box width — oracle counts UTF-8 bytes (`Köln` box is 1 column wider), gently counts characters |
| `gently-cbk` | `shape: edge` nodes — gently errors with "edge path cells are not orthogonally adjacent"; oracle renders the node along the edge path |
| `gently-klf` | interlinked triangle layout — gently's back-edge drifts 2 columns from the oracle drawing |

### Text parser

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-58m` | anonymous-group syntax `( [ Bonn ] ) -> [ Berlin ]` — oracle parses, gently is a parse error |
| `gently-8xd` | trailing node attribute block (`{ label: "Huge"; }` after the name) — oracle applies it, gently silently drops it |
| `gently-m4i` | comma-chained node lists (`[ A ], [ B ] --> [ C ]`) — oracle parses, gently is a parse error |
| `gently-y7i` | `\l` `\r` `\c` label-alignment escapes — oracle aligns, gently renders them raw |

### DOT parser

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-2n5` | bare scope `{a b}` as an edge-chain endpoint — oracle parses, gently is a parse error |
| `gently-a0k` | graph-scope attribute statements (`rankdir=LR`) — oracle parses, gently is a parse error |

### Layout & edge attributes

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-89r` | `start:`/`end:` port attributes — oracle routes edges from the named port, gently ignores them |
| `gently-byq` | indexed ports `{ start: front, N }` — oracle honors the index, gently ignores it (SO 47323345) |
| `gently-emd` | `textwrap: auto` / `textwrap` edge attribute — oracle wraps labels, gently ignores it |
| `gently-yhg` | `minlen` edge attribute — oracle enforces the minimum path length, gently ignores it |

## Byte-compat scope-off (deliberate, no divergence claims)

Per the byte-compat scope contract (`specs/ge-oracle.md` c7/p7), three
input classes are outside byte-equality with the oracle by design:

- **Hash-randomized inputs.** Upstream renders some inputs differently
  across Perl hash seeds even under a pinned seed; those classes (e.g.
  the POD edge-style block, the three-node fan) are verified as claim
  probes, never as byte fixtures.
- **Beyond-envelope sizes.** The oracle needs superlinear time (~10 s at
  200 nodes), so large-graph behavior (e.g. 1000 nodes) is verified
  against gently's own structural invariants and performance budgets,
  not oracle bytes.
- **Round-trip gaps the oracle itself has.** `as_graphviz` always
  directs edges, named groups round-trip as `cluster0..N`, and
  group-endpoint edges crash upstream's `as_graphviz` — byte-compat on
  these is unsatisfiable by construction, so round-trip claims are
  worded per upstream behavior instead.

## Verification layers

- `just oracle-verify` — differential gate over the fixture corpus
  (pinned oracle, `tests/fixtures/graph-easy/`)
- `just examples-verify` — the published examples in `examples/` must
  stay byte-identical to their committed expected output
- `tests/repro/` — falsifiable probe harness (`probes.pl`) computing
  tier-1 admission and recording oracle-observed behavior for the
  divergences above
