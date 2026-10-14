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

## Purpose

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
| c4 | invariant | Every edge is routed along an orthogonal path of grid cells connecting its source port to its target port, passing through its label cell when the edge has a label; parallel edges between the same node pair are routed through distinct cells. Port-constrained edges (gently-89r) route from/to the NAMED side in the east pipeline: a `start:`/`end:` side south of its node (`right` = inflow+90 = south, upstream _flow_as_side; absolute `south`/`down` spellings alike) dips the edge through the gap row below the band — exiting the source's south, running straight under everything between, entering the target's south (`^`) — and a north side (`left`/`up`) bends above it (`v` into the target's north); `front` keeps the default attachment; the port never changes placement (tests/repro/claims/port-side-minimal.observed: the no-port bypass chain lays Trier inline exactly like the ported one). Indexed ports (`front,N`) and the `back` (west) wrap stay out of scope — they need geometry beyond the flat 3-line grid (variable node heights / an out-of-bounding-box run; docs/known-hard-cases.md, bead gently-byq for the index form) — and conflicting end sides, parallel channels, self-loops, cross-band routes, and the non-east flows keep their unported routes (no recorded shape ports them). The east placement pass processes edges in upstream's chain order (gently-pua, re-derived from Layout.pm's chain tracking and realized action stack): chains start at the roots (nodes never appearing as an edge target) in name order and walk along the flow — a `<>` edge counts from both ends, a plain or `--` edge only from its stored source — extending through each walk's unique unchained successor; a walk that runs into an already-chained node absorbs that chain's suffix (the later root's chain re-owns the shared nodes outright, upstream's merge), and a walk with several successors recurses first and absorbs the longest successor chain (first-in-model-order tie-break). Chains are laid out root-anchored first, then longest first, then start-name; each chain's layout claims its unclaimed members in order (the walk's opener edge — preferring an edge to an already-claimed neighbor — then the consecutive-link edges) and splices in the not-yet-laid chains its members' leftover links reach (upstream's inter-chain splice), so every node is placed by the first chain to claim it and a chain whose members were all absorbed re-ranks as a leftover leaf attaching to its placed successor west, east, else one band below — the box chain Bonn–train–Ulm lands inline on the flow row (Bonn west of the edge cell via the undirected `--`, Ulm east via the directed `-->`) and the anon buy leaf ranks one band below train (tests/repro/claims/bg-edge-node.observed, freshly re-derived: Bonn(0,2), train(2,2), Ulm(4,2), anon(2,4)); never-claimed leftovers keep model order, and the placement stays a pure function (c1). The pass also ranks against the graph's longest-path depths (gently-jrr): when a node's rank sits one column beyond the source's immediate east slot — the shared-middle bypass shape (Koblenz -> Trier -> Frankfurt forces Frankfurt east of Trier even when the Koblenz -> Frankfurt skip-edge would claim the east slot first; the chain order emits Trier -> Frankfurt before the skip-edge, and the rank rule guards any remaining inversion) — the node takes that rank column, leaving the middle node its inline slot on the band, and the skip-edge (a same-band non-adjacent route) bends through the gap row above the band (tests/repro/claims/bg-minlen-bypass.observed, byte-identical with and without the tutorial's incidental `minlen: 3`). Re-derived (gently-cbk): a west-going multi-band route bends out of the source's north/south FIRST — vertically through the gap rows at the source column — then runs along the gap row adjacent to the target band, ending at the target column; the source-band walk would drive through the intervening node cells (bg-edge-node.observed: Ulm->car exits Ulm north, turns west through the gap row below car's band, and its merged full-width dash cell ends at car's west edge). This is a within-envelope contract: at larger scales the pinned oracle itself silently drops edge paths ("could only place ... - giving up", Layout.pm:846; tests/repro/claims/layout-edge-drop.observed: 74 of 75 edges placed at 50 nodes), so large-graph edge-count fidelity is oracle-scoped-off ([[ge.oracle]] c7) and gently's own invariant — every model edge gets a routed path — is verified structurally instead. Re-derived (gently-4ga): both multi-band vertical walks — east-going down/up the target column, west-going out of the source's north/south (gently-cbk) — cover EVERY row between their endpoints at the walked column, not just the reserved gap rows, and jog around a band row occupied by a stacked node through the free gap column beside it: the jog happens on the free row before the blocked one and shifts back on the first free row after, east of the walked column unless that column sits at the grid's east edge (a stacked column always has a node one column east — placement only stacks below when the east slot is taken — and odd columns never hold nodes), so the realized path stays grid-adjacent end to end — source cell to first path cell to last path cell to target cell — and never enters a node cell (tests/scenarios/ge_layout.rs p4_fan_stack_routes_jog_not_jump: the comma-chain fan `[ A ] -> [ B ], [ C ], [ D ]` and the n1..n4 -> hub star render instead of erroring with "path cells not orthogonally adjacent"). The exact oracle fan PLACEMENT (upstream's multi-celled _near_places slot rings over grown node clusters plus the repair passes — hub grown to two east-side attachments, the second source slotting below the first east attachment) stays out of scope: the fan-out class is hash-seed-dependent under the pinned oracle (seed 2 reorders the flanking children; same class as the pod-fan probe) and the hash-stable fan-in star needs that placement engine — recorded as a residual in docs/known-hard-cases.md. Re-derived (gently-awd): a size-attributed node is MULTI-celled — it occupies its whole `columns`×`rows` grid-cell span from its placement cell, so the rank→column map accumulates each same-band westward node's extra columns (`col(rank, band) = 2·rank + Σ(cx(n) − 1)` over the band's smaller-rank nodes — upstream places at absolute grid coordinates, `col(r) = col(r-1) + cx(r-1) + 1` when the band's ranks are contiguous), the bands below a multi-row node shift south over its row-span continuation rows (empty ones render 0-tall), a straight edge routes through the single gap column EAST of its source's span (or WEST of a target's, entering at the target's east border `col + cx − 1`), the routing walks treat every spanned cell as occupied, and the grid width extends over the eastmost span end (tests/repro/claims/size-attr.observed, tests/fixtures/graph-easy/size-attr.ascii.expected). Re-derived (gently-awd completion): parallel channels between a MULTI-ROW pair stack as grid ROWS through the boxes' row span — the pair's first edge keeps the straight band-row run and channel k rides the grid row `band + k` (a straight gap-column run, single path cell; a non-adjacent pair bends through the stacked row), while channels beyond the row span and one-row pairs keep the recorded bend shape (parallel.txt: channel 1 bends above the band); the channel rows carry edge cells, so the box grows to its full row span (4 + 3 + 3 = the flowchart's 10 rows). The channel ORDER stays model order (ge.layout.c1): the pinned oracle's channel order is hash-driven (seed 0 assigns the pair's second model edge to the band row, seed 1 the first — tests/repro/claims/so-flowchart-size.observed pins both), and gently's deterministic order byte-matches the seed-1 render — the same tier-2 hash-dependence class as the fan residuals (docs/known-hard-cases.md). | [[ge.layout]] | |
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
| p4 | unit | [[ge.layout.c4]] | within-envelope graphs with labelled, self-loop, and parallel multi-edges, plus the stacked-column fans (comma-chain fan-out, fan-in star), the size-span chain (gently-awd), and the multi-row flowchart's stacked channels (gently-awd completion) | each edge's routed path is orthogonal, connected, passes through its label, and parallel edges never share a path cell; the fans' stacked-column walks jog around occupied band rows (gently-4ga) and stay grid-adjacent source cell to target cell; a size-attributed source's straight edge routes through the single gap column east of its span and enters no spanned cell (gently-awd); a multi-row pair's parallel channels ride distinct rows of the boxes' row span — channel k's single gap cell sits on grid row `band + k` — while one-row pairs keep the bend shape (gently-awd completion); structurally (any scale): every model edge receives a routed path |
| p5 | unit | [[ge.layout.c5]] | renderer-style consumers reading the grid output contract | every consumer read matches the layout's own inspection of grid, extents, and paths |
| p6 | unit | [[ge.layout.c6]] | large arbitrary graphs with dense crossing regions | each stage's running time stays within its polynomial bound |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** arbitrary graphs laid out twice under identical options — oracle-comparable sizes within the recorded envelope, gently-only (deterministic re-render) beyond it
- **THEN** both runs produce byte-identical grid outputs
- **VERIFIES** [[ge.layout.p1]]

#### Scenario: p2

- **WHEN** arbitrary graphs including dense and wide ones
- **THEN** the produced grid contains no node cell overlap
- **VERIFIES** [[ge.layout.p2]]

#### Scenario: p3

- **WHEN** acyclic graphs with a uniform graph-level flow across all four directions and self-loops, excluding the corpus-pinned shared-target/diamond shapes and any per-edge flow override
- **THEN** every edge's source precedes its target along the flow axis, self-loops and the exempted shapes excepted
- **VERIFIES** [[ge.layout.p3]]

#### Scenario: p4

- **WHEN** within-envelope graphs with labelled, self-loop, and parallel multi-edges, plus the stacked-column fans (comma-chain fan-out, fan-in star), the size-span chain (gently-awd), and the multi-row flowchart's stacked channels (gently-awd completion)
- **THEN** each edge's routed path is orthogonal, connected, passes through its label, and parallel edges never share a path cell; the fans' stacked-column walks jog around occupied band rows (gently-4ga) and stay grid-adjacent source cell to target cell; a size-attributed source's straight edge routes through the single gap column east of its span and enters no spanned cell (gently-awd); a multi-row pair's parallel channels ride distinct rows of the boxes' row span — channel k's single gap cell sits on grid row `band + k` — while one-row pairs keep the bend shape (gently-awd completion); structurally (any scale): every model edge receives a routed path
- **VERIFIES** [[ge.layout.p4]]

#### Scenario: p5

- **WHEN** renderer-style consumers reading the grid output contract
- **THEN** every consumer read matches the layout's own inspection of grid, extents, and paths
- **VERIFIES** [[ge.layout.p5]]

#### Scenario: p6

- **WHEN** large arbitrary graphs with dense crossing regions
- **THEN** each stage's running time stays within its polynomial bound
- **VERIFIES** [[ge.layout.p6]]

