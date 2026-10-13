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
| c2 | invariant | Edge style maps to glyph runs exactly as Graph::Easy's short-edge `_draw_hor` rule: fill the gap with the style's unit token (`solid` `-`, `dotted` `.`, `dashed` `- `, `double` `==`, `wave` `~`, `bold`/`wide`/`broad` `##`, `dot-dash` `.-`, `dot-dot-dash` `..-`, `double-dash` `= `), undirected cells shift the fill by the gap cell's absolute x (`rx % len` — directed SHORT cells skip the shift), truncate to the gap width, drop one char per START_W and START_E/END_E, and a directed run replaces its last char with `>`. At the minimum gap width (5 columns, widened to 6 for dot-dot-dash per upstream `_correct_size`) the observed runs are solid `-->`/`---`, double `==>`/`===`, dotted `..>`/`...`, wave `~~>`/`~~~`, dashed `- >`/` - `, bold/wide/broad `##>`/`###`, dot-dash `.->`/`-.-`, dot-dot-dash `..->`/`-..-`, and double-dash `= >`/` = ` — the undirected runs therefore shift with the edge's column position (chain gap 2 renders `..-.`, gap 3 `.-..`). Re-derived (gently-dcp): the earlier fixed-width forms `....>` and `- - >` are false at the probed minimum width — the oracle emits the minimum runs above (tests/repro/claims/ascii-render-tables.observed); re-derived (gently-oy2): the repeated POD patterns (`..-..-..->`, `.-.-.->`, `= = >`) keep their styling, previously collapsed to plain `-->` (tests/repro/claims/edge-style-short-runs.observed); re-derived (gently-bpq): a labeled UNDIRECTED run draws NO arrowhead — the fill pattern only (upstream `_draw_hor` heads only on EDGE_END_* flags; `------------` in tests/repro/claims/pod-label-attr.observed); re-derived (gently-7qz): a BIDIRECTIONAL edge (`<-->`, `<-- label -->`) renders its straight run through an EDGE_SHORT_BD_EW cell — BOTH arrowheads point outward (` <` + fill + `>` drawn at cell-x 0, `w-1` chars), the undirected ofs shift still applies, upstream `_correct_size` widens the horizontal BD cell by one column (solid ` <--> ` at gap 6; labeled ` <-------> ` at gap 11 = label + 5 + 1 with the label centered in [2, w-3) of the widened cell — tests/repro/claims/pod-bidir.observed, pod-bidir-label-straight.observed), and a BENT BD edge stamps the START arrowhead at the source's exit attachment pointing INTO the source box (exit North → `v`, mirroring the target-side entry rule — tests/repro/claims/pod-bidir-parallel.observed); re-derived (gently-51d): `wide` and `broad` render the BOLD runs byte-for-byte — upstream As_ascii `$edge_styles` rows wide/broad are identical to bold (`##`) and `_correct_size` does NOT widen their gap (only dot-dot-dash), so the directed runs are `##>` and the arrow-less runs are `###` (tests/repro/claims/pod-style-family.observed, edge-style-wide-undirected.observed); re-derived (gently-klf): a vertical edge run attaches at its edge cell's LEFT+2 — upstream `_draw_cross` draws the vertical line at fixed offset 2 within the cell (`_printfb_ver($fb, 2, 0, $line)`), not at the node's center, so a wide box's north/south edge attachments sit 2 chars in from its left border (tests/repro/claims/pod-triangle.observed: the triangle's back-edge rides the band with `|` at Bonn's left+2 and the `v` at Hamburg's left+2; every pinned vertical attachment in the corpus sits at left+2); re-derived (gently-yhg): a straight edge with `minlen >= 2` widens its gap by exactly ONE column — the probed oracle renders minlen 2, 3, 4 and 5 identically (one extra GAP_WIDTH, the run filling the doubled gap; tests/repro/claims/bg-minlen-stretch.observed, 4-seed stable), and the stretch is a gap-column WIDTH, not a placement change — the bg-minlen-bypass shape renders byte-identically WITHOUT minlen (probed), pinning a shared-middle bypass ranking divergence instead (follow-up bead); re-derived (gently-cbk): a straight edge whose endpoint is a `shape: edge` node renders through the stripped endpoint flags (upstream Scout.pm) — a shape:edge SOURCE loses its START flags so the directed run hangs flush off its border (`--->` at cell-x 0 into Ulm), a shape:edge TARGET on an undirected run keeps only `START_W` (the `--------` chain through Bonn–train–Ulm, no arrowhead), while a DIRECTED run's arrowhead always survives the short-cell strip (rent's `>` rides INTO the car edge cell); a straight VERTICAL edge renders its `EDGE_VER` short cell — pipes at the cell's left+2 down the gap band's full height with the `^`/`v` head at the target-adjacent end (`^` entering train's south) and START flags never rendering (upstream: vertical starting points are invisible) (tests/repro/claims/bg-edge-node.observed); gently implements exactly this rule. | [[ge.ascii_render]] | |
| c3 | invariant | The rendered output is a function of the grid output alone: no hidden state, no dependence on input text or parse order. | [[ge.ascii_render]] | |
| c4 | invariant | Edge labels occupy cells on their edge's routed path and never overlap node cells or other labels. Re-derived (gently-527): a straight edge's label CELL is sized by its widest line — width = longest-line byte length + 5 (+1 for a bidirectional arrowhead column), height = lines + 2 — and every cell in that row is grown to the row height (upstream `_prepare_layout` rewrites each cell's h to its row's height), so a multiline label grows its band and both endpoint boxes grow one blank interior row with vertically centered labels; the label rows render one per line, bottom-aligned in the cell's `h-2` field starting at byte offset 2 (`w-5` bytes wide eastward — `ws = xs + xr = 3` with the east head reserving the last byte — `w-4` westward), left-aligned by default (upstream's edge `align` attribute defaults `left`, node `center`) (tests/repro/claims/edge-label-split-minimal.observed, bg-multiline-align.observed, so-flowchart-size.observed); re-derived (gently-cbk): a straight VERTICAL single-gap edge renders its label cell too — the label lines ride the gap band's middle line, left-aligned at byte offset 4 (upstream `_draw_ver`'s `_insert_label(xs=4, ys=1, ..., 'middle')`), growing no band (tests/repro/claims/bg-edge-node.observed: `buy`); gently implements exactly this rule. | [[ge.ascii_render]] | |
| c5 | invariant | A label splits into one interior row per line — a literal `\n` splits (gently-527) and the `\l`/`\r`/`\c` escapes align the FOLLOWING line left/right/center (gently-y7i) exactly as upstream `Node::_aligned_label` does: each escape terminates the previous segment (which is unescaped `\|` → `|`, `\\` → `\`, and trimmed), the segment's alignment is the PREVIOUS terminator's (`\n` means the object's default — node `center`, edge `left`), a trailing terminator adds no line, and multiline node boxes grow to lines+2 with the label rows vertically centered (upstream `_draw_label`'s middle alignment, `ys = 0.5`). Label width arithmetic counts UTF-8 BYTES (gently-bvb, re-derived: the oracle byte-lengths label text via its byte-string `length()` — `[ Köln ]` is a 7-dash box and the 中 pin is a 7-byte box, tests/repro/claims/ascii-render-tables.observed label wide glyph, label-byte-width.observed), superseding the earlier display-width reading recorded for gently-dcp. Re-derived (gently-emd): when the object's effective `textwrap` is not `none` (the object's attribute, else its class's, else the graph's — upstream defaults `inherit` over a graph default of `none`), wrapping REPLACES the escape split (upstream `_aligned_label` dispatches to `_wrapped_label`): every `\n`/`\r`/`\l`/`\c` becomes a space, whitespace collapses to single spaces, and the text wraps once a line reaches the wrap width at its LAST space (dropped) or last hyphen (kept on the line), whichever is later — a line with no break point never wraps; `auto` picks `int(sqrt(byte_length) * 1.4)` floored at 2, a number wraps after that many characters, and every wrapped row carries the object's default alignment (the escapes' alignments are lost) (tests/repro/claims/bg-textwrap.observed: the 33-byte node label wraps at 8, the edge's own `textwrap: 10` overrides the inherited mode, and the wrapped lines drive the box/gap sizing exactly as aligned lines do — node box to lines+2, labelled straight gap to widest line + 5); gently implements exactly this rule. | [[ge.ascii_render]] | |
| c6 | invariant | A node's `shape` attribute (upstream vocabulary: rounded, point, circle, ellipse, diamond, triangle, pentagon, hexagon, octagon, parallelogram, house, invisible, img — `box` is NOT valid; the default shape is `rect`) changes the rendered outline per the upstream ASCII shape table; color attributes do not affect ASCII output. Re-derived (gently-dcp): the probed table collapses to four distinct outlines — every non-special shape renders the plain box for small nodes, `rounded` blanks the corners, `point` swaps the label for a centered `*`, `invisible` draws nothing, and `shape: box` is upstream-rejected (tests/repro/claims/ascii-render-tables.observed, shape-outline-collapse.observed); re-derived (gently-cbk): `shape: edge` renders the node as an edge cell — upstream `Node::as_ascii` builds an `EDGE_HOR | EDGE_LABEL_CELL` cell from the node's label: a solid dash line across the cell's FULL column width on the band's middle row (no borders, no side glyphs), the label riding above it bottom-aligned in the cell's `h-2` field at byte offset 1 in a `w-2` field — the label rides the FRESH edge the cell renders through, whose `align` defaults LEFT, so the node's own align (center) is not consulted (`train`/`car` sit at the cell's offset 1) (tests/repro/claims/bg-edge-node.observed); gently implements exactly this collapse. | [[ge.ascii_render]] | |
| c7 | invariant | An anonymous node renders BOXLESS: the upstream `Node::Anon` class carries borderstyle `none` and the label `' '` (its `_correct_size` fixes the cell at 3×3), so the node's own cell renders blank and the edges terminate at the empty space around it (`[ ] --> [ Bonn ]` leaves the anon cell as 3 blank columns with the arrow centered in the gap); an edgeless anon node contributes no visible bytes. Anonymous nodes are the empty-name nodes of the model API plus the parser-generated `#N` names (gently-6j0's odd counter; upstream's parser rejects a literal `#` in node names, so a `#\d+` name can only be parser-generated). Re-derived (gently-b0z): gently previously rendered boxed `#1`/`#3` — the oracle bytes are tests/repro/claims/pod-anon-nodes.observed (also bg-anon-chain.observed); re-derived (gently-cbk): an anon node does not hold the render's trailing floor open — a trailing anon-only band contributes no blank lines, the oracle's output ends at the last drawn line (bg-edge-node.observed ends at the buy edge's run); gently implements exactly this rule. | [[ge.ascii_render]] | |
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
| p2 | unit | [[ge.ascii_render.c2]] | graphs with every edge style (bold/wide/broad as one aliased family), in straight gaps and chains, plus bidirectional (`<-->`, `<-- label -->`) and labeled-undirected edges straight and bent | each edge's glyph run matches the observed minimum-run style table byte-for-byte (arrowed and arrow-less), wide and broad alias the bold runs (`##>`/`###`), undirected runs shift with the gap's absolute column, dot-dot-dash widens its gap to 6, labeled undirected runs carry no head, bidirectional straight runs draw both outward heads at the widened gap with the label centered in [2, w-3), and a bent bidirectional edge stamps the start arrowhead into its source box; vertical runs attach at their edge cell's left+2 (the POD triangle's back-edge and every pinned wide-box vertical), a `minlen >= 2` straight edge fills its one-column-widened gap, an edge whose endpoint is a `shape: edge` node renders through the stripped endpoint flags (`--->` flush off the shape:edge source, the undirected `--------` chain through the edge cell, a directed `>` riding INTO the edge cell), and the vertical short cell heads its target-adjacent end (`^` into train's south) |
| p3 | unit | [[ge.ascii_render.c3]] | one graph rendered via two different construction orders | outputs are byte-identical |
| p4 | unit | [[ge.ascii_render.c4]] | graphs with labelled crossing edges, plus straight edges whose labels carry literal `\n` splits and `\r` alignment escapes (gently-527/y7i) | no label cell coincides with a node cell or another label cell; a multiline edge label renders one left-aligned row per line bottom-aligned in the band's `h-2` field at byte offset 2, the gap widening to the widest line + 5, the run staying on the grown band's `H-2` line, and both endpoint boxes growing one blank interior row — byte-identical to the pinned oracle bytes (tests/repro/claims/edge-label-split-minimal.observed); a straight vertical single-gap edge's label rides the gap band's middle line at byte offset 4 (bg-edge-node.observed: `buy`) |
| p5 | unit | [[ge.ascii_render.c5]] | node labels with long text, explicit `\n` breaks, `\l`/`\r`/`\c` alignment escapes, multi-byte glyphs (`中`, `Köln`), and `textwrap: auto` / numeric `textwrap` on graph and edge scopes (gently-emd) | each label line occupies one interior row placed by its `l`/`r`/`c` alignment (the escape aligning the FOLLOWING line), a multiline box grows to lines+2 with vertically centered rows, and all width arithmetic follows UTF-8 byte length — the 中 box is 7 bytes wide and Köln's 7-dash box renders byte-identically to the pinned oracle bytes (tests/repro/claims/label-byte-width.observed); with textwrap set, wrapping replaces the escape split and the wrapped rows size the boxes byte-identically to the pinned oracle bytes (tests/repro/claims/bg-textwrap.observed) |
| p6 | unit | [[ge.ascii_render.c6]] | nodes with every shape in the upstream vocabulary, with and without color attributes | each shape's outline matches the probed collapse (rounded/point/invisible diverge; every other non-edge valid shape renders the plain box; `box` is upstream-rejected; `edge` renders the edge cell — full-width dash line, label left-aligned above at offset 1) and color attributes leave the output byte-identical |
| p7 | unit | [[ge.ascii_render.c7]] | the POD anon chain (`[ ] --> [ Bonn ]`, `[ Bonn ] --> [ ]`) | the anon cells render blank (3 columns wide, no box, no `#N` label), the arrows attach to the empty space, and a trailing anon-only band contributes no blank lines — byte-identical to the pinned POD bytes (tests/repro/claims/pod-anon-nodes.observed) |
| p8 | unit | [[ge.ascii_render.c8]] | named groups over connected members — one row, stacked rows, three-member chains, long titles | each group's dashed frame (`+ - - +` corners, `'` sides) encloses its members with the 3-row title band above and the blank row + border below, the title renders the stored group name verbatim at the leftmost member's box x, and the title widens the leftmost member's column (`title+2`) with the label re-centered — byte-identical to the pinned POD bytes |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** nodes with every border style
- **THEN** each rendered box border matches its declared style, with bold/wide/broad sharing the same `#####` outline as observed
- **VERIFIES** [[ge.ascii_render.p1]]

#### Scenario: p2

- **WHEN** graphs with every edge style (bold/wide/broad as one aliased family), in straight gaps and chains, plus bidirectional (`<-->`, `<-- label -->`) and labeled-undirected edges straight and bent
- **THEN** each edge's glyph run matches the observed minimum-run style table byte-for-byte (arrowed and arrow-less), wide and broad alias the bold runs (`##>`/`###`), undirected runs shift with the gap's absolute column, dot-dot-dash widens its gap to 6, labeled undirected runs carry no head, bidirectional straight runs draw both outward heads at the widened gap with the label centered in [2, w-3), and a bent bidirectional edge stamps the start arrowhead into its source box; vertical runs attach at their edge cell's left+2 (the POD triangle's back-edge and every pinned wide-box vertical), a `minlen >= 2` straight edge fills its one-column-widened gap, an edge whose endpoint is a `shape: edge` node renders through the stripped endpoint flags (`--->` flush off the shape:edge source, the undirected `--------` chain through the edge cell, a directed `>` riding INTO the edge cell), and the vertical short cell heads its target-adjacent end (`^` into train's south)
- **VERIFIES** [[ge.ascii_render.p2]]

#### Scenario: p3

- **WHEN** one graph rendered via two different construction orders
- **THEN** outputs are byte-identical
- **VERIFIES** [[ge.ascii_render.p3]]

#### Scenario: p4

- **WHEN** graphs with labelled crossing edges, plus straight edges whose labels carry literal `\n` splits and `\r` alignment escapes (gently-527/y7i)
- **THEN** no label cell coincides with a node cell or another label cell; a multiline edge label renders one left-aligned row per line bottom-aligned in the band's `h-2` field at byte offset 2, the gap widening to the widest line + 5, the run staying on the grown band's `H-2` line, and both endpoint boxes growing one blank interior row — byte-identical to the pinned oracle bytes (tests/repro/claims/edge-label-split-minimal.observed); a straight vertical single-gap edge's label rides the gap band's middle line at byte offset 4 (bg-edge-node.observed: `buy`)
- **VERIFIES** [[ge.ascii_render.p4]]

#### Scenario: p5

- **WHEN** node labels with long text, explicit `\n` breaks, `\l`/`\r`/`\c` alignment escapes, multi-byte glyphs (`中`, `Köln`), and `textwrap: auto` / numeric `textwrap` on graph and edge scopes (gently-emd)
- **THEN** each label line occupies one interior row placed by its `l`/`r`/`c` alignment (the escape aligning the FOLLOWING line), a multiline box grows to lines+2 with vertically centered rows, and all width arithmetic follows UTF-8 byte length — the 中 box is 7 bytes wide and Köln's 7-dash box renders byte-identically to the pinned oracle bytes (tests/repro/claims/label-byte-width.observed); with textwrap set, wrapping replaces the escape split and the wrapped rows size the boxes byte-identically to the pinned oracle bytes (tests/repro/claims/bg-textwrap.observed)
- **VERIFIES** [[ge.ascii_render.p5]]

#### Scenario: p6

- **WHEN** nodes with every shape in the upstream vocabulary, with and without color attributes
- **THEN** each shape's outline matches the probed collapse (rounded/point/invisible diverge; every other non-edge valid shape renders the plain box; `box` is upstream-rejected; `edge` renders the edge cell — full-width dash line, label left-aligned above at offset 1) and color attributes leave the output byte-identical
- **VERIFIES** [[ge.ascii_render.p6]]

#### Scenario: p7

- **WHEN** the POD anon chain (`[ ] --> [ Bonn ]`, `[ Bonn ] --> [ ]`)
- **THEN** the anon cells render blank (3 columns wide, no box, no `#N` label), the arrows attach to the empty space, and a trailing anon-only band contributes no blank lines — byte-identical to the pinned POD bytes (tests/repro/claims/pod-anon-nodes.observed)
- **VERIFIES** [[ge.ascii_render.p7]]

#### Scenario: p8

- **WHEN** named groups over connected members — one row, stacked rows, three-member chains, long titles
- **THEN** each group's dashed frame (`+ - - +` corners, `'` sides) encloses its members with the 3-row title band above and the blank row + border below, the title renders the stored group name verbatim at the leftmost member's box x, and the title widens the leftmost member's column (`title+2`) with the label re-centered — byte-identical to the pinned POD bytes
- **VERIFIES** [[ge.ascii_render.p8]]

