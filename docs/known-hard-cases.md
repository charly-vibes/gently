# Known hard cases — compatibility status

gently is gated byte-identically against the pinned Graph::Easy oracle
(**v0.69** @ `ededa3d7…`). The cases below are where the two currently
differ. This is a *compatibility status page*, not a to-do list: each row
is tracked in the issue tracker ([beads](https://github.com/gastownhall/beads)), and gently is deliberately
conservative — an example is only published in [`examples/`](../examples/)
when it is byte-identical to the oracle.

## Divergences (open)

### ASCII rendering

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-4ga` | RESOLVED for the render error (comma-chain fans and fan-in stars route contiguously and render — the stacked-column walk jogs around occupied band rows, specs/ge-layout.md c4 re-derived). Residual: the oracle's exact fan PLACEMENT differs (bytes below). |

**`gently-4ga`** — residual (exact fan placement; the render error itself is
fixed — fans render, routes grid-adjacent):

input (`bg-comma-lists.observed`, example 1):

```
[ A ] -> [ B ], [ C ], [ D ]
```

gently (deterministic; the fan-in star n1..n4 -> hub renders with the same
jogged-walk shape):

```
+---+     +---+
| A | --> | B |
+---+     +---+
  |
  +----+
  v    |
+---+  |
| C |  |
+---+  |
       |
  +----+
  v
+---+
| D |
+---+
```

oracle — the fan-out class is HASH-SEED-DEPENDENT (seed 2 reorders the
flanking children, same class as the pod-fan probe), so its exact bytes
are oracle-scoped-off ([[ge.oracle]] c7 tier 2); every seed flanks two
children east/west of the source and hangs one below:

```
+---+     +---+     +---+
| C | <-- | A | --> | D |
+---+     +---+     +---+
            |
            |
            v
          +---+
          | B |
          +---+
```

The hash-STABLE fan-in star (n1..n4 -> hub, 12 seeds) would be tier-1, but
matching it needs upstream's multi-celled `_near_places` slot rings over
grown node clusters plus the repair passes (hub grows two east-side
attachment rows; the second source slots below the first east attachment)
— a placement-engine port, not a router fix. gently renders the same star
deterministically with the jogged-walk shape instead.

### Text parser

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-58m` | Group-endpoint edges from the anonymous-group syntax `( [ Bonn ] ) -> [ Berlin ]`: the oracle's `as_txt` omits the group-only-connected node, gently lists Berlin as a plain node (bytes below). The parse itself is fixed (`gently-m4i`). |

**`gently-58m`** — input and the divergence side-by-side:

input (`pod-anon-group.observed`):

```
( [ Bonn ] ) -> [ Berlin ]

( [ Bonn ] { color: red; } )
```

oracle `as_txt` — Berlin is omitted (only reachable through the dropped
group edge):

```
[ Bonn ] { color: red; }

( )

(
  [ Bonn ]
)
```

gently — same parse, but `as_txt` lists Berlin as a plain node (the
edge-linked anon group renders empty as `( Group \#0 )`):

```
( Group \#0 )

( Group \#1
  [ Bonn ] { color: red; }
)

[ Berlin ]
```

(The `as_ascii` bytes for both — Bonn and Berlin as plain boxes — are in
`pod-anon-group.observed`.)

### Dot parser

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-fb1` | Bare scope nested in a named subgraph (`digraph G { subgraph outer { { x } } }`): the oracle's group_stack survives bare-scope boundaries, so x lands IN group `outer` (bytes below); gently keeps x ungrouped. Only diverges inside a named subgraph — top-level bare scopes are ungrouped in both (pinned p4); fix shape: track the enclosing named group separately from the bare-scope frame. |

**`gently-fb1`** — input and the divergence side-by-side:

input (`dot-bare-scope-in-named.observed`):

```
digraph G { subgraph outer { { x } } }
```

oracle — `groups: outer=[x]`; x renders inside the dashed `outer` group
box (`as_ascii`):

```
+ - - - - +
'  outer  '
'         '
' +-----+ '
' |  x  | '
' +-----+ '
'         '
+ - - - - +
```

gently — keeps x ungrouped (no group emitted, no group box).

### Layout & edge attributes

| bead | behavior delta (oracle → gently) |
|------|----------------------------------|
| `gently-awd` | `{ size: 5,10; }` node attribute blocks (SO 47323345 flowchart): the oracle honors node sizes and lays Server/Msg Server/Client out 2x2 (bytes below), gently lays a 1-3-2 flow. RESOLVED SLICES (gently-awd, committed): (1) txt merges `size: W,H` (ge.txt_render c5/p5); (2) column spans — `size: W,H` / `columns` / `rows` stretch nodes over their grid-cell span (rank→col accumulation `2·rank + Σ(cx−1)`, per-column/row sizing from single-celled cells only, balance smallest-first to the label width, rows spans render 0-tall continuation rows) — the 5-chain size-attr fixture renders byte-identically (tests/fixtures/graph-easy/size-attr.*); (3) span-aware edge attachments (straight gap east/west of the span, bends entering/leaving through span columns). The flowchart's PLACEMENT now also matches the recorded oracle (Server col 0, Msg Server cols 2-6, Client cols 8-12, one band). REMAINING DELTA: parallel-edge CHANNEL routing — the oracle gives each parallel edge its own grid ROW below the band (the runs render as separate straight runs at distinct heights through the 10-line-tall boxes, and those edge rows grow Msg Server's box to its full 10-row span), while gently bends parallel channels through the gap rows above/below the band, leaving the continuation rows 0-tall (box stays 4 lines). Needs the upstream parallel-channel-as-rows mechanism — kin to gently-byq's indexed-port slots. |
| `gently-byq` | Indexed ports `{ start: front, N }`: the oracle honors the index and lands the edges at the indexed slots (bytes below), gently routes to the default slot. Unindexed side routing (`start: right` dips below, `left` bends above, `front` is the default) is PORTED byte-identically (`gently-89r`, `port-*` fixtures) — the index form needs variable node heights (the oracle stretches a 1-row box to 5 lines for a `front,1` slot) and the unprobed `back` (west) wrap runs outside the bounding box. |

**`gently-awd`** — input and the divergence side-by-side:

input (`so-flowchart-size.observed`):

```
[ Client ] { size: 5,10; }
[ Msg Server ] { size: 5,10; }

[ Server ] -- put msg\n via http --> [ Msg Server ]
[ Msg Server ] -- 1. Number X of unread --> [ Client ]
[ Msg Server ] -- 3. X Msg --> [ Client ]
[ Client ] -- 2. read X --> [ Msg Server ]
```

oracle — `{ size: 5,10; }` honored, taller/wider boxes, 2x2 layout
(`so-flowchart-size.observed`, `as_ascii`):

```
+--------+  put msg    +------------+                          +--------+
| Server |  via http   |            |  3. X Msg                |        |
|        | ----------> |            | -----------------------> |        |
+--------+             |            |                          |        |
                       | Msg Server |  1. Number X of unread   | Client |
                       |            | -----------------------> |        |
                       |            |                          |        |
                       |            |  2. read X               |        |
                       |            | <----------------------- |        |
                       +------------+                          +--------+
```

gently — same input after the span slices (placement + box widths match
the oracle; the parallel edges bend above/below instead of taking their
own rows, so Msg Server's box stays 4 lines instead of stretching to its
10-row span):

```
                         +---------------------------------------+
                         |                                       v
+--------+  put msg    +------------+                          +--------+
| Server |  via http   | Msg Server |  1. Number X of unread   | Client |
|        | ----------> |            | -----------------------> |        |
+--------+             +------------+                          +--------+
                         ^                                       |
                         +---------------------------------------+
```

**`gently-byq`** — input and the divergence side-by-side:

input (`so-start-port-index.observed`):

```
[ Msg Server ] -- 1. Number X of unread --> { start: front,0; } [ Client ]
[ Msg Server ] -- 3. X Msg --> { start: front,3; } [ Client ]
[ Client ] -- 2. read X --> { end: front,2; } [ Msg Server ]
```

oracle — edges land at the indexed ports; Msg Server stretched to 10
rows so `front,0` and `front,3` are distinct slots (`as_ascii`):

```
+------------+  1. Number X of unread   +-----------+
|            | -----------------------> |  Client   | -+
|            |                          +-----------+  |
|            |                            ^            |
|            |                            | 3. X Msg   |
| Msg Server |                            |            |
|            |  2. read X                 |            |
|            | <--------------------------+------------+
|            |                            |
|            |                            |
|            | ---------------------------+
+------------+
```

gently — routes all edges to the default slot (1-row node, wrap loop):

```
  +---------------------------------------+
  |                                       v
+------------+  1. Number X of unread   +--------+
| Msg Server | -----------------------> | Client |
+------------+                          +--------+
  ^                                       |
  +---------------------------------------+
```

The already-ported unindexed form renders byte-identically in both
(`port-side-minimal.observed`, `start: right` dips below the node row):

```
+---+     +---+
| A |     | B |
+---+     +---+
  |         ^
  +---------+
```

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
