# Subagent brief: gently-ni9.3 — mine GitHub issues on ironcamel/Graph-Easy

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-ni9.3` and is hands-off: you own it
end-to-end. You may file follow-up bug beads for divergences; do NOT close
tickets other than gently-ni9.3.

## TIME BUDGET (binding)

HARD 30-minute budget (`timeout 1800` enforced). Orientation ≤5 tool calls.
Triage incrementally and commit early — a partial committed triage with
verified repros beats an unfinished full sweep.

## Orientation (max 5 tool calls)

- `bd show gently-ni9.3` and `bd show gently-ni9`
- `curl -s https://api.github.com/repos/ironcamel/Graph-Easy/issues?state=all&per_page=100`
  (unauthenticated, 60 req/hr budget — list once, then fetch only bodies of
  issues with reproducible-looking graph inputs; ~5-10 issue fetches max).
  Also `rt.cpan.org` has the older queue; skip it if slow — GitHub is the target.
- `tests/repro/probes.pl` + `tools/oracle.pl` usage; `tests/repro/claims/` for
  the .observed format (see existing pod-* files from gently-ni9.1).
- Byte-compat scope decision:
  `.wai/projects/graph-easy-port/designs/2026-10-09-decision-byte-compat-scope.md`

## Work

1. List issues; filter to ones containing concrete graph text/DOT inputs,
   rendering or parsing complaints (skip pure feature requests without input).
2. For each reproducible-looking case: extract the minimal input, run it
   against the PINNED oracle
   (`PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tools/oracle.pl ...`
   or a probes.pl claim) — the oracle v0.69 behavior is the contract, not
   what upstream fixed later.
3. Compare gently's output for the same input (cargo run / the CLI).
   Outcomes: (a) gently matches pinned oracle → record probe claim as a
   hard-case regression pin; (b) gently diverges → file bug bead with probe
   name + both outputs; (c) oracle itself crashes/undefined → record under
   scope-off in the notes file, no fixture.
4. Write triage table to
   `.wai/projects/graph-easy-port/research/2026-10-11-gh-issues-triage.md`
   (issue #, input shape, oracle verdict, gently verdict, action).

## Must gate (from bd acceptance)

- Triage covers all GitHub issues with reproducible inputs (state=all)
- Each still-reproducing case has a probes.pl claim + .observed evidence
- Divergences filed as beads with probe names; zero unverified claims published
- Meter: `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl claims`

## Gates (all green)

- `ah check` (0 findings), `cargo test` (146), `just oracle-verify` (unchanged
  is fine — this task records claims, not fixtures)

## Ship

1. One commit: triage notes + claims/.observed evidence + beads filed.
   Reference gently-ni9.3.
2. `bd close gently-ni9.3`
3. Do NOT push — orchestrator pushes.
4. Print final summary: issues triaged, repros pinned, divergences filed, gates.