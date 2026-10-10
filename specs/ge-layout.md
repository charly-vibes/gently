---
id: ge.layout
kind: intent
statement: "WHEN a graph model is laid out THE layout SHALL assign ranks, orders, and grid coordinates deterministically and route every edge along an orthogonal path without overlapping node cells."
---

# layout

The layout engine — gently's port of Graph::Easy::Layout and its
rank/order/position/path submodules. It consumes [[ge.graph_model]]
through the published model contract and produces a deterministic grid
of cells that the renderers consume. Flow direction, node and edge
labels, and group placement all influence the result.

## Constraints

| id | kind | expr | traces_to | satisfies |
|----|------|------|-----------|-----------|
| c1 | invariant | The layout is a pure function of the graph model and the layout options: identical inputs always produce identical ranks, orders, coordinates, and edge paths. Verification is tiered (the [[ge.oracle]] c7 pattern): byte-comparable against the pinned oracle within the recorded envelope, and gently-only beyond it — deterministic re-render of gently's own output — because the oracle arms a default 5-second `alarm()` (Layout.pm:481) and cannot witness any claim at out-of-envelope scales (tests/repro/claims/size-envelope.observed: 200 nodes → "layout did not finish in time"). | [[ge.layout]] | |
| c2 | invariant | In the final grid, no two node cells overlap and every node is fully contained inside the grid. | [[ge.layout]] | [[ge.graph_model.c5]] |
| c3 | invariant | For acyclic graphs under a uniform graph-level flow, the configured flow direction (down, up, left, right) maps source nodes to strictly earlier positions along the flow axis than their targets; self-loops are exempt (source and target coincide), and the packed shared-target/diamond shapes the recorded corpus pins rank-equal are exempt (tests/scenarios/ge_layout.rs scope note). Cycles and per-edge flow overrides are documented divergences: the oracle routes back-edges and overridden edges AGAINST the flow (tests/repro/claims/layout-flow-direction.observed), so no flow-consistent reading of the unqualified claim exists; gently's placement follows the recorded corpus. | [[ge.layout]] | |
| c4 | invariant | Every edge is routed along an orthogonal path of grid cells connecting its source port to its target port, passing through its label cell when the edge has a label; parallel edges between the same node pair are routed through distinct cells. This is a within-envelope contract: at larger scales the pinned oracle itself silently drops edge paths ("could only place ... - giving up", Layout.pm:846; tests/repro/claims/layout-edge-drop.observed: 74 of 75 edges placed at 50 nodes), so large-graph edge-count fidelity is oracle-scoped-off ([[ge.oracle]] c7) and gently's own invariant — every model edge gets a routed path — is verified structurally instead. | [[ge.layout]] | |
| c5 | extension_point | The layout publishes its grid-output contract — cell grid, node extents, edge cell paths — which [[ge.ascii_render]], [[ge.boxart_render]], and [[ge.html_render]] consume via `satisfies`. | [[ge.layout]] | |
| c6 | advisory | For typical inputs the layout completes without exponential blowup: ranking, ordering, and routing run as bounded, deterministic heuristic iterations over the graph. | [[ge.layout]] | |

## Model

### States

- `initial`
- `ranked`
- `ordered`
- `positioned`
- `routed`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | initial | ranked | [[ge.layout.c1]] |
| t2 | ranked | ordered | [[ge.layout.c3]] |
| t3 | ordered | positioned | [[ge.layout.c2]] |
| t4 | positioned | routed | [[ge.layout.c4]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.layout.c1]] | arbitrary graphs laid out twice under identical options — oracle-comparable sizes within the recorded envelope, gently-only (deterministic re-render) beyond it | both runs produce byte-identical grid outputs |
| p2 | unit | [[ge.layout.c2]] | arbitrary graphs including dense and wide ones | the produced grid contains no node cell overlap |
| p3 | unit | [[ge.layout.c3]] | acyclic graphs with a uniform graph-level flow across all four directions and self-loops, excluding the corpus-pinned shared-target/diamond shapes and any per-edge flow override | every edge's source precedes its target along the flow axis, self-loops and the exempted shapes excepted |
| p4 | unit | [[ge.layout.c4]] | within-envelope graphs with labelled, self-loop, and parallel multi-edges | each edge's routed path is orthogonal, connected, passes through its label, and parallel edges never share a path cell; structurally (any scale): every model edge receives a routed path |
| p5 | unit | [[ge.layout.c5]] | renderer-style consumers reading the grid output contract | every consumer read matches the layout's own inspection of grid, extents, and paths |
| p6 | unit | [[ge.layout.c6]] | large arbitrary graphs with dense crossing regions | each stage's running time stays within its polynomial bound |
