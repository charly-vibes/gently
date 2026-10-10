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

## Purpose

The ASCII-art renderer — gently's port of Graph::Easy's
`as_ascii`/`As_ascii.pm`. It consumes [[ge.layout]] through the published
grid contract and turns cells into a character grid. This is the format
most Graph::Easy users associate with the tool, so its fixtures come
straight from the upstream test corpus.

## Constraints

| id | kind | expr | traces_to | satisfies |
|----|------|------|-----------|-----------|
| c1 | invariant | Each node renders as a box whose interior is its label and whose border style (solid, dotted, dashed, double, wave, bold, wide, broad, dot-dash, dot-dot-dash, double-dash, none) comes from the node's border attributes. Re-derived (gently-dcp): bold, wide, and broad render the SAME `#####` outline in ASCII — distinct attribute values only in the HTML border widths (tests/repro/claims/shape-outline-collapse.observed) — so the ASCII style table collapses them onto one glyph set (tests/repro/claims/ascii-render-tables.observed). | [[ge.ascii_render]] | [[ge.layout.c5]] |
| c2 | invariant | Edge style maps to glyph runs exactly as Graph::Easy does: at the minimum gap width the observed runs are solid `-->`, double `==>`, dotted `..>`, wave `~~>`, dashed `- >`, and bold `##>`, and arrow-less edges use the pattern without the arrow head; labelled or wider runs repeat the style's fill pattern up to the run length. Re-derived (gently-dcp): the earlier fixed-width forms `....>` and `- - >` are false at the probed minimum width — the oracle emits the minimum runs above (tests/repro/claims/ascii-render-tables.observed); gently implements exactly this table. | [[ge.ascii_render]] | |
| c3 | invariant | The rendered output is a function of the grid output alone: no hidden state, no dependence on input text or parse order. | [[ge.ascii_render]] | |
| c4 | invariant | Edge labels occupy cells on their edge's routed path and never overlap node cells or other labels. | [[ge.ascii_render]] | |
| c5 | invariant | A node label wraps into multiple interior rows exactly as upstream does, and each glyph occupies its display width (double-width glyphs take two columns); alignment never depends on byte length. | [[ge.ascii_render]] | |
| c6 | invariant | A node's `shape` attribute (upstream vocabulary: rounded, point, circle, ellipse, diamond, triangle, pentagon, hexagon, octagon, parallelogram, house, invisible, img — `box` is NOT valid; the default shape is `rect`) changes the rendered outline per the upstream ASCII shape table; color attributes do not affect ASCII output. Re-derived (gently-dcp): the probed table collapses to four distinct outlines — every non-special shape renders the plain box for small nodes, `rounded` blanks the corners, `point` swaps the label for a centered `*`, `invisible` draws nothing, and `shape: box` is upstream-rejected (tests/repro/claims/ascii-render-tables.observed, shape-outline-collapse.observed); gently implements exactly this collapse. | [[ge.ascii_render]] | |

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
| p1 | unit | [[ge.ascii_render.c1]] | nodes with every border style | each rendered box border matches its declared style, with bold/wide/broad sharing the same `#####` outline as observed |
| p2 | unit | [[ge.ascii_render.c2]] | graphs with every edge style | each edge's glyph run matches the observed minimum-run style table byte-for-byte (arrowed and arrow-less) |
| p3 | unit | [[ge.ascii_render.c3]] | one graph rendered via two different construction orders | outputs are byte-identical |
| p4 | unit | [[ge.ascii_render.c4]] | graphs with labelled crossing edges | no label cell coincides with a node cell or another label cell |
| p5 | unit | [[ge.ascii_render.c5]] | node labels with long text, explicit line breaks, and double-width glyphs | wrapped lines each occupy one interior row and column counts follow display width, not byte count |
| p6 | unit | [[ge.ascii_render.c6]] | nodes with every shape in the upstream vocabulary, with and without color attributes | each shape's outline matches the probed collapse (rounded/point/invisible diverge; every other valid shape renders the plain box; `box` is upstream-rejected) and color attributes leave the output byte-identical |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** nodes with every border style
- **THEN** each rendered box border matches its declared style, with bold/wide/broad sharing the same `#####` outline as observed
- **VERIFIES** [[ge.ascii_render.p1]]

#### Scenario: p2

- **WHEN** graphs with every edge style
- **THEN** each edge's glyph run matches the observed minimum-run style table byte-for-byte (arrowed and arrow-less)
- **VERIFIES** [[ge.ascii_render.p2]]

#### Scenario: p3

- **WHEN** one graph rendered via two different construction orders
- **THEN** outputs are byte-identical
- **VERIFIES** [[ge.ascii_render.p3]]

#### Scenario: p4

- **WHEN** graphs with labelled crossing edges
- **THEN** no label cell coincides with a node cell or another label cell
- **VERIFIES** [[ge.ascii_render.p4]]

#### Scenario: p5

- **WHEN** node labels with long text, explicit line breaks, and double-width glyphs
- **THEN** wrapped lines each occupy one interior row and column counts follow display width, not byte count
- **VERIFIES** [[ge.ascii_render.p5]]

#### Scenario: p6

- **WHEN** nodes with every shape in the upstream vocabulary, with and without color attributes
- **THEN** each shape's outline matches the probed collapse (rounded/point/invisible diverge; every other valid shape renders the plain box; `box` is upstream-rejected) and color attributes leave the output byte-identical
- **VERIFIES** [[ge.ascii_render.p6]]

