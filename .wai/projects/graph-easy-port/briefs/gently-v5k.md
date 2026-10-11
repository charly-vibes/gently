# Subagent brief: gently-v5k — fix README status drift

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-v5k` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is small (complexity:s) — if you are not writing the fix within
10 minutes, you are off track. Ship early rather than polishing.

## Orientation (max 4 tool calls)

- `bd show gently-v5k`
- Read `README.md` (22 lines).
- `git log --oneline -3` for status facts.

## Work

README.md claims "Status: specification phase ... no implementation yet".
That is stale: the full pipeline shipped (parse → model → layout → ascii/
boxart/html/txt/graphviz renderers → CLI), 146 cargo tests green, all
capability beads closed. Rewrite the Status paragraph to reflect reality:

- Status is: initial capability wave complete, byte-compat gated against the
  pinned Perl oracle (v0.69, `just oracle-verify`), perf-gated
  (`just perf-check`).
- Keep the existing tone, brevity, and the wai badge; do NOT restructure the
  file beyond the status paragraph and do NOT add an examples section (that
  is gently-ni9.5's job — do not touch it).
- Do not invent claims you cannot verify from the repo (e.g. version numbers,
  crates.io publication).

## Gates (run all; all must be green)

- `just lint` (or `spk lint specs/` if justfile target differs)
- `ah check` (0 findings expected)
- `cargo test` (146 passing)
- `just gates` if cheap — optional for a README-only change

## Ship

1. Commit ONLY README.md: `docs(readme): status now reflects shipped capability wave (gently-v5k)`
2. `bd close gently-v5k`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: what changed, gate results, commit sha.