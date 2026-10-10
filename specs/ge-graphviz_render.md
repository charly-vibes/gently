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
| c2 | invariant | Every edge emits one edge statement whose arrow (`->` or `--`) matches the model edge direction, with edge style attributes mapped to DOT attribute values. Re-derived (gently-0kg): the pinned oracle does NOT do this — its arrow follows the graph type, not the edge direction, so a `digraph` emits `->` for every edge and undirected/bidirectional model edges come back directed (tests/repro/claims/graphviz-round-trip.observed; `--` appears only under an undirected graph type, dot-direction.observed). gently mirrors the model edge direction as a documented divergence, so the emitted DOT preserves the model's directionality; byte-compat with upstream is abandoned here by design. | [[ge.graphviz_render]] |
| c3 | invariant | Every group emits a subgraph cluster containing exactly its member nodes, with the group label as cluster label; anonymous groups emit clusters named `cluster<N>` after their internal id, as upstream does. | [[ge.graphviz_render]] |
| c4 | invariant | Feeding the emitted DOT through [[ge.dot_parser]] yields a model isomorphic to the source model (same nodes, edges, directions, group membership, and full group identity — the [[ge.dot_parser]] cluster convention restores the verbatim group name from `cluster_<name>` and anonymity from `cluster<N>`). Group-endpoint edges are outside this contract: the gently model cannot represent them, and upstream is lossy there anyway — `as_graphviz` emits an ARROWLESS statement for a member-bearing group-endpoint edge and crashes (`_graphviz_point` on undefined, As_graphviz.pm:547) on a bare one (tests/repro/claims/group-edge-graphviz.observed). | [[ge.graphviz_render]] |

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
| p3 | unit | [[ge.graphviz_render.c3]] | graphs with named and anonymous groups | cluster membership equals group membership and anonymous clusters get `cluster<N>` names in internal-id order |
| p4 | unit | [[ge.graphviz_render.c4]] | arbitrary graphs rendered to DOT and re-parsed | the re-parsed model is isomorphic to the source model with full group identity — verbatim named group names and restored anonymity via the [[ge.dot_parser]] cluster convention (`cluster_<name>` re-parses as `<name>`, `cluster<N>` as an anonymous group); group membership itself round-trips; group-endpoint edges excluded |
