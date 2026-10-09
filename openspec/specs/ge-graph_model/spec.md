---
id: ge.graph_model
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
| c1 | invariant | Within one graph, every node — named or anonymous — is identified by a unique name; anonymous nodes receive generated `#N` names from the shared object-id counter (the text parser resets that counter at each parse, making parser-derived names deterministic) and are referenceable through those names exactly like named nodes. | [[ge.graph_model]] |
| c2 | invariant | Every attribute assignment applies to exactly the object or class scope it targets, and its value passes through the store-layer unquote before storage; the `border` attribute is decomposed at assignment time into border-style, border-width, and border-color (never stored verbatim) and recomposed on read, exactly as upstream Graph::Easy does. | [[ge.graph_model]] |
| c3 | invariant | Every edge references exactly two existing endpoints as source and target at all times — usually nodes, but a group object is a legal endpoint (stored as the group object itself, never rewritten to the group's members); self-loops (source == target) are legal; deleting a node removes it and drops all its incident edges, and a later edge construction naming the deleted node succeeds by implicitly re-creating it as a fresh, attribute-less node (the deleted node's stored attributes and old edges are not revived). | [[ge.graph_model]] |
| c4 | invariant | Each edge is either directed or undirected and this direction, together with per-end arrow-head presence, is preserved exactly as constructed. | [[ge.graph_model]] |
| c5 | extension_point | The model publishes its traversal and attribute-access contract for downstream specs: [[ge.layout]], [[ge.ascii_render]], [[ge.html_render]], and [[ge.graphviz_render]] consume it via `satisfies` cells declared on their own constraint rows. | [[ge.graph_model]] |

## Model

### States

- `initial`
- `populated`
- `mutating`
- `ready`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | initial | populated | [[ge.graph_model.c1]] |
| t2 | populated | mutating | [[ge.graph_model.c3]] |
| t3 | mutating | populated | [[ge.graph_model.c2]] |
| t4 | mutating | ready | [[ge.graph_model.c4]] |
| t5 | ready | mutating | [[ge.graph_model.c4]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.graph_model.c1]] | arbitrary node names inserted into one graph, including duplicates, plus parser-created anonymous nodes | the model never exposes two distinct nodes with the same name; a duplicate name insertion merges into the existing node (upstream add_node semantics: the existing node is returned, no second node is created); every anonymous node carries a generated `#N` name and a lookup by that name resolves to the same node object |
| p2 | unit | [[ge.graph_model.c2]] | arbitrary attribute assignments over nodes, edges, groups, and classes | for any assignment, reading the attribute back on the targeted scope yields the stored form — the assigned value after store-layer unquoting, or the recomposed `border` attribute for border assignments — and reads on other scopes are unchanged (class-scope assignments do not leak into instance reads, which override them) |
| p3 | unit | [[ge.graph_model.c3]] | arbitrary edge constructions over existing nodes, groups, and group objects, including self-loops, plus node deletions followed by edge constructions referencing the deleted names | every stored edge resolves both endpoints to live model objects (nodes or group objects); edges constructed with a group as endpoint are stored with the group object itself, never rewritten to the group's members; after a node deletion no incident edge survives, and an edge construction naming the deleted node succeeds, re-creating the node with no stored instance attributes and none of the deleted node's old edges revived |
| p4 | unit | [[ge.graph_model.c4]] | arbitrary directed, undirected, and bidirectional-edge constructions | round-tripping an edge through the model preserves direction and per-end arrow presence bit-exactly |
| p5 | unit | [[ge.graph_model.c5]] | downstream consumers exercising traversal and attribute reads through the published contract | every consumer-visible traversal and attribute read matches direct model inspection |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** arbitrary node names inserted into one graph, including duplicates, plus parser-created anonymous nodes
- **THEN** the model never exposes two distinct nodes with the same name; a duplicate name insertion merges into the existing node (upstream add_node semantics: the existing node is returned, no second node is created); every anonymous node carries a generated `#N` name and a lookup by that name resolves to the same node object
- **VERIFIES** [[ge.graph_model.p1]]

#### Scenario: p2

- **WHEN** arbitrary attribute assignments over nodes, edges, groups, and classes
- **THEN** for any assignment, reading the attribute back on the targeted scope yields the stored form — the assigned value after store-layer unquoting, or the recomposed `border` attribute for border assignments — and reads on other scopes are unchanged (class-scope assignments do not leak into instance reads, which override them)
- **VERIFIES** [[ge.graph_model.p2]]

#### Scenario: p3

- **WHEN** arbitrary edge constructions over existing nodes, groups, and group objects, including self-loops, plus node deletions followed by edge constructions referencing the deleted names
- **THEN** every stored edge resolves both endpoints to live model objects (nodes or group objects); edges constructed with a group as endpoint are stored with the group object itself, never rewritten to the group's members; after a node deletion no incident edge survives, and an edge construction naming the deleted node succeeds, re-creating the node with no stored instance attributes and none of the deleted node's old edges revived
- **VERIFIES** [[ge.graph_model.p3]]

#### Scenario: p4

- **WHEN** arbitrary directed, undirected, and bidirectional-edge constructions
- **THEN** round-tripping an edge through the model preserves direction and per-end arrow presence bit-exactly
- **VERIFIES** [[ge.graph_model.p4]]

#### Scenario: p5

- **WHEN** downstream consumers exercising traversal and attribute reads through the published contract
- **THEN** every consumer-visible traversal and attribute read matches direct model inspection
- **VERIFIES** [[ge.graph_model.p5]]

