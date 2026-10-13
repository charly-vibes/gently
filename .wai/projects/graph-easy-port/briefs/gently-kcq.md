# Subagent brief: gently-kcq — examples/README: add the cargo build step

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-kcq` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is a one-line docs fix (complexity:xs) — if you are not editing
within 10 minutes, you are off track.

## Orientation (max 3 tool calls)

- `bd show gently-kcq`
- Read `examples/README.md` (~33 lines).

## Work

`examples/README.md` line ~33 tells readers to run
`gently --as ascii < examples/simple-chain/input.gv` but only the main
README's quickstart has the `cargo build -p gently-cli` step. A foraging
reader landing directly on `examples/` gets a command-not-found failure.

Fix: add the build step (or a one-line "see the README quickstart to build"
pointer) immediately BEFORE the run instruction. Pick whichever reads better
in place; if you add the build command, mirror the main README's exact
invocation (`cargo build -p gently-cli`).

## Co-modification constraints (PRE-002)

`examples/README.md` is also touched by gently-zf5 (how-to guide, runs AFTER
this) and gently-irrelevant gallery ticket later. Keep the surrounding
structure stable; do NOT add a how-to section (zf5's job), do NOT add a
gallery link, do NOT touch other files.

## Gates (run all; all must be green)

- `just gates`
- `just examples-verify` (must stay byte-identical — you changed no code)
- `ah check` (0 findings)

## Ship

1. Commit ONLY examples/README.md: `docs(examples): build step before the run instruction (gently-kcq)`
2. `bd close gently-kcq`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: what changed, gate results, commit sha.
