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
| `gently-cbk` | `shape: edge` nodes — gently errors with "edge path cells are not orthogonally adjacent"; oracle renders the node along the edge path |

### Text parser

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-58m` | group-endpoint edges from the anonymous-group syntax `( [ Bonn ] ) -> [ Berlin ]` — the oracle's as_txt omits the group-only-connected node (Berlin is only reachable through the dropped group edge), gently lists it as a plain node (the parse itself is fixed, gently-m4i) |

### Layout & edge attributes

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-byq` | indexed ports `{ start: front, N }` — oracle honors the index, gently routes to the default slot; unindexed side routing (`start: right` dips below, `left` bends above, `front` is the default) is PORTED byte-identically (gently-89r, port-* fixtures) — the index form needs variable node heights (the oracle stretches a 1-row box to 5 lines for a `front,1` slot) and the unprobed `back` (west) wrap runs outside the bounding box |
| `gently-bg-minlen-bypass-shape` | shared-middle bypass shape (`Koblenz -> Trier -> Frankfurt` beside a skip-edge to Frankfurt) — oracle ranks Trier inline and routes the skip-edge over the top, gently attaches Trier below; RE-DERIVATION (gently-klf run): renders byte-identically WITHOUT `minlen: 3`, so the tutorial's minlen is incidental — this is a ranking divergence, not a minlen one (tests/repro/claims/bg-minlen-bypass.observed) |

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
