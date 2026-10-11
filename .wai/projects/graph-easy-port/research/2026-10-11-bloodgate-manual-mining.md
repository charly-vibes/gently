# bloodgate.com + Graph::Easy::Manual POD mining — gently-ni9.2

Date: 2026-10-11 · Owner: subagent for gently-ni9.2 · Budget: 30 min (held)

## Sources mined (all HTTP 200, live)

- `http://www.bloodgate.com/perl/graph/` — plugin landing page (menu only)
- `manual/index.html` — Manual index; links confirm the manual pages are the
  rendered **Graph::Easy::Manual POD** (`search.cpan.org/~tels/Graph-Easy-Manual`)
  → site mining and Manual POD mining are the same corpus. The metacpan
  fastapi release lookup for `SHLOMIF/Graph-Easy-Manual` returned
  `{"release": null}` (no indexed release under that author path); instead of
  burning budget on the dist tarball, the rendered POD pages were mined
  directly — same content, live.
- `manual/syntax.html` (20 KB, 48 pre-blocks) — richest page: comments,
  multiline labels, `\l \r \c` alignment, textwrap, comma-chained lists,
  anon nodes, quoting/escapes (`;` `#` `"`), attribute scoping, joint nodes
  (`[ A | B ]`), edge-style shortcut table, edge labels, groups
- `manual/tutorial_bypass.html` (12 pre-blocks) — minlen, origin/offset,
  start:/end: ports
- `manual/tutorial_edges.html` (8 pre-blocks) — edge-as-node (`shape: edge`)
- `manual/tutorial.html`, `tutorial_tree.html`, `faq.html`, `overview.html`,
  `attributes.html` — prose pages, no additional renderable pairs
- `fun.html`, `screenshots.html` — no graph snippets (screenshots are images)

## Deliverables

### Fixtures added (pass tier-1 admission + `just oracle-verify`, 14 inputs total)

| fixture | source | gently vs oracle |
|---|---|---|
| `bg-chain-continue` | syntax.html line-continuation chain `[ A ] -> [ B ] -> [ C ] ↵ -> [ D ] ↵ -> [ E ]` | byte-identical |
| `bg-dotted-style` | syntax.html dotted edge `--> { style: dotted; }` | byte-identical |

### Claims recorded (oracle-verified observations, `tests/repro/claims/bg-*.observed`)

All render under the pinned oracle v0.69 @ ededa3d7 across seeds 0/1; the
divergence vs gently is named in each probe comment. Registered in
`tests/repro/probes.pl` %SERVES/%CLAIMS (bloodgate-mining section, pattern of
ni9.1/ni9.3).

| claim | source | outcome |
|---|---|---|
| `bg-multiline-align` | syntax.html | oracle: `\l \r \c` = per-line alignment, `\n` splits label; gently prints escapes raw → bead |
| `bg-comma-lists` | syntax.html | oracle lays `[ A ] -> [ B ], [ C ], [ D ]` / `[ A ], [ B ], [ C ] --> [ D ]`; gently parse-errors on `,` → bead |
| `bg-trailing-node-attr` | syntax.html | oracle: trailing `{ label: "Huge"; }` renames node; gently drops the block → bead |
| `bg-anon-chain` | syntax.html | oracle hides anon nodes; gently draws boxed `#1`/`#3` → bead |
| `bg-minlen-bypass` | tutorial_bypass.html | oracle honors `minlen: 3` (stretched top route); gently ignores → bead |
| `bg-edge-node` | tutorial_edges.html | oracle dissolves `shape: edge` nodes into edges; gently render-error (path cells) → bead |
| `bg-joint-classes` | syntax.html | **oracle-side divergence**: pinned v0.69 parser REJECTS `class: \|1` — site is rendered by a newer Graph::Easy; site bytes NOT pin-reachable (no gently bug) |
| `bg-start-right` | tutorial_bypass.html | oracle honors `{ start: right; }` (Trier inline right of Koblenz); gently ignores ports → bead |
| `bg-textwrap` | syntax.html | oracle wraps at 10 cols (`textwrap: 10`; note site shows `text-wrap:` — newer attr name, pinned parser wants `textwrap`); gently doesn't wrap → bead |

### Divergences (divergence ≠ fixture because gently ≠ oracle on them)

Bug beads filed (all P2, open): anon nodes visible (`gently-…`), shape:edge
path-cell render error, minlen ignored, `\l \r \c` raw, comma-list parse
error, trailing node attr dropped, start:/end: ports ignored, textwrap
ignored, **class-scoped node rules render empty output** (observed in a
passing-fixture probe session, `x-classes`: gently produced NO output at all
for `node.city { color: red; }` + classed node — not yet claim-probed).

## Dedupe outcomes (vs claims/, fixtures/, examples/)

- Site's basic forms already covered, no new pairs: single node, `A->B->C`
  chain (fixtures `chain`, example `simple-chain`), fan-out (claim
  `pod-fan`), fan-in shared target (fixture `txt-shared-target`), diamond,
  selfloop, isolated node, inline edge attrs (example `edge-attributes`,
  claim `pod-inline-label`), edge-style shortcut table (claims
  `pod-edge-styles`/`pod-style-family`/`pod-double-dash`), groups (claims
  `group-syntax`/`pod-group`), quoting-`"` (claim `attr-quote-value`),
  `#`-escapes (claims `sharp-escape`/`sharp-label`), anon numbering (claims
  `anon-numbering`/`pod-anon-nodes`), dot-direction (claim `dot-direction`).
- Net-new pairs published: 2 fixtures + 9 claims = **11 oracle-verified
  pairs**, every one with committed expected bytes.

## Divergences where the SITE's shown bytes differ from the pinned oracle

- `bg-edge-node`: site shows a single-row box layout; pinned oracle routes
  the dissolved edges differently. Oracle bytes recorded (contract).
- `bg-minlen-bypass`: site shows Trier below; pinned oracle routes the
  minlen-stretched bypass across the TOP. Oracle bytes recorded.
- `bg-joint-classes`: site renders per-cell border styles; pinned parser
  rejects the input outright (see claims table).
- `bg-textwrap`: site uses `text-wrap:` attr name; pinned parser accepts
  `textwrap` only (recorded companion uses the pinned spelling).

## Gates at ship time

- `ah check` → 0 findings (ok:true)
- `cargo test` → 146 passed, 0 failed
- `just oracle-verify` → green (14 inputs, pin v0.69 @ ededa3d7)
- `just repro` → admission manifest rewritten, all new fixtures stable

Beads: gently-ni9.2 closed by orchestrator brief; 9 follow-up bug beads
filed (see `bd list`).
