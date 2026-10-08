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

## Constraints

| id | kind | expr | traces_to |
|----|------|------|-----------|
| c1 | invariant | The output starts with class attribute sections for graph, node, edge, and group classes, emitted in sorted class order with sorted attribute order. | [[ge.txt_render]] |
| c2 | invariant | Every node is emitted with its name and its instance attributes, and every edge as an operator chain whose operator matches the edge style and direction. | [[ge.txt_render]] |
| c3 | invariant | Parsing the emitted text with [[ge.text_parser]] reproduces a model with the same nodes, edges, styles, labels, directions, and attributes as the source model. | [[ge.txt_render]] |
| c4 | invariant | Recorded upstream Graph::Easy fixtures (input text plus expected canonical text) render byte-identically once their layout-annotation attributes are honored. | [[ge.txt_render]] |

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
| p3 | unit | [[ge.txt_render.c3]] | arbitrary graphs serialized then re-parsed | the re-parsed model is equal to the source model under the model-equality predicate |
| p4 | unit | [[ge.txt_render.c4]] | the recorded upstream fixture corpus | every fixture's expected canonical text matches gently's output byte-for-byte |
