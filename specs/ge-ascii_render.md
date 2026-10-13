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
| c1 | invariant | Each node renders as a box whose interior is its label and whose border style (solid, dotted, dashed, double, wave, bold, wide, broad, dot-dash, dot-dot-dash, double-dash, none) comes from the node's border attributes. Re-derived (gently-dcp): bold, wide, and broad render the SAME `#####` outline in ASCII — distinct attribute values only in the HTML border widths (tests/repro/claims/shape-outline-collapse.observed) — so the ASCII style table collapses them onto one glyph set (tests/repro/claims/ascii-render-tables.observed). | [[ge.ascii_render]] | [[ge.layout.c5]] |
| c2 | invariant | Edge style maps to glyph runs exactly as Graph::Easy's short-edge `_draw_hor` rule: fill the gap with the style's unit token (`solid` `-`, `dotted` `.`, `dashed` `- `, `double` `==`, `wave` `~`, `bold`/`wide`/`broad` `##`, `dot-dash` `.-`, `dot-dot-dash` `..-`, `double-dash` `= `), undirected cells shift the fill by the gap cell's absolute x (`rx % len` — directed SHORT cells skip the shift), truncate to the gap width, drop one char per START_W and START_E/END_E, and a directed run replaces its last char with `>`. At the minimum gap width (5 columns, widened to 6 for dot-dot-dash per upstream `_correct_size`) the observed runs are solid `-->`/`---`, double `==>`/`===`, dotted `..>`/`...`, wave `~~>`/`~~~`, dashed `- >`/` - `, bold/wide/broad `##>`/`###`, dot-dash `.->`/`-.-`, dot-dot-dash `..->`/`-..-`, and double-dash `= >`/` = ` — the undirected runs therefore shift with the edge's column position (chain gap 2 renders `..-.`, gap 3 `.-..`). Re-derived (gently-dcp): the earlier fixed-width forms `....>` and `- - >` are false at the probed minimum width — the oracle emits the minimum runs above (tests/repro/claims/ascii-render-tables.observed); re-derived (gently-oy2): the repeated POD patterns (`..-..-..->`, `.-.-.->`, `= = >`) keep their styling, previously collapsed to plain `-->` (tests/repro/claims/edge-style-short-runs.observed); re-derived (gently-bpq): a labeled UNDIRECTED run draws NO arrowhead — the fill pattern only (upstream `_draw_hor` heads only on EDGE_END_* flags; `------------` in tests/repro/claims/pod-label-attr.observed); re-derived (gently-7qz): a BIDIRECTIONAL edge (`<-->`, `<-- label -->`) renders its straight run through an EDGE_SHORT_BD_EW cell — BOTH arrowheads point outward (` <` + fill + `>` drawn at cell-x 0, `w-1` chars), the undirected ofs shift still applies, upstream `_correct_size` widens the horizontal BD cell by one column (solid ` <--> ` at gap 6; labeled ` <-------> ` at gap 11 = label + 5 + 1 with the label centered in [2, w-3) of the widened cell — tests/repro/claims/pod-bidir.observed, pod-bidir-label-straight.observed), and a BENT BD edge stamps the START arrowhead at the source's exit attachment pointing INTO the source box (exit North → `v`, mirroring the target-side entry rule — tests/repro/claims/pod-bidir-parallel.observed); re-derived (gently-51d): `wide` and `broad` render the BOLD runs byte-for-byte — upstream As_ascii `$edge_styles` rows wide/broad are identical to bold (`##`) and `_correct_size` does NOT widen their gap (only dot-dot-dash), so the directed runs are `##>` and the arrow-less runs are `###` (tests/repro/claims/pod-style-family.observed, edge-style-wide-undirected.observed); gently implements exactly this rule. | [[ge.ascii_render]] | |
| c3 | invariant | The rendered output is a function of the grid output alone: no hidden state, no dependence on input text or parse order. | [[ge.ascii_render]] | |
| c4 | invariant | Edge labels occupy cells on their edge's routed path and never overlap node cells or other labels. | [[ge.ascii_render]] | |
| c5 | invariant | A node label wraps into multiple interior rows exactly as upstream does, and each glyph occupies its display width (double-width glyphs take two columns); alignment never depends on byte length. | [[ge.ascii_render]] | |
| c6 | invariant | A node's `shape` attribute (upstream vocabulary: rounded, point, circle, ellipse, diamond, triangle, pentagon, hexagon, octagon, parallelogram, house, invisible, img — `box` is NOT valid; the default shape is `rect`) changes the rendered outline per the upstream ASCII shape table; color attributes do not affect ASCII output. Re-derived (gently-dcp): the probed table collapses to four distinct outlines — every non-special shape renders the plain box for small nodes, `rounded` blanks the corners, `point` swaps the label for a centered `*`, `invisible` draws nothing, and `shape: box` is upstream-rejected (tests/repro/claims/ascii-render-tables.observed, shape-outline-collapse.observed); gently implements exactly this collapse. | [[ge.ascii_render]] | |
| c7 | invariant | An anonymous node renders BOXLESS: the upstream `Node::Anon` class carries borderstyle `none` and the label `' '` (its `_correct_size` fixes the cell at 3×3), so the node's own cell renders blank and the edges terminate at the empty space around it (`[ ] --> [ Bonn ]` leaves the anon cell as 3 blank columns with the arrow centered in the gap); an edgeless anon node contributes no visible bytes. Anonymous nodes are the empty-name nodes of the model API plus the parser-generated `#N` names (gently-6j0's odd counter; upstream's parser rejects a literal `#` in node names, so a `#\d+` name can only be parser-generated). Re-derived (gently-b0z): gently previously rendered boxed `#1`/`#3` — the oracle bytes are tests/repro/claims/pod-anon-nodes.observed (also bg-anon-chain.observed); gently implements exactly this rule. | [[ge.ascii_render]] | |
| c8 | invariant | A named group with members renders a dashed border FRAME around its members' bounding box: the `dashed` style's border glyphs (`+` corners, `- ` run shifted one, `'` sides — one char/row of padding between frame and members), the 3-row title band above (border row, title row, blank row) and the blank row + border row below. The title row renders the group's stored name verbatim at the leftmost member's box x — upstream's parser keeps the `Name:` colon in the group name (probed: `$grp->{name}` is `Cities:`) — and the title cell sits in the leftmost member's grid column, whose `label+2` minimum widens that column so the member box fills it with its label re-centered (`Cities:` widens the westmost member's box from 8 to 9 columns, `Verylonggroupname:` to 20; `G:` stays 8). Re-derived (gently-b0z, gently-7rx): gently previously drew no border — the oracle bytes are tests/repro/claims/pod-group.observed plus the probed relatives (single member, stacked members, long titles); the frame is drawn by the renderer as an overlay around the member box extents (gently's layout has no group cells), so groups whose members are disconnected (the oracle frames each separately) or whose frame a foreign edge crosses (the oracle shifts the edge through the frame) remain unpinned divergences; gently implements exactly the pinned rule. | [[ge.ascii_render]] | |

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
| p2 | unit | [[ge.ascii_render.c2]] | graphs with every edge style (bold/wide/broad as one aliased family), in straight gaps and chains, plus bidirectional (`<-->`, `<-- label -->`) and labeled-undirected edges straight and bent | each edge's glyph run matches the observed minimum-run style table byte-for-byte (arrowed and arrow-less), wide and broad alias the bold runs (`##>`/`###`), undirected runs shift with the gap's absolute column, dot-dot-dash widens its gap to 6, labeled undirected runs carry no head, bidirectional straight runs draw both outward heads at the widened gap with the label centered in [2, w-3), and a bent bidirectional edge stamps the start arrowhead into its source box |
| p3 | unit | [[ge.ascii_render.c3]] | one graph rendered via two different construction orders | outputs are byte-identical |
| p4 | unit | [[ge.ascii_render.c4]] | graphs with labelled crossing edges | no label cell coincides with a node cell or another label cell |
| p5 | unit | [[ge.ascii_render.c5]] | node labels with long text, explicit line breaks, and double-width glyphs | wrapped lines each occupy one interior row and column counts follow display width, not byte count |
| p6 | unit | [[ge.ascii_render.c6]] | nodes with every shape in the upstream vocabulary, with and without color attributes | each shape's outline matches the probed collapse (rounded/point/invisible diverge; every other valid shape renders the plain box; `box` is upstream-rejected) and color attributes leave the output byte-identical |
| p7 | unit | [[ge.ascii_render.c7]] | the POD anon chain (`[ ] --> [ Bonn ]`, `[ Bonn ] --> [ ]`) | the anon cells render blank (3 columns wide, no box, no `#N` label) and the arrows attach to the empty space, byte-identical to the pinned POD bytes (tests/repro/claims/pod-anon-nodes.observed) |
| p8 | unit | [[ge.ascii_render.c8]] | named groups over connected members — one row, stacked rows, three-member chains, long titles | each group's dashed frame (`+ - - +` corners, `'` sides) encloses its members with the 3-row title band above and the blank row + border below, the title renders the stored group name verbatim at the leftmost member's box x, and the title widens the leftmost member's column (`title+2`) with the label re-centered — byte-identical to the pinned POD bytes |
