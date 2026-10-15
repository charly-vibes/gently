---
id: ge.txt_render
kind: intent
statement: "WHEN canonical serialization is requested THE txt-render SHALL emit the Graph::Easy canonical text form so that re-parsing it reproduces a model equal to the source."
---

# txt-render

The canonical text serializer — gently's port of Graph::Easy's
`as_txt` (As_txt.pm). Beyond being one output format, it is the
lingua franca of golden testing: Graph::Easy's own test corpus stores
expected results in this form, so the port's differential tests read
[[ge.text_parser]] fixtures in and compare [[ge.txt_render]] output
against recorded upstream expectations.

The fixture corpus is pinned to one upstream revision; re-recording
expectations is a deliberate, separately reviewed change, never a
side effect of code edits.

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | The output starts with class attribute sections for graph, node, edge, and group classes, emitted in sorted class order with sorted attribute order. | [[ge.txt_render]] |
| c2 | invariant | Every node is emitted with its name and its instance attributes, and every edge as an operator chain whose operator matches the edge style and direction. | [[ge.txt_render]] |
| c3 | invariant | Parsing the emitted text with [[ge.text_parser]] reproduces a model with the same nodes, edges, styles, labels, directions, attributes, and group membership as the source model. Re-derived (gently-0kg, closed by gently-8jf): member node declarations are emitted inside their group's section — the upstream `( name\n  [ member ]\n)` shape (tests/repro/claims/group-edge-graphviz.observed) — so membership round-trips through the parser's group-scope rule; a node belongs to exactly one group, so its declaration lives only in that section. Group-endpoint edges remain outside the contract: upstream serializes them as plain node edges (tests/repro/claims/group-edge-graphviz.observed) and the gently model cannot represent them. | [[ge.txt_render]] |
| c4 | invariant | Recorded upstream Graph::Easy fixtures — stored under `tests/fixtures/graph-easy/` as input-text files with expected canonical-text companions, captured from one pinned upstream revision — render byte-identically. | [[ge.txt_render]] |
| c5 | invariant | Multi-cell size attributes emit in the upstream As_txt canonical form: whenever `columns` is stored, the emitted attribute set merges `columns`/`rows` into the shorter `size: W,H;` form (rows defaulting to 1) and never outputs the default `1,1`; a lone `rows` attribute is already shorter and stays `rows: N;`. A node whose every attribute collapses to nothing (e.g. `size: 1,1`) is not attribute-bearing — it is declared in the isolated-node pass, not the declaration pass. Re-derived (gently-awd): tests/repro/claims/size-attr.observed. | [[ge.txt_render]] |

| c6 | advisory | A node whose only edge-predecessor is a group-endpoint edge (the `( [ Bonn ] ) -> [ Berlin ]` repro, pinned oracle Graph::Easy v0.69 @ ededa3d7) is still listed by gently's as_txt in the isolated-node pass, beside the group block. Upstream's as_txt omits the node entirely — its group-end handler makes the closed group the edge endpoint, so the declaration pass never reaches the node — while upstream's own as_ascii renders BOTH boxes: upstream is self-inconsistent between the two renderers. Adjudicated class (b) — ORACLE defect, gently is right(er); recorded upstream, DECIDED NOT TO FILE per user decision (gently-278 is the durable record, the item-3 evidence of the decision-trace E4 (b)-checklist). Evidence: gently's own bytes are pinned by tests/repro/claims/as-txt-group-endpoint-omission.observed (class-b raw-scaling pin — verified against gently's invariants under [[ge.layout.c1]], NOT the oracle); the oracle's defect bytes are tests/repro/claims/pod-anon-group.observed (gently-58m). | [[ge.txt_render]] |

## Model

### States

- `model_received`
- `serializing`
- `txt_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | model_received | serializing | [[ge.txt_render.c1]] |
| t2 | serializing | txt_done | [[ge.txt_render.c2]] |
| t3 | txt_done | serializing | [[ge.txt_render.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.txt_render.c1]] | graphs with class attributes on every class | class sections appear first, sorted, with sorted attributes |
| p2 | unit | [[ge.txt_render.c2]] | arbitrary graphs of nodes, edges, and styles | every object appears exactly once with a correct operator form |
| p3 | unit | [[ge.txt_render.c3]] | arbitrary graphs serialized then re-parsed | the re-parsed model is equal to the source model under the model-equality predicate for the parser-supported subset — group membership included: members emitted inside their group section re-parse into the same named membership |
| p4 | unit | [[ge.txt_render.c4]] | the recorded upstream fixture corpus | every fixture's expected canonical text matches gently's output byte-for-byte |
| p5 | unit | [[ge.txt_render.c5]] | attribute-bearing nodes with `size`, `columns`, `rows`, and combinations, plus a node whose attributes all collapse to the default | `columns` merges into `size: W,H` (rows default 1), `columns` + `rows` merge together, a lone `rows` stays `rows: N`, the default `1,1` is dropped, and a fully collapsed node renders attribute-less in the isolated-node pass |
| p6 | unit | [[ge.txt_render.c6]] | the group-endpoint repro `( [ Bonn ] ) -> [ Berlin ]` (pinned oracle v0.69 @ ededa3d7; the class-b raw-scaling pin's source input) | gently's as_txt emits the group block AND `[ Berlin ]` (scenario ge_txt_render::p6, bytes = tests/repro/claims/as-txt-group-endpoint-omission.observed), and gently's as_ascii draws the dashed group border around Bonn with Berlin beside it — the oracle renders the former without Berlin and the latter without the group border |
