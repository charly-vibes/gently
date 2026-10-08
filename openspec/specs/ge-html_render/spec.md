---
id: ge.html_render
kind: intent
statement: "WHERE the html renderer is used THE html-render SHALL emit table-based HTML mirroring the layout grid with CSS classes derived from the graph, node, and edge attributes."
---

# html-render

The HTML renderer — gently's port of Graph::Easy's `as_html`. It consumes
[[ge.layout]] through the published grid contract and emits an HTML
table plus CSS classes, the default output format of the Perl module.

## Purpose

The HTML renderer — gently's port of Graph::Easy's `as_html`. It consumes
[[ge.layout]] through the published grid contract and emits an HTML
table plus CSS classes, the default output format of the Perl module.

## Constraints

| id | kind | expr | traces_to | satisfies |
|----|------|------|-----------|-----------|
| c1 | invariant | Every grid cell maps to exactly one `td` element, with a CSS class derived from its content kind (node, edge, group, empty). | [[ge.html_render]] | [[ge.layout.c5]] |
| c2 | invariant | Node cells carry the node's class name and label, and a node `link` attribute becomes an `a href` wrapping the label; label text and link URLs are HTML-escaped. | [[ge.html_render]] | |
| c3 | invariant | Edge styles map to the documented border-image CSS classes, and edge labels render as text on the edge cells. | [[ge.html_render]] | |
| c4 | invariant | The emitted document embeds the CSS rules for every class it uses, so the output is self-contained. | [[ge.html_render]] | |
| c5 | invariant | Node and edge `fill`, `background`, and `color` attributes map to CSS color declarations on the corresponding elements, using the upstream W3C color-name scheme, and node `shape` attributes map to their documented CSS classes. | [[ge.html_render]] | |

## Model

### States

- `grid_received`
- `emitting`
- `html_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | grid_received | emitting | [[ge.html_render.c1]] |
| t2 | emitting | html_done | [[ge.html_render.c2]] |
| t3 | html_done | emitting | [[ge.html_render.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.html_render.c1]] | arbitrary laid-out graphs | cell count equals grid size and each td carries the right content class |
| p2 | unit | [[ge.html_render.c2]] | nodes with plain labels, link attributes, and labels containing `&`, `<`, `>`, quotes | labels and links render as specified and fully escaped |
| p3 | unit | [[ge.html_render.c3]] | graphs with every edge style | edge cells carry the documented CSS classes |
| p4 | unit | [[ge.html_render.c4]] | rendered documents of arbitrary graphs | every class referenced in the table has a matching CSS rule in the document |
| p5 | unit | [[ge.html_render.c5]] | nodes and edges with named, rgb, and hex colors plus every shape | colors appear as the upstream CSS declarations and shapes carry their documented classes |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** arbitrary laid-out graphs
- **THEN** cell count equals grid size and each td carries the right content class
- **VERIFIES** [[ge.html_render.p1]]

#### Scenario: p2

- **WHEN** nodes with plain labels, link attributes, and labels containing `&`, `<`, `>`, quotes
- **THEN** labels and links render as specified and fully escaped
- **VERIFIES** [[ge.html_render.p2]]

#### Scenario: p3

- **WHEN** graphs with every edge style
- **THEN** edge cells carry the documented CSS classes
- **VERIFIES** [[ge.html_render.p3]]

#### Scenario: p4

- **WHEN** rendered documents of arbitrary graphs
- **THEN** every class referenced in the table has a matching CSS rule in the document
- **VERIFIES** [[ge.html_render.p4]]

#### Scenario: p5

- **WHEN** nodes and edges with named, rgb, and hex colors plus every shape
- **THEN** colors appear as the upstream CSS declarations and shapes carry their documented classes
- **VERIFIES** [[ge.html_render.p5]]

