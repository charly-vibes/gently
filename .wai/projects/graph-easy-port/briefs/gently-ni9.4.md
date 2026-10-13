# Subagent brief: gently-ni9.4 — mine community Q&A for real-world graph-easy usage

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-ni9.4` and is hands-off: you own it
end-to-end. You may file follow-up beads; do NOT close other tickets.

## TIME BUDGET (binding)

HARD 30-minute budget (`timeout 1800`). Orientation ≤5 tool calls. Work
incrementally; commit before the budget ends even if coverage is partial —
the acceptance allows a documented partial-outcome.

## Orientation (max 5 tool calls)

- `bd show gently-ni9.4` and `bd show gently-ni9`
- Dedupe corpus: `tests/repro/claims/` (pod-*, gh-*, bg-*), `tests/fixtures/graph-easy/`,
  `examples/`. Verification: `tests/repro/probes.pl`, `tools/oracle.pl`,
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib`.
- Network IS available. curl is your search/fetch tool.

## Work

1. Search for real-world graph-easy usage with concrete graph inputs:
   - Stack Overflow: `curl -s "https://api.stackexchange.com/2.3/search/advanced?order=desc&sort=relevance&q=graph-easy&site=stackoverflow&pagesize=50"`
     then fetch bodies of questions with graph snippets
     (`/questions/{ids}/body?site=stackoverflow&filter=withbody`).
   - PerlMonks: `curl -s "https://perlmonks.org/index.pl?node_id=..."` or its
     search (`node=Search` pages) for Graph::Easy nodes.
   - Reddit r/perl: `curl -s "https://www.reddit.com/search.json?q=%22graph-easy%22&limit=50"`
     (user-agent header required).
   - Blogs: duckduckgo html search `curl -s "https://html.duckduckgo.com/html/?q=%22graph-easy%22+ascii+graph+example"`.
2. Extract graph input snippets (people paste real messy graphs: unicode,
   mixed quotes, huge labels, attribute soup). Dedupe against existing corpus.
3. Verify each NEW snippet against the PINNED oracle v0.69 (the contract —
   community posts may show older/newer output; oracle bytes win).
4. Record verified pairs as probes.pl claims; fixtures only for tier-1
   admission classes. If gently diverges: file bug bead with probe name.
5. Notes: `.wai/projects/graph-easy-port/research/2026-10-11-community-mining.md`
   — queries run, posts examined, dedupe outcomes, divergences.

## Must gate (from bd acceptance)

- ≥5 oracle-verified pairs from community sources OR documented
  no-new-findings outcome WITH the search log (queries + result counts +
  dedupe rationale per post)
- every published pair has committed expected output
- Meter: `just oracle-verify` passes with any new fixtures

## Gates (all green)

- `ah check` (0), `cargo test` (146), `just oracle-verify` (if fixtures),
  `just examples-verify` (if examples/ extended)

## Ship

1. One commit: notes + claims/.observed + any fixtures/examples.
   Reference gently-ni9.4.
2. `bd close gently-ni9.4`
3. Do NOT push — orchestrator pushes.
4. Print final summary: pairs recorded, sources searched, divergences filed, gates.