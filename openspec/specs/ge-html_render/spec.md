---
id: spec
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
| c1 | invariant | Every grid cell maps to exactly one `td` element, with a CSS class derived from its content kind (node, edge, group, empty). | [[spec]] | ge.layout.c5 (cross-file) |
| c2 | invariant | Node cells carry the node's class name and label, and a node `link` attribute becomes an `a href` wrapping the label; label text and link URLs are HTML-escaped. | [[spec]] |  |
| c3 | invariant | Edge styles map to the documented border-image CSS classes, and edge labels render as text on the edge cells. | [[spec]] |  |
| c4 | invariant | The emitted document embeds the CSS rules for every class it uses, so the output is self-contained. | [[spec]] |  |
| c5 | invariant | Node and edge `fill`, `background`, and `color` attributes map to CSS color declarations on the corresponding elements, using the upstream W3C color-name scheme, and node `shape` attributes map to their documented CSS classes. | [[spec]] |  |

## Model

### States

- `grid_received`
- `emitting`
- `html_done`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | grid_received | emitting | [[spec.c1]] |
| t2 | emitting | html_done | [[spec.c2]] |
| t3 | html_done | emitting | [[spec.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[spec.c1]] | arbitrary laid-out graphs | cell count equals grid size and each td carries the right content class |
| p2 | unit | [[spec.c2]] | nodes with plain labels, link attributes, and labels containing `&`, `<`, `>`, quotes | labels and links render as specified and fully escaped |
| p3 | unit | [[spec.c3]] | graphs with every edge style | edge cells carry the documented CSS classes |
| p4 | unit | [[spec.c4]] | rendered documents of arbitrary graphs | every class referenced in the table has a matching CSS rule in the document |
| p5 | unit | [[spec.c5]] | nodes and edges with named, rgb, and hex colors plus every shape | colors appear as the upstream CSS declarations and shapes carry their documented classes |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** arbitrary laid-out graphs
- **THEN** cell count equals grid size and each td carries the right content class
- **VERIFIES** [[spec.p1]]

#### Scenario: p2

- **WHEN** nodes with plain labels, link attributes, and labels containing `&`, `<`, `>`, quotes
- **THEN** labels and links render as specified and fully escaped
- **VERIFIES** [[spec.p2]]

#### Scenario: p3

- **WHEN** graphs with every edge style
- **THEN** edge cells carry the documented CSS classes
- **VERIFIES** [[spec.p3]]

#### Scenario: p4

- **WHEN** rendered documents of arbitrary graphs
- **THEN** every class referenced in the table has a matching CSS rule in the document
- **VERIFIES** [[spec.p4]]

#### Scenario: p5

- **WHEN** nodes and edges with named, rgb, and hex colors plus every shape
- **THEN** colors appear as the upstream CSS declarations and shapes carry their documented classes
- **VERIFIES** [[spec.p5]]

