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
| `gently-4ga` | comma-chain fan layouts (`[ A ] -> [ B ], [ C ], [ D ]`, gently-m4i parser p11) — the oracle renders the fan fine (`bg-comma-lists.observed` as_ascii); gently errors `edge 2 path cells (0, 1) -> (0, 3) are not orthogonally adjacent` — three edges leaving A land in one row with a 2-column gap the polyline cannot fill. Parser + model side is done and pinned; only the fan render is open. Sibling of gently-cbk (RESOLVED). |

### Text parser

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-58m` | group-endpoint edges from the anonymous-group syntax `( [ Bonn ] ) -> [ Berlin ]` — the oracle's as_txt omits the group-only-connected node (Berlin is only reachable through the dropped group edge), gently lists it as a plain node (the parse itself is fixed, gently-m4i) |

### Dot parser

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-fb1` | bare scope nested in a named subgraph (`digraph G { subgraph outer { { x } } }`) — the oracle's group_stack survives bare-scope boundaries, so x lands IN group `outer` (GROUP(outer: x)); gently keeps x ungrouped (only diverges inside a named subgraph — top-level bare scopes are ungrouped in both, pinned p4). Fix shape: track the enclosing named group separately from the bare-scope frame. |

### Layout & edge attributes

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-awd` | `{ size: 5,10; }` node attribute blocks (SO 47323345 flowchart) — the oracle honors node sizes (taller/wider boxes, comma-in-attribute-block form parses) and lays Server/Msg Server/Client out 2x2; gently lays a 1-3-2 flow (`so-flowchart-size.observed`). Variable band heights now EXIST in the renderer (gently-527: per-row heights + vertically centered labels), unblocking the node-stretching part; the size-attr placement/ranking is the remaining delta. |
| `gently-byq` | indexed ports `{ start: front, N }` — oracle honors the index, gently routes to the default slot; unindexed side routing (`start: right` dips below, `left` bends above, `front` is the default) is PORTED byte-identically (gently-89r, port-* fixtures) — the index form needs variable node heights (the oracle stretches a 1-row box to 5 lines for a `front,1` slot) and the unprobed `back` (west) wrap runs outside the bounding box |

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
