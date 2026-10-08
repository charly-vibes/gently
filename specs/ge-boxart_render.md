---
id: ge.boxart_render
kind: intent
statement: "WHERE the boxart renderer is used THE boxart-render SHALL draw the laid-out graph with Unicode box-drawing and block glyphs so that every cell of the grid output renders in one character."
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
| c3 | invariant | Edge styles map to their Unicode line glyphs (solid `──`, double `══`, dotted `┄┄`, dashed `╌╌`, wave `≈≈`) with arrow heads from the edge direction. | [[ge.boxart_render]] | |

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
| p3 | unit | [[ge.boxart_render.c3]] | graphs with every edge style | each edge renders with its documented Unicode line glyphs and arrow heads |
