# Subagent brief: gently-ni9.2 — mine bloodgate.com + Graph::Easy::Manual POD

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-ni9.2` and is hands-off: you own it
end-to-end. You may file follow-up beads; do NOT close other tickets.

## TIME BUDGET (binding)

HARD 30-minute budget (`timeout 1800`). Orientation ≤5 tool calls. Work
incrementally, commit early. Network IS available (bloodgate.com verified
live moments ago — HTTP 200). If a specific URL 503s, move on and note it;
do not retry loops.

## Orientation (max 5 tool calls)

- `bd show gently-ni9.2` and `bd show gently-ni9`
- `curl -s -m 15 http://www.bloodgate.com/perl/graph/` — the tutorial/index
  page; follow gallery/example links (tutor.html, examples, gallery).
- The Graph::Easy::Manual POD was DEFERRED from gently-ni9.1 (separate CPAN
  dist, absent from the pinned checkout). Fetch it from metacpan's source
  API, e.g.:
  `curl -s https://fastapi.metacpan.org/v1/source/SHLOMIF/Graph-Easy-Manual/...`
  or the dist page https://metacpan.org/dist/Graph-Easy-Manual — grab the
  .pod/.pm POD and mine FAQ/cookbook/tutorial chapters. Cap total fetch +
  read time at ~10 minutes.
- Existing corpus to dedupe against: `tests/repro/claims/` (pod-*, gh-*),
  `tests/fixtures/graph-easy/`, `examples/` (gently-ni9.5 added 7).
- Verification toolchain: `tests/repro/probes.pl`, `tools/oracle.pl`,
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib`.

## Work

1. Walk the bloodgate tutorial + gallery pages; extract graph input snippets
   and their rendered output (the site shows ASCII/boxart demos — capture
   both input and shown output).
2. Mine the Manual POD: FAQ entries, cookbook recipes, attribute examples.
3. Dedupe against ALL existing claims/fixtures/examples. Only NEW pairs are
   deliverables.
4. Verify each new pair: run the input through the pinned oracle (v0.69 —
   the contract, NOT what the live site's renderer, which may be newer,
   shows). Where the site's shown output differs from the pinned oracle,
   record the oracle's bytes and note the divergence in the notes file.
5. Record verified pairs as probes.pl claims (claims/*.observed) or — where
   they pass tier-1 admission and are fixture-class — new corpus fixtures.
6. Notes: `.wai/projects/graph-easy-port/research/2026-10-11-bloodgate-manual-mining.md`
   (URLs visited, snippets found, dedupe outcomes, divergences).

## Must gate (from bd acceptance)

- ≥5 oracle-verified pairs from site/Manual material OR documented
  dedupe-vs-existing outcome if the sources add nothing new
- every published pair has committed expected output
- Meter: `just oracle-verify` passes with any new fixtures

## Gates (all green)

- `ah check` (0), `cargo test` (146), `just oracle-verify` (if fixtures added),
  `just examples-verify` (if examples/ extended — expected.txt must be
  oracle-pinned bytes)

## Ship

1. One commit: notes + claims/.observed + any fixtures/examples.
   Reference gently-ni9.2.
2. `bd close gently-ni9.2`
3. Do NOT push — orchestrator pushes.
4. Print final summary: pairs recorded, sources mined, divergences noted, gates.