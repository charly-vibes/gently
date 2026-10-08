---
id: ge.ascii_render
kind: intent
statement: "WHEN a laid-out graph is rendered as ASCII THE ascii-render SHALL draw the grid with box-drawing and arrow glyphs so that the output is byte-compatible with Graph::Easy's classic ascii output for the same input."
---

# ascii-render

The ASCII-art renderer — gently's port of Graph::Easy's
`as_ascii`/`As_ascii.pm`. It consumes [[ge.layout]] through the published
grid contract and turns cells into a character grid. This is the format
most Graph::Easy users associate with the tool, so its fixtures come
straight from the upstream test corpus.

## Constraints

| id | kind | expr | traces_to | satisfies |
|----|------|------|-----------|-----------|
| c1 | invariant | Each node renders as a box whose interior is its label and whose border style (solid, dotted, dashed, double, wave, bold, wide) comes from the node's border attributes. | [[ge.ascii_render]] | [[ge.layout.c5]] |
| c2 | invariant | Edge style maps to glyph runs exactly as Graph::Easy does: solid `-->`, double `==>`, dotted `....>`, wave `~~~>`, dashed `- - >`, and arrow-less edges use the pattern without the arrow head. | [[ge.ascii_render]] | |
| c3 | invariant | The rendered output is a function of the grid output alone: no hidden state, no dependence on input text or parse order. | [[ge.ascii_render]] | |
| c4 | invariant | Edge labels occupy cells on their edge's routed path and never overlap node cells or other labels. | [[ge.ascii_render]] | |
| c5 | invariant | A node label wraps into multiple interior rows exactly as upstream does, and each glyph occupies its display width (double-width glyphs take two columns); alignment never depends on byte length. | [[ge.ascii_render]] | |
| c6 | invariant | A node's `shape` attribute (upstream vocabulary: box, rounded, point, circle, ellipse, diamond, triangle, pentagon, hexagon, octagon, parallelogram, house, invisible, img) changes the rendered outline per the upstream ASCII shape table; color attributes do not affect ASCII output. | [[ge.ascii_render]] | |

## Model

### States

- `grid_received`
- `drawing`
- `ascii_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | grid_received | drawing | [[ge.ascii_render.c1]] |
| t2 | drawing | ascii_done | [[ge.ascii_render.c2]] |
| t3 | ascii_done | drawing | [[ge.ascii_render.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.ascii_render.c1]] | nodes with every border style | each rendered box border matches its declared style |
| p2 | unit | [[ge.ascii_render.c2]] | graphs with every edge style | each edge's glyph run matches the style table byte-for-byte |
| p3 | unit | [[ge.ascii_render.c3]] | one graph rendered via two different construction orders | outputs are byte-identical |
| p4 | unit | [[ge.ascii_render.c4]] | graphs with labelled crossing edges | no label cell coincides with a node cell or another label cell |
| p5 | unit | [[ge.ascii_render.c5]] | node labels with long text, explicit line breaks, and double-width glyphs | wrapped lines each occupy one interior row and column counts follow display width, not byte count |
| p6 | unit | [[ge.ascii_render.c6]] | nodes with every shape in the upstream vocabulary, with and without color attributes | each shape's outline matches the upstream ASCII shape table and color attributes leave the output byte-identical |
