# Known hard cases — compatibility status

gently is gated byte-identically against the pinned Graph::Easy oracle
(**v0.69** @ `ededa3d7…`). The cases below are where the two currently
differ. Each row tells you *when you'll see it* — if your graph doesn't
match one of these triggers, gently's output should be byte-identical to
the oracle's. Resolved divergences and verification internals live in the
maintainer history (`.wai/resources/maintainer-docs/compatibility-history.md`
in the repository).

## Open divergences

| you'll see this when | what differs (oracle → gently) | bead |
|----------------------|--------------------------------|------|
| you use an indexed port with the `back` direction, wrapping outside the bounding box (`{ end: back,N; }` on a west edge) | indexed `{ start,end: front,N }` ports render byte-identically; the `back` (west) wrap outside the bounding box does not yet | `gently-byq` (residual) |
| you print a graph as Graph::Easy text (`--as txt`) where an edge starts or ends at an anonymous group, e.g. `( [ Bonn ] ) -> [ Berlin ]` | the oracle's `as_txt` omits the group-only-connected node; gently lists it as a plain node. (The minimal variant is adjudicated as an oracle defect — see the maintainer history.) | `gently-58m` |
| you parse DOT with a bare scope nested inside a named subgraph, e.g. `digraph G { subgraph outer { { x } } }` | the oracle puts `x` inside group `outer` (dashed group box); gently keeps `x` ungrouped. Top-level bare scopes are ungrouped in both | `gently-fb1` |

## Byte-compat scope-off (deliberate, no divergence claims)

Per the byte-compat scope contract, three input classes are outside
byte-equality with the oracle by design:

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

## How this is verified

- `just oracle-verify` — differential gate over the pinned-oracle
  fixture corpus (`tests/fixtures/graph-easy/`)
- `just examples-verify` — every published [example](../examples/) stays
  byte-identical to its committed expected output

Each divergence is tracked in the issue tracker; the full resolved
history and the reproduction/adjudication pipeline are documented in the
maintainer history (`.wai/resources/maintainer-docs/compatibility-history.md`).
