# Subagent brief: gently-vhc — decision-trace wrap-up (docs row + archive)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-vhc` and is hands-off: you own
the work end-to-end. Do NOT push and do NOT `bd close` — the orchestrator
owns ship.

## TIME BUDGET (binding): 20 minutes. Small slice.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=decision-trace`
- `bd show gently-vhc`
- `docs/known-hard-cases.md` — find the workflow/roadmap section; add a
  row referencing the decision-trace pipeline.
- `openspec/AGENTS.md` + `openspec --help` — the archive procedure
  (`openspec archive add-decision-trace` moves the change into specs/).
- State check: tasks.md sections 1,2,3,5 are all [x]; 4/6 were done
  earlier. Everything shipped through d561eac (spec c1-c6, trace layer,
  dump corpus 27 files, trace-diff + EXCL-001).

## Desired outcome

1. **Docs row**: `docs/known-hard-cases.md` gains a workflow row (or
   short paragraph in the workflow/roadmap section) referencing the
   decision-trace pipeline: divergence → `GENTLY_TRACE=<stage>` trace →
   diff vs committed oracle dump (`tests/repro/dumps/`) → minimal
   antichain cut → adjudication via the committed registry (a/b/c).
   Keep it 3-6 lines; match the file's existing table/style.
2. **openspec archive**: run the archive command for
   `add-decision-trace` and stage whatever it produces (spec updates
   under openspec/specs/, change moved to archive). If the CLI asks for
   confirmation non-interactively fails, use the documented flag.
3. **Gate sweep** (report numbers in your final message):
   `just gates`, `cargo test` full + `cargo clippy --all-targets`,
   `just examples-verify`, `ah check --run-tests` (79 passed, backlog 4).
4. rustfmt any regenerated specodelic artifacts (specodelic#15).

## Commit protocol

- Commit 1: docs row. `docs(decision-trace): known-hard-cases workflow row (gently-vhc)`.
- Commit 2: openspec archive. `chore(openspec): archive add-decision-trace (gently-vhc)`.
- Keep commits file-scoped; do not sweep .whisper/.wai churn.

## Out of scope

- Any Rust/source change beyond docs + openspec moves. Push. bd close.

## Report back (≤100 tokens)

- commit shas, archive result, gate numbers.
