# Subagent brief: gently-ni9.1 — mine upstream POD (Graph::Easy, Parser, Manual)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-ni9.1` and is hands-off: you own it
end-to-end. You may also file follow-up beads for divergences you discover
(create with `bd create ... -t bug --metadata '{"files":[...]}'`; do NOT close
them unless you fix+verify them yourself — prefer filing over fixing here).

## TIME BUDGET (binding)

You have a HARD 30-minute budget enforced by `timeout 1800`. Orientation ≤5
tool calls. Work incrementally: record findings as you go (a notes file +
committed pairs), never batch everything for the end. A partial, committed,
verified result beats an unfinished full sweep.

## Orientation (max 5 tool calls)

- `bd show gently-ni9.1` and `bd show gently-ni9` (parent epic, source rules)
- `ls /var/tmp/ge0.69/Graph-Easy-0.69/lib/Graph/Easy/` — the PINNED oracle
  checkout is the mining source. Its POD is exactly the documentation the
  oracle implements. Prefer it over the network.
- Read the POD files found there (Manual book too if present under
  lib/Graph/Easy/Manual* or blib; if Manual POD is absent, fetch from
  web.archive.org via curl — see ni9.2's brief style — but do not sink >5 min
  into fetching).
- `tests/repro/probes.pl` usage + `tests/fixtures/graph-easy/README.md`
  (fixture layout + pin headers) + `tools/oracle.pl` record/verify modes.
- Byte-compat scope: `.wai/projects/graph-easy-port/designs/2026-10-09-decision-byte-compat-scope.md`
  — only propose fixtures for classes that pass tier-1 admission (hash-stable
  + <10s); anything else becomes a documented example WITHOUT a fixture or a
  filed follow-up bead.

## Work

1. Extract every runnable example from the POD: synopses, tutorial snippets,
   FAQ answers, cookbook recipes, attribute-reference tables with examples.
2. Dedupe against existing corpus: `ls tests/fixtures/graph-easy/` and
   `perl tests/repro/probes.pl claims`.
3. For each NEW example: run it through the pinned oracle
   (`PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tools/oracle.pl ...`
   or probes.pl) and record input + output as a probe claim (claims/ dir) or
   corpus fixture per the fixture README layout.
4. Log edge cases POD prose reveals (quoting, self-loops, unicode, groups,
   flow, edge styles) even where you can't build a fixture — write them to
   `.wai/projects/graph-easy-port/research/2026-10-11-pod-mining-notes.md`.
5. If gently diverges from the oracle on any example: file a bug bead with
   the probe name as evidence. Do not fix implementation code.

## Must gate (from bd acceptance)

- ≥10 oracle-verified input/output pairs recorded (probes claims + .observed
  files, or fixtures)
- `just oracle-verify` passes with any new fixtures
- zero unverified pairs published
- Meter: `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl claims && just oracle-verify`

## Gates (all green before ship)

- `ah check` (0 findings)
- `cargo test` (146 passing; new corpus fixtures add tb::corpus tests — keep green)
- `just oracle-verify` (if new fixtures landed)

## Ship

1. Commit in two: (a) mining notes + probe claims/.observed evidence,
   (b) fixtures + any corpus test wiring. Reference gently-ni9.1 in messages.
2. `bd close gently-ni9.1`
3. Do NOT push — orchestrator pushes.
4. Print final summary: pairs recorded, edge cases found, beads filed, gates.