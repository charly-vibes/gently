# Subagent brief: gently-tyv — visual oracle-vs-gently evidence in known-hard-cases

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-tyv` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is docs-only (complexity:s) — if you are not editing the doc
within 10 minutes, you are off track. Ship early rather than polishing.

## Orientation (max 4 tool calls)

- `bd show gently-tyv`
- Read `docs/known-hard-cases.md` (64 lines).
- `ls tests/repro/claims/` and cat the evidence files named below.

## Work

`docs/known-hard-cases.md` rows are 80–120-word prose walls with ZERO visual
evidence. For each of the 5 open rows (`gently-4ga`, `gently-58m`,
`gently-fb1`, `gently-awd`, `gently-byq`) add a compact ~3-line evidence
block: the minimal input snippet, then oracle bytes vs gently bytes
side-by-side — or oracle-only with a one-line gently delta where gently
errors (e.g. 4ga's `edge 2 path cells (0, 1) -> (0, 3) are not orthogonally
adjacent`).

Evidence sources (real recorded bytes — one cat away, do NOT invent):

- `gently-4ga` → `tests/repro/claims/bg-comma-lists.observed` (as_ascii section;
  the oracle renders the fan fine; gently errors — you can regenerate gently's
  error with the CLI: `cargo run -p gently-cli -- --as ascii` on the snippet)
- `gently-58m` → `tests/repro/claims/pod-anon-group.observed`
- `gently-awd` → `tests/repro/claims/so-flowchart-size.observed`
- `gently-byq` → `tests/repro/claims/so-start-port-index.observed`
  (+ `port-side-minimal.observed` for the already-ported unindexed form)
- `gently-fb1` → no .observed file exists. Either record one via the probe
  harness (`PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl
  tests/repro/probes.pl claim <name>` — each example its OWN graph, one claim
  per graph) and cite it, or use oracle-only bytes captured from the harness
  output plus a one-line gently delta (gently keeps x ungrouped).

Keep the prose cell to ≤2 sentences and let the snippet carry the detail.
Keep the bead/probe ids (use code spans; link only if the file already has a
link convention for them). Choose a markdown shape that renders correctly —
if fenced blocks can't live inside table cells, put the table row short and
the evidence block immediately after the table (or per-row subsections); the
page must still render sensibly on GitHub. Do NOT restructure the rest of
the page, do NOT touch closed/resolved rows, do NOT change facts or fix
the divergences themselves.

## Gates (run all; all must be green)

- `just gates`
- `just examples-verify` (must stay byte-identical — you changed no code)
- `ah check` (0 findings)

## Ship

1. Commit ONLY the doc (plus a new .observed + probes.pl claim if you recorded
   one for fb1): `docs: visual oracle-vs-gently evidence per known-hard-cases row (gently-tyv)`
2. `bd close gently-tyv`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: what changed per row, gate results, commit sha.
