---
id: spec
kind: intent
statement: "WHEN canonical serialization is requested THE txt-render SHALL emit the Graph::Easy canonical text form so that re-parsing it reproduces a model equal to the source."
---

# txt-render

The canonical text serializer — gently's port of Graph::Easy's
`as_txt` (As_txt.pm). Beyond being one output format, it is the
lingua franca of golden testing: Graph::Easy's own test corpus stores
expected results in this form, so the port's differential tests read
[[ge.text_parser]] fixtures in and compare [[spec]] output
against recorded upstream expectations.

The fixture corpus is pinned to one upstream revision; re-recording
expectations is a deliberate, separately reviewed change, never a
side effect of code edits.

## Purpose

The canonical text serializer — gently's port of Graph::Easy's
`as_txt` (As_txt.pm). Beyond being one output format, it is the
lingua franca of golden testing: Graph::Easy's own test corpus stores
expected results in this form, so the port's differential tests read
[[ge.text_parser]] fixtures in and compare [[spec]] output
against recorded upstream expectations.
The fixture corpus is pinned to one upstream revision; re-recording
expectations is a deliberate, separately reviewed change, never a
side effect of code edits.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | The output starts with class attribute sections for graph, node, edge, and group classes, emitted in sorted class order with sorted attribute order. | [[spec]] |
| c2 | invariant | Every node is emitted with its name and its instance attributes, and every edge as an operator chain whose operator matches the edge style and direction. | [[spec]] |
| c3 | invariant | Parsing the emitted text with ge.text_parser (cross-file) reproduces a model with the same nodes, edges, styles, labels, directions, and attributes as the source model. | [[spec]] |
| c4 | invariant | Recorded upstream Graph::Easy fixtures — stored under `tests/fixtures/graph-easy/` as input-text files with expected canonical-text companions, captured from one pinned upstream revision — render byte-identically. | [[spec]] |

## Model

### States

- `model_received`
- `serializing`
- `txt_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | model_received | serializing | [[spec.c1]] |
| t2 | serializing | txt_done | [[spec.c2]] |
| t3 | txt_done | serializing | [[spec.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[spec.c1]] | graphs with class attributes on every class | class sections appear first, sorted, with sorted attributes |
| p2 | unit | [[spec.c2]] | arbitrary graphs of nodes, edges, and styles | every object appears exactly once with a correct operator form |
| p3 | unit | [[spec.c3]] | arbitrary graphs serialized then re-parsed | the re-parsed model is equal to the source model under the model-equality predicate |
| p4 | unit | [[spec.c4]] | the recorded upstream fixture corpus | every fixture's expected canonical text matches gently's output byte-for-byte |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** graphs with class attributes on every class
- **THEN** class sections appear first, sorted, with sorted attributes
- **VERIFIES** [[spec.p1]]

#### Scenario: p2

- **WHEN** arbitrary graphs of nodes, edges, and styles
- **THEN** every object appears exactly once with a correct operator form
- **VERIFIES** [[spec.p2]]

#### Scenario: p3

- **WHEN** arbitrary graphs serialized then re-parsed
- **THEN** the re-parsed model is equal to the source model under the model-equality predicate
- **VERIFIES** [[spec.p3]]

#### Scenario: p4

- **WHEN** the recorded upstream fixture corpus
- **THEN** every fixture's expected canonical text matches gently's output byte-for-byte
- **VERIFIES** [[spec.p4]]

