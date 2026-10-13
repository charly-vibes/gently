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
| `gently-4ga` | Comma-chain fan layouts (`[ A ] -> [ B ], [ C ], [ D ]`): the oracle renders the fan fine, gently errors on the fan render (bytes below). Parser + model side is done and pinned (`gently-m4i` p11); only the fan render is open — sibling of `gently-cbk` (RESOLVED). |

**`gently-4ga`** — input and the divergence side-by-side:

input (`bg-comma-lists.observed`, example 1):

```
[ A ] -> [ B ], [ C ], [ D ]
```

gently — render error (reproducible: `gently file.txt --as ascii`):

```
gently: render error: edge 2 path cells (0, 1) -> (0, 3) are not orthogonally adjacent
```

oracle — the comma-list fan renders fine (`bg-comma-lists.observed`, `as_ascii`):

```
       +-------------------+
       |                   v
     +---+     +---+     +---+
     |   | --> | C | --> |   |
     |   |     +---+     |   |
     |   |       ^       |   |
     | A | ------+       | D |
     |   |               |   |
     |   |               |   |
  +- |   | ------------> |   |
  |  +---+               +---+
  |    |                   ^
  |    |                   |
  |    v                   |
  |  +---+                 |
  +> | B | ----------------+
     +---+
```

Three edges leaving `A` land in one row with a 2-column gap the polyline
cannot fill.

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
| `gently-awd` | `{ size: 5,10; }` node attribute blocks (SO 47323345 flowchart): the oracle honors node sizes and lays Server/Msg Server/Client out 2x2 (bytes below), gently lays a 1-3-2 flow. Variable band heights now EXIST in the renderer (`gently-527`: per-row heights + vertically centered labels); the size-attr placement/ranking is the remaining delta. |
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

gently — same input, 1-3-2 flow (sizes not placed):

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
