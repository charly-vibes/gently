---
id: spec
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
| c1 | invariant | Each node renders as a box whose interior is its label and whose border style (solid, dotted, dashed, double, wave, bold, wide) comes from the node's border attributes. | [[spec]] | ge.layout.c5 (cross-file) |
| c2 | invariant | Edge style maps to glyph runs exactly as Graph::Easy does: solid `-->`, double `==>`, dotted `....>`, wave `~~~>`, dashed `- - >`, and arrow-less edges use the pattern without the arrow head. | [[spec]] |  |
| c3 | invariant | The rendered output is a function of the grid output alone: no hidden state, no dependence on input text or parse order. | [[spec]] |  |
| c4 | invariant | Edge labels occupy cells on their edge's routed path and never overlap node cells or other labels. | [[spec]] |  |
| c5 | invariant | A node label wraps into multiple interior rows exactly as upstream does, and each glyph occupies its display width (double-width glyphs take two columns); alignment never depends on byte length. | [[spec]] |  |
| c6 | invariant | A node's `shape` attribute (upstream vocabulary: box, rounded, point, circle, ellipse, diamond, triangle, pentagon, hexagon, octagon, parallelogram, house, invisible, img) changes the rendered outline per the upstream ASCII shape table; color attributes do not affect ASCII output. | [[spec]] |  |

## Model

### States

- `grid_received`
- `drawing`
- `ascii_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | grid_received | drawing | [[spec.c1]] |
| t2 | drawing | ascii_done | [[spec.c2]] |
| t3 | ascii_done | drawing | [[spec.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[spec.c1]] | nodes with every border style | each rendered box border matches its declared style |
| p2 | unit | [[spec.c2]] | graphs with every edge style | each edge's glyph run matches the style table byte-for-byte |
| p3 | unit | [[spec.c3]] | one graph rendered via two different construction orders | outputs are byte-identical |
| p4 | unit | [[spec.c4]] | graphs with labelled crossing edges | no label cell coincides with a node cell or another label cell |
| p5 | unit | [[spec.c5]] | node labels with long text, explicit line breaks, and double-width glyphs | wrapped lines each occupy one interior row and column counts follow display width, not byte count |
| p6 | unit | [[spec.c6]] | nodes with every shape in the upstream vocabulary, with and without color attributes | each shape's outline matches the upstream ASCII shape table and color attributes leave the output byte-identical |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** nodes with every border style
- **THEN** each rendered box border matches its declared style
- **VERIFIES** [[spec.p1]]

#### Scenario: p2

- **WHEN** graphs with every edge style
- **THEN** each edge's glyph run matches the style table byte-for-byte
- **VERIFIES** [[spec.p2]]

#### Scenario: p3

- **WHEN** one graph rendered via two different construction orders
- **THEN** outputs are byte-identical
- **VERIFIES** [[spec.p3]]

#### Scenario: p4

- **WHEN** graphs with labelled crossing edges
- **THEN** no label cell coincides with a node cell or another label cell
- **VERIFIES** [[spec.p4]]

#### Scenario: p5

- **WHEN** node labels with long text, explicit line breaks, and double-width glyphs
- **THEN** wrapped lines each occupy one interior row and column counts follow display width, not byte count
- **VERIFIES** [[spec.p5]]

#### Scenario: p6

- **WHEN** nodes with every shape in the upstream vocabulary, with and without color attributes
- **THEN** each shape's outline matches the upstream ASCII shape table and color attributes leave the output byte-identical
- **VERIFIES** [[spec.p6]]

