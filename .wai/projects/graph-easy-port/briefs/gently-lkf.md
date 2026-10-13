# Subagent brief: gently-lkf — README reorder: quickstart + formats table before maintainer gates

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-lkf` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is docs-only (complexity:s) — if you are not editing README.md
within 10 minutes, you are off track. Ship early rather than polishing.

## Orientation (max 4 tool calls)

- `bd show gently-lkf`
- Read `README.md` (58 lines).
- Verify the runtime facts below with the CLI if anything looks off.

## Work

README.md serves maintainer CI gates (`just lint/gates/oracle-verify/
perf-check/examples-verify`, currently before Quickstart) BEFORE the user's
first success. A first-time human meets CI tooling before rendering their
first graph. Reorder:

1. **Quickstart first** — build + render steps and the exact `--as ascii`
   output block stay byte-for-byte what they are today.
2. **Then a small supported-formats table** — parse: text + dot; render via
   CLI: ascii; render via library: boxart/HTML/Graphviz DOT (CLI wiring
   pending). This table carries the detail the Status paragraph currently
   buries.
3. **Then the gates block under a "For contributors" heading.**
4. **Collapse the Status paragraph to ≤2 lines** — keep the wai badge and
   the pinned-oracle fact (v0.69, `just oracle-verify`); the formats table
   carries the surface detail.

Runtime-verified facts you MUST preserve exactly:

- the quickstart `--as ascii` output bytes (the Bonn/Berlin/Hamburg box)
- the CLI format fact: unknown format errors with `gently: unknown output
  format`, `valid formats: ascii` (CLI wiring for other formats is
  gently-4ig's job — do not promise CLI formats beyond ascii)

## Co-modification constraints (PRE-002)

README.md is also touched by gently-cof (jargon links — runs AFTER this) and
gently-irk (gallery link — runs after both). Keep section anchors/heading
names stable so the later edits apply cleanly. Do NOT add jargon
definitions/links (cof's job), do NOT add a gallery (irk's job).

## Gates (run all; all must be green)

- `just gates`
- `just examples-verify` (must stay byte-identical — you changed no code)
- `ah check` (0 findings)

## Ship

1. Commit ONLY README.md: `docs(readme): quickstart + supported-formats table before maintainer gates (gently-lkf)`
2. `bd close gently-lkf`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: the new section order, gate results, commit sha.
