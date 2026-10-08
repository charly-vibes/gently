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
| c1 | invariant | The layout is a pure function of the graph model and the layout options: identical inputs always produce identical ranks, orders, coordinates, and edge paths. | [[ge.layout]] | |
| c2 | invariant | In the final grid, no two node cells overlap and every node is fully contained inside the grid. | [[ge.layout]] | [[ge.graph_model.c5]] |
| c3 | invariant | The configured flow direction (down, up, left, right) maps source nodes to strictly earlier positions along the flow axis than their targets; self-loops are exempt, since source and target coincide. | [[ge.layout]] | |
| c4 | invariant | Every edge is routed along an orthogonal path of grid cells connecting its source port to its target port, passing through its label cell when the edge has a label; parallel edges between the same node pair are routed through distinct cells. | [[ge.layout]] | |
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
| p1 | unit | [[ge.layout.c1]] | arbitrary graphs laid out twice under identical options | both runs produce byte-identical grid outputs |
| p2 | unit | [[ge.layout.c2]] | arbitrary graphs including dense and wide ones | the produced grid contains no node cell overlap |
| p3 | unit | [[ge.layout.c3]] | graphs with all four flow directions and self-loops | every edge's source precedes its target along the flow axis, self-loops excepted |
| p4 | unit | [[ge.layout.c4]] | graphs with labelled, self-loop, and parallel multi-edges | each edge's routed path is orthogonal, connected, passes through its label, and parallel edges never share a path cell |
| p5 | unit | [[ge.layout.c5]] | renderer-style consumers reading the grid output contract | every consumer read matches the layout's own inspection of grid, extents, and paths |
| p6 | unit | [[ge.layout.c6]] | large arbitrary graphs with dense crossing regions | each stage's running time stays within its polynomial bound |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** arbitrary graphs laid out twice under identical options
- **THEN** both runs produce byte-identical grid outputs
- **VERIFIES** [[ge.layout.p1]]

#### Scenario: p2

- **WHEN** arbitrary graphs including dense and wide ones
- **THEN** the produced grid contains no node cell overlap
- **VERIFIES** [[ge.layout.p2]]

#### Scenario: p3

- **WHEN** graphs with all four flow directions and self-loops
- **THEN** every edge's source precedes its target along the flow axis, self-loops excepted
- **VERIFIES** [[ge.layout.p3]]

#### Scenario: p4

- **WHEN** graphs with labelled, self-loop, and parallel multi-edges
- **THEN** each edge's routed path is orthogonal, connected, passes through its label, and parallel edges never share a path cell
- **VERIFIES** [[ge.layout.p4]]

#### Scenario: p5

- **WHEN** renderer-style consumers reading the grid output contract
- **THEN** every consumer read matches the layout's own inspection of grid, extents, and paths
- **VERIFIES** [[ge.layout.p5]]

#### Scenario: p6

- **WHEN** large arbitrary graphs with dense crossing regions
- **THEN** each stage's running time stays within its polynomial bound
- **VERIFIES** [[ge.layout.p6]]

