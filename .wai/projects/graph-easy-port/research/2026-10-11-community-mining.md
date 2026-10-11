# Community Q&A mining — gently-ni9.4 (2026-10-11)

Goal: mine Stack Overflow / PerlMonks / Reddit / blogs for real-world
graph-easy usage with concrete graph inputs; dedupe against the corpus;
verify each new snippet against the pinned oracle v0.69.

## Sources searched

| Source | Query / endpoint | Result |
|---|---|---|
| Stack Overflow | `api.stackexchange.com/2.3/search/advanced q="graph-easy"` | 16 hits, total field absent; fetched bodies of 9 questions + their answers |
| Reddit | `reddit.com/search.json?q="graph-easy"` | BLOCKED — non-JSON response (bot block, UA header sent); abandoned |
| PerlMonks | `index.pl?node=Search` / `node_id=3989` / `node=Graph::Easy` | BLOCKED — all three serve the same 3038-byte challenge page to scripts; abandoned |
| Blogs | `html.duckduckgo.com/html q="graph-easy" perl ascii example blog` | 14 KB response, zero result links extracted (challenge/empty page); abandoned |

Stack Overflow was the only programmatically accessible source; it
yielded the ≥5 pairs. Reddit/PerlMonks remain unmined — follow-up beads
could retry with a headless browser or their official Super Search form.

## Posts examined (SO)

Question IDs fetched with bodies: 47323345 (flowchart), 71869025
(Graph.pm→Graph::Easy), 47661180 (edge weights), 59659560 (dot→ascii),
834395 (python ascii), 3921931 (C structs), 41799279 (akka), 2742644
(node coords), 19053970 (dot large graphs) plus all answers for the
first five.

Dedupe outcomes:
- 3921931, 41799279, 2742644, 19053970: graph-easy mentioned only in
  passing or no graph-easy input snippet — no pairs.
- 834395: accepted answer is hand-drawn art, not graph-easy output; the
  `[a]---[b]` dotted-edge class it implies is already covered by
  fixture bg-dotted-style — deduped, no pair.
- 59659560: real dot dump (go-callvis); mined as the
  so-graphviz-unknown-attrs claim (module-level Parser::Graphviz
  behavior — unknown attrs WARN and render anyway; gently has no dot
  CLI path, and upstream's CLI has none either, so no bug bead).
- 47661180 question body: invalid `frequency` custom attribute — the
  ANSWER's relabeled version is the pair (so-edge-labels).

## Verified pairs recorded (5 claim probes, tests/repro/claims/)

| Probe | Source | Divergence? |
|---|---|---|
| so-graph-convert | SO 71869025 a71869162 (accepted) | YES → gently-5vf (stem column off by one) |
| so-edge-labels | SO 47661180 a47661977 | none — byte-identical to the posted output |
| so-flowchart-size | SO 47323345 question body | YES → gently-527 (`\n` label raw, layout) |
| so-start-port-index | SO 47323345 a48119766 | YES → gently-byq (indexed ports ignored) |
| so-graphviz-unknown-attrs | SO 59659560 | n/a (module-level; no gently dot CLI to diverge) |

All recorded under `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib` across
hash seeds 0 and 1; hash-stable. Not added as corpus fixtures — per the
brief, fixtures are for tier-1 admission classes; these are claim
probes (several diverge in gently, so they could not be admitted as
byte-comparison fixtures anyway).

Notable community-usage observations: real users hit indexed port
attributes (`{start: front,0}`) as an ordering workaround, paste dot
dumps with unknown attributes, and use `{ size: w,h; }` blocks — none
of which appear in POD. The zero-space `{attrs}[Node]` junction parses
clean in both oracle and gently.

## Divergence bug beads filed

- gently-5vf — so-graph-convert stem column off by one
- gently-527 — so-flowchart-size literal `\n` printed raw
- gently-byq — so-start-port-index indexed port attrs ignored

## Gates

- `just repro-claims` lists the new probes (run post-commit)
- `ah check` — 0 findings
- `cargo test` — 146 pass
- `just oracle-verify` — green (no fixture changes; claims only)
