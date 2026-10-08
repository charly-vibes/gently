---
id: ge.graphviz_render
kind: intent
statement: "WHEN graphviz output is requested THE graphviz-render SHALL emit DOT source that dot lays out equivalently to the gently layout of the same graph model."
---

# graphviz-render

The Graphviz DOT output renderer — gently's port of Graph::Easy's
`as_graphviz`. It consumes [[ge.graph_model]] (not the layout grid —
dot does its own layout) and emits DOT source whose elements correspond
one-to-one with the model.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | Every node emits exactly one DOT node statement with a safely quoted name, and node attributes map onto their DOT counterparts. | [[ge.graphviz_render]] |
| c2 | invariant | Every edge emits one edge statement whose arrow (`->` or `--`) matches the model edge direction, with edge style attributes mapped to DOT attribute values. | [[ge.graphviz_render]] |
| c3 | invariant | Every group emits a subgraph cluster containing exactly its member nodes, with the group label as cluster label. | [[ge.graphviz_render]] |
| c4 | invariant | Feeding the emitted DOT through [[ge.dot_parser]] yields a model isomorphic to the source model (same nodes, edges, directions, and group membership). | [[ge.graphviz_render]] |

## Model

### States

- `model_received`
- `emitting`
- `dot_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | model_received | emitting | [[ge.graphviz_render.c1]] |
| t2 | emitting | dot_done | [[ge.graphviz_render.c2]] |
| t3 | dot_done | emitting | [[ge.graphviz_render.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.graphviz_render.c1]] | arbitrary graphs with quoted, spaced, and unicode node names | each node appears exactly once with its quoted name and mapped attributes |
| p2 | unit | [[ge.graphviz_render.c2]] | graphs with directed, undirected, and styled edges | arrow type and attribute mapping match the model exactly |
| p3 | unit | [[ge.graphviz_render.c3]] | graphs with named and anonymous groups | cluster membership equals group membership |
| p4 | unit | [[ge.graphviz_render.c4]] | arbitrary graphs rendered to DOT and re-parsed | the re-parsed model is isomorphic to the source model |
