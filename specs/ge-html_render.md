---
id: ge.html_render
kind: intent
statement: "WHERE the html renderer is used THE html-render SHALL emit table-based HTML mirroring the layout grid with CSS classes derived from the graph, node, and edge attributes."
---

# html-render

The HTML renderer — gently's port of Graph::Easy's `as_html`. It consumes
[[ge.layout]] through the published grid contract and emits an HTML
table plus CSS classes, the default output format of the Perl module.

## Constraints

| id | kind | expr | traces_to | satisfies |
|----|------|------|-----------|-----------|
| c1 | invariant | Every grid cell maps to exactly one `td` element, with a CSS class derived from its content kind (node, edge, group, empty). Re-derived (gently-dcp): the pinned oracle does NOT do this — its nodes span their block as a single `td colspan=4 rowspan=4` over the oracle's 4×4 subcell grid, with filler rows padding the machinery, so the oracle's td count is below its cell count (tests/repro/claims/td-colspan.observed). gently renders one `td` per grid cell as a documented simplification of that subcell machinery (skeleton, classes, and cell bytes otherwise follow the oracle) — the one-td-per-cell contract holds against the implementation while byte-compat with the oracle at the skeleton level is abandoned here by design. | [[ge.html_render]] | [[ge.layout.c5]] |
| c2 | invariant | Node cells carry the node's class name and label, and a node `link` attribute becomes an `a href` wrapping the label; label text is HTML-escaped. Re-derived (gently-dcp): link URLs are NOT HTML-escaped — the pinned oracle keeps ampersands raw in the href and only encodes space→`+` and `'`→`%27` (tests/repro/claims/href-escaping.observed); gently matches the oracle (`escape_href`). | [[ge.html_render]] | |
| c3 | invariant | Edge styles map to the documented border-image CSS classes, and edge labels render as text on the edge cells. | [[ge.html_render]] | |
| c4 | invariant | The emitted document embeds the CSS rules for every class it uses, so the output is self-contained. Re-derived (gently-dcp): the pinned oracle's `as_html` emits ONLY the table — the CSS rules are exposed separately via `css()` and embedded by the document form `as_html_file` (tests/repro/claims/html-css-rules.observed). gently mirrors this split: `render` (as_html) emits the bare table; `document` (as_html_file) embeds the `css()` rules in a `<style>` block ahead of the table. | [[ge.html_render]] | |
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
| p1 | unit | [[ge.html_render.c1]] | arbitrary laid-out graphs | td count equals grid size (the documented one-td-per-cell simplification, not the oracle's colspan=4 rowspan=4 span) and each td carries the right content class |
| p2 | unit | [[ge.html_render.c2]] | nodes with plain labels, link attributes, and labels containing `&`, `<`, `>`, quotes | labels render HTML-escaped and links follow the observed escaping rule (space→+, '→%27, raw `&`) |
| p3 | unit | [[ge.html_render.c3]] | graphs with every edge style | edge cells carry the documented CSS classes |
| p4 | unit | [[ge.html_render.c4]] | rendered tables and documents of arbitrary graphs | the table renders bare (no CSS, as upstream `as_html`) and every class referenced in the table has a matching CSS rule in the document's `<style>` block |
| p5 | unit | [[ge.html_render.c5]] | nodes and edges with named, rgb, and hex colors plus every shape | colors appear as the upstream CSS declarations and shapes carry their documented classes |
