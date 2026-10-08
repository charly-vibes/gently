---
id: spec
kind: intent
statement: "THE graph-model SHALL represent nodes, edges, groups, and per-object attributes losslessly so that every parser, layout, and renderer capability consumes exactly one shared model."
---

# graph-model

The core in-memory data model of `gently` — the Rust port of Perl's
Graph::Easy. One `Graph` owns nodes, edges, and groups; every object
carries a class-scoped attribute table. This capability renders and lays
out nothing itself; the parsers ([[ge.text_parser]], [[ge.dot_parser]]),
[[ge.layout]], and the render specs are its consumers.

Out of scope for v1, matching upstream's own packaging: SVG output
(lives in the separate Graph::Easy::As_svg dist upstream), GraphML and
VCG output (As_graphml.pm / As_vcg.pm), and animations. These are
deliberately unmodeled; introducing any of them is a new capability
spec, not a silent extension of an existing one.

## Purpose

The core in-memory data model of `gently` — the Rust port of Perl's
Graph::Easy. One `Graph` owns nodes, edges, and groups; every object
carries a class-scoped attribute table. This capability renders and lays
out nothing itself; the parsers ([[ge.text_parser]], [[ge.dot_parser]]),
[[ge.layout]], and the render specs are its consumers.
Out of scope for v1, matching upstream's own packaging: SVG output
(lives in the separate Graph::Easy::As_svg dist upstream), GraphML and
VCG output (As_graphml.pm / As_vcg.pm), and animations. These are
deliberately unmodeled; introducing any of them is a new capability
spec, not a silent extension of an existing one.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | Within one graph, every named node is identified by a unique name; anonymous nodes are unnamed and cannot be referenced again after creation. | [[spec]] |
| c2 | invariant | Every attribute assignment applies to exactly the object or class scope it targets, and its value is stored verbatim (no loss); derived border components (style, width, color) are computed at assignment time exactly as upstream Graph::Easy does. | [[spec]] |
| c3 | invariant | Every edge references exactly two existing nodes as source and target at all times; self-loops (source == target) are legal. | [[spec]] |
| c4 | invariant | Each edge is either directed or undirected and this direction, together with per-end arrow-head presence, is preserved exactly as constructed. | [[spec]] |
| c5 | extension_point | The model publishes its traversal and attribute-access contract for downstream specs: ge.layout (cross-file), ge.ascii_render (cross-file), ge.html_render (cross-file), and ge.graphviz_render (cross-file) consume it via `satisfies` cells declared on their own constraint rows. | [[spec]] |

## Model

### States

- `initial`
- `populated`
- `mutating`
- `ready`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | initial | populated | [[spec.c1]] |
| t2 | populated | mutating | [[spec.c3]] |
| t3 | mutating | populated | [[spec.c2]] |
| t4 | mutating | ready | [[spec.c4]] |
| t5 | ready | mutating | [[spec.c4]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[spec.c1]] | arbitrary node names inserted into one graph, including duplicates | the model never exposes two distinct nodes with the same name; a duplicate name insertion merges into the existing node (upstream add_node semantics: the existing node is returned, no second node is created) |
| p2 | unit | [[spec.c2]] | arbitrary attribute assignments over nodes, edges, groups, and classes | for any assignment, reading the attribute back on the targeted scope yields exactly the assigned value and reads on other scopes are unchanged |
| p3 | unit | [[spec.c3]] | arbitrary edge constructions over existing nodes, including self-loops and attempts on dropped nodes | every stored edge resolves both endpoints to live nodes; no dangling edge ever escapes a mutation |
| p4 | unit | [[spec.c4]] | arbitrary directed, undirected, and bidirectional-edge constructions | round-tripping an edge through the model preserves direction and per-end arrow presence bit-exactly |
| p5 | unit | [[spec.c5]] | downstream consumers exercising traversal and attribute reads through the published contract | every consumer-visible traversal and attribute read matches direct model inspection |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** arbitrary node names inserted into one graph, including duplicates
- **THEN** the model never exposes two distinct nodes with the same name; a duplicate name insertion merges into the existing node (upstream add_node semantics: the existing node is returned, no second node is created)
- **VERIFIES** [[spec.p1]]

#### Scenario: p2

- **WHEN** arbitrary attribute assignments over nodes, edges, groups, and classes
- **THEN** for any assignment, reading the attribute back on the targeted scope yields exactly the assigned value and reads on other scopes are unchanged
- **VERIFIES** [[spec.p2]]

#### Scenario: p3

- **WHEN** arbitrary edge constructions over existing nodes, including self-loops and attempts on dropped nodes
- **THEN** every stored edge resolves both endpoints to live nodes; no dangling edge ever escapes a mutation
- **VERIFIES** [[spec.p3]]

#### Scenario: p4

- **WHEN** arbitrary directed, undirected, and bidirectional-edge constructions
- **THEN** round-tripping an edge through the model preserves direction and per-end arrow presence bit-exactly
- **VERIFIES** [[spec.p4]]

#### Scenario: p5

- **WHEN** downstream consumers exercising traversal and attribute reads through the published contract
- **THEN** every consumer-visible traversal and attribute read matches direct model inspection
- **VERIFIES** [[spec.p5]]

