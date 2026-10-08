---
id: ge.boxart_render
kind: intent
statement: "WHERE the boxart renderer is used THE boxart-render SHALL draw the laid-out graph with Unicode box-drawing glyphs so that borders and edge lines join seamlessly as upstream Graph::Easy's boxart output does."
---

# boxart-render

The Unicode (boxart) renderer — gently's port of Graph::Easy's
`as_boxart`/`As_boxart.pm`. Like [[ge.ascii_render]] it consumes
[[ge.layout]] through the published grid contract, but uses Unicode
box-drawing characters so borders and edge lines join seamlessly.

## Constraints

| id | kind | expr | traces_to | satisfies |
|----|------|------|-----------|-----------|
| c1 | invariant | Each node renders as a box built from Unicode box-drawing glyphs (`─ │ ┌ ┐ └ ┘ ═ ║ ╔ ╗ ╚ ╝` and friends) selected by the node's border style. | [[ge.boxart_render]] | [[ge.layout.c5]] |
| c2 | invariant | Border corners and T-junctions are drawn from the actual neighbouring cells, so a junction always renders as the correct combined glyph. | [[ge.boxart_render]] | |
| c3 | invariant | Edge styles map to the upstream Unicode edge-style table: solid `─│`, double `═║`, dotted `·:`, dashed `╴╵`, dot-dash `·-`/`!`, dot-dot-dash `··-`/`!`, wave `∼≀`, bold `━┃`, double-dash `═ `/`∥`; horizontal repeat units may span several columns per cell width. | [[ge.boxart_render]] | |
| c4 | invariant | A node's `shape` attribute (upstream vocabulary: box, rounded, point, circle, ellipse, diamond, triangle, pentagon, hexagon, octagon, parallelogram, house, invisible, img) changes the box outline per the upstream Unicode shape table. | [[ge.boxart_render]] | |

## Model

### States

- `grid_received`
- `glyphing`
- `boxart_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | grid_received | glyphing | [[ge.boxart_render.c1]] |
| t2 | glyphing | boxart_done | [[ge.boxart_render.c2]] |
| t3 | boxart_done | glyphing | [[ge.boxart_render.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.boxart_render.c1]] | nodes with every border style | each border style yields its Unicode glyph set, one character per cell |
| p2 | unit | [[ge.boxart_render.c2]] | grids with junctions between borders and edges of mixed styles | every junction cell equals the combined glyph for its exact neighbourhood |
| p3 | unit | [[ge.boxart_render.c3]] | graphs with every edge style | each edge renders with the exact upstream Unicode glyphs, repeat units spanning the same column counts as upstream |
| p4 | unit | [[ge.boxart_render.c4]] | nodes with every shape in the upstream vocabulary | each shape's outline matches the upstream Unicode shape table glyph for glyph |
