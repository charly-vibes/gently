# POD mining notes — gently-ni9.1 (Graph::Easy, Parser; v0.69 pin)

Date: 2026-10-11 · Source: pinned oracle checkout
`/var/tmp/ge0.69/Graph-Easy-0.69/lib/Graph/Easy{.pm,/Parser.pm}`.
The Graph::Easy::Manual dist is NOT part of the Graph-Easy tarball; archive.org
fetch attempts returned 503 — Manual/FAQ/cookbook mining deferred (see beads).

## Method

1. Extracted every runnable text-graph example from Easy.pm SYNOPSIS/EXAMPLES
   and Parser.pm SYNOPSIS/Input/Input-Syntax sections.
2. Deduped against `tests/fixtures/graph-easy/` (chain, parallel, selfloop,
   mixed_isolated, txt-diamond, txt-isolated, txt-shared-target, tracer
   already cover: chains, parallel edges, self-loops, disconnected parts).
3. Each candidate ran through tier-1 admission (`probes.pl admit`); stable +
   byte-compatible ones became corpus fixtures; everything else became a
   `tests/repro/claims/pod-*.observed` claim probe (oracle evidence under 2
   hash seeds) + a divergence bug bead.

## Outcome

- **Compatible fixtures landed (8 pairs):** `pod-one-node`, `pod-bold-style`,
  `pod-bold-label`, `pod-inline-styles` — all verified green by
  `just oracle-verify`.
- **13 claim probes recorded** in `tests/repro/claims/` (`pod-*.observed`),
  each showing the pinned oracle's render under hash seeds 0 and 1.

## Divergences found (gently vs oracle) — each with a filed bead

| # | POD example | Oracle | gently | bead |
|---|-------------|--------|--------|------|
| 1 | `( [ Bonn ] ) -> [ Berlin ]` anon group | parses | `'- ' not recognized` parse error | gently-58m |
| 2 | `[ ] -> [ Bonn ] -> [ ]` anon nodes | anon nodes drawn boxless, unnamed | boxed `#1`/`#3` visible | gently-b0z |
| 3 | `[ C ] .-.- [ D ]` arrowless dot-dash | `-.-` | `---` | bead (arrowless style) |
| 4 | `( Cities: … )` named group | dashed `+ - - -+` border + title | no border | bead |
| 5 | `[ A ] <-- label --> [ B ]` | arrows both ends, label above | `|` start, label inline | bead |
| 6 | `[ AB ] -- { label: … } [ CD ]` | no arrowheads | spurious `>` | bead |
| 7 | `[ C ] ..-..-..-> [ D ]` repeats | `..->` dot-dot-dash kept | `-->` collapsed | bead |
| 8 | interlinked triangle | 2-col narrower back-edge | layout drift | bead |
| 9 | `[ Bonn ] --> [ Köln ]` UTF-8 | width = UTF-8 **bytes** | width = chars | bead |
| 10 | `{ style: wide/broad; }` | `##>` | `-->` | bead |
| 11 | `[ Bonn ] <-> [ Berlin ]` | `<-->` | `-->` (same root cause as #5) | claim only |
| 12 | `[ A ] = > [ B ]` double-dash | `= >` round-trips | `-->` | bead |

## Hash-dependent (out of byte-compat scope, ge.oracle.c7)

- `pod-edge-styles` (Easy.pm "Different edge styles" 8-edge block)
- `pod-fan` (Easy.pm "Three nodes" fan)
- `pod-inline-label` (Parser.pm inline label examples)
- `pod-parser-input` (Parser.pm Input section)

These ship as claim probes showing divergence across seeds — never fixtures.

## Edge cases surfaced by POD prose (not fixture-able here)

- **Quoting:** POD says nodes are "quoted" with `[ ]`; attribute values may be
  quoted or unquoted (`{ label: foo; }` vs `{ label: "foo bar"; }`) —
  already covered by claims `attr-quote-value`, `sharp-escape`.
- **Self-loops:** `add_edge('Bremen','Bremen')` (Easy.pm prose) — covered by
  `selfloop` fixture.
- **Unicode:** input "encoded in UTF-8" (Parser.pm Input) — the byte-vs-char
  width bug above is the concrete consequence.
- **Edge style grammar:** patterns >1 char must repeat fully
  (`..-..-..->` valid, `..-..-..>` invalid; `.-.->` invalid) — parser-level
  validation worth a negative-parse test (no fixture; follows from bead above).
- **Arrow-less edges cannot use style shortcuts** (`~` alone invalid; needs
  `~~` / `.-.-`).
- **Inline labels only with arrows** (`-- label --` invalid; use attributes).
- **Groups:** only nodes are members; edges join implicitly when both
  endpoints are in the group (Parser.pm groups prose).
- **VCG/graphviz magic detection:** only for `digraph`-prefixed/named graphs
  (Parser.pm POD) — dot side covered by `subgraph-handling`, `dot-*` claims.

## Manual POD (deferred)

`Graph::Easy::Manual` is a separate CPAN dist (0.76 era); the pinned
Graph-Easy-0.69 checkout has no Manual POD. Fetch attempts (archive.org,
metacpan) got HTTP 503 within budget. Follow-up: retry fetch or use
`Graph::Easy::Manual` on CPAN mirror for tutorial/FAQ/cookbook mining —
richest remaining source per the epic.
