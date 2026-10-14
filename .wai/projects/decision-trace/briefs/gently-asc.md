# Subagent brief: gently-asc — decision-trace E1 retrospective localization

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core`, edition 2021; experiment drivers may be
bash/perl). An orchestrator has claimed ticket `gently-asc` and is hands-off:
you own the implementation end-to-end (prototype, runs, measurements, commit).
Do NOT close the ticket and do NOT push — the orchestrator owns ship.

## Orientation (max 10 tool calls)

- `export WAI_PROJECT=decision-trace`
- `bd show gently-asc`
- Read `.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md` — sections D2, D3, D7 (E1 row + decision rule).
- `bd show gently-awd` and `bd show gently-4ga` — SYMPTOM descriptions only at this point.
- Read the `gently-4ga` row in `docs/known-hard-cases.md`.
- E2 landed this session: `tests/repro/dumps/` holds oracle `_prepare_layout`
  dumps per fixture (see `tests/repro/dumps/README.md`). `so-flowchart-size.dump.txt`
  is the oracle-side placement trace you will diff against.
- Find the gently CLI: `cargo run -p gently-cli -- --help` (or the built
  `target/debug/gently`); render a fixture input with `--as` ascii (check the
  flag once).

## ANTI-GOAL — pre-registration MUST come first (reward-hacking guard)

The instrumented stage for each case MUST be chosen from OPEN-divergence
evidence only (bead symptom descriptions + known-hard-cases rows) and written
to `tests/repro/trace/e1-preregistration.md` BEFORE you run any `git show`,
`git diff`, or `git log -p` on the fix commits (8a0d56b, 280f9dc) or their
parents. The preregistration file must contain, per case: the chosen stage, a
one-paragraph rationale citing the symptom evidence you used, and the commit
sha of the preregistration commit. **Commit the preregistration as its own
commit first** — that is the timestamped record. Only then may you read the
fix diffs to build the METER comparison. Post-hoc stage selection invalidates
the run: if you catch yourself revising a stage after reading a diff, STOP and
record the violation honestly in the README.

**Pre-registered stage choices** (recorded by the orchestrator from the same
open evidence; your file should independently agree — record any disagreement
as a finding):

1. **gently-awd case** — stage `layout.place`: grid band assembly + per-node
   cell placement (which grid cell and span each node occupies). Symptom
   evidence: the bead describes the oracle laying Server/Msg Server/Client
   2x2 while gently lays a 1-3-2 flow — a placement-shape divergence.
   Fixture input: `tests/fixtures/graph-easy/size-attr.txt` (committed with
   the size-attr slice; its `.ascii.expected` is the pinned oracle bytes).
2. **gently-4ga case** — stage `render.route`: edge path cell walking (the
   vertical/horizontal walk that emits edge cells between endpoints).
   Symptom evidence: the bead row describes path cells jumping occupied band
   rows ("path cells not orthogonally adjacent") — a routing divergence.
   Fixture input: the `bg-comma-lists` comma-chain fan example texts (extract
   them from the `bg-comma-lists` claim probe body in `tests/repro/probes.pl`).

## What to build

A minimal env-gated decision-trace prototype + a trace-diff driver:

1. **Trace emission in gently-core** (`crates/gently-core/src/layout.rs` /
   `layout/placement.rs` / `render/ascii/edges.rs` — wherever the two stages'
   decisions are materialized): when `GENTLY_TRACE` names a stage
   (`layout.place` or `render.route`), append one compact line per decision to
   the file named by `GENTLY_TRACE_FILE` (or stderr if unset). Format: one
   line per node placement `place <name> x=<x> y=<y> cx=<cx> cy=<cy>` and one
   line per routed edge `route <edge-id> cells=<n> bends=<bend-count>` (exact
   fields may adapt to what the stage actually decides — keep it COMPACT and
   deterministic). NO default-path behavior change: the corpus must render
   byte-identically with the env var unset; `cargo test` and
   `just examples-verify` must stay green. Keep the instrumentation small —
   this is a prototype, not the trace layer.
2. **Driver** `tests/repro/trace/e1.sh` (bash; perl only if needed): for each
   case, create throwaway worktrees at `fix^` and `fix`
   (`git worktree add /tmp/e1-<id>-<rev> <rev>`), build gently in each
   (`cargo build -p gently-cli` — reuse a shared target dir via
   `CARGO_TARGET_DIR=/tmp/e1-target` to keep it fast), run the fixture with
   `GENTLY_TRACE=<stage>`, and diff traces.
   - **awd case**: diff gently@8a0d56b^ trace vs the oracle-side evidence —
     the `so-flowchart-size.dump.txt` E2 dump (align its band/node lines to
     the trace's placement lines; the alignment shim is part of the driver)
     AND vs `size-attr.txt.expected` bytes for the divergence confirmation.
   - **4ga case**: diff gently@280f9dc^ trace vs gently@280f9dc trace (the
     oracle has no routing trace; the fix commit is the reference here).
3. **Minimal cut**: report the FIRST differing event group between the two
   traces (D3: minimal cuts, not "the first event") — if the diff is noisy,
   the cut is the smallest set of trace events whose removal makes the traces
   agree on the divergent outcome. State the cut in the README.
4. **Results file** `tests/repro/trace/README.md`: per case — the cut, which
   stage the cut's events belong to, PASS/KILL vs the pre-registered meter,
   and the COST comparison: wall-clock + command count for the trace route,
   vs the original hunt's cost (from the bead notes: gently-awd's byte-diff
   hunt spanned multiple orchestrated sessions; gently-4ga's needed
   instrumented-oracle decode sessions — cite the beads).

## Pre-registered meter (do not deviate)

- **PASS** iff, for BOTH cases, the reported minimal cut's events belong to
  the pre-registered stage for that case.
- **KILL** iff any cut points elsewhere, or the trace route to reach the
  localization cost MORE (wall-clock or session-equivalents) than the
  original byte-diff hunt.
- A clean kill is a VALID outcome (it feeds the D7 matrix; do not massage
  results). Record honest numbers, including disagreements between your
  independent stage choice and the orchestrator's preregistration.

## Out of scope

- No openspec/spec changes, no specodelic artifacts, no new claim probes, no
  changes to `.observed` files or fixture companions.
- No trace LAYER design implementation (D1–D6 stay provisional) — prototype
  only, env-gated, deletable.
- Leave pre-existing worktree dirt alone (`.whisper/`,
  `tests/repro/claims/so-flowchart-size.observed`); commit ONLY your files.
- Clean up throwaway worktrees (`git worktree remove`) before finishing.

## Commit sequence (each its own commit)

1. `test(repro): E1 stage pre-registration (anti-goal guard) (gently-asc)` — the preregistration file. NO fix diffs may have been read before this one.
2. `feat(repro): env-gated decision-trace prototype + E1 driver (gently-asc)` — instrumentation + e1.sh (instrumentation commits may follow RED/GREEN: add emission, verify corpus still byte-identical).
3. `chore(repro): E1 results — <verdict per case> (gently-asc)` — README.md with cuts, meters, cost comparison.

Pre-commit hooks run; if the spec-drift-gate complains it is misfiring on
unrelated dirt — use `--no-verify` ONLY after verifying your own files pass
(documented pattern). Budget: 35 minutes hard cap — commit incrementally so
partial work survives.

## Follow-up threshold

If the instrumentation needs to touch render semantics (not just emit), or
the diff alignment shim balloons beyond ~100 lines of driver: stop, commit
what works, leave a precise hand-off — do not expand scope.
