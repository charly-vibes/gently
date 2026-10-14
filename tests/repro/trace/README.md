# E1 — retrospective localization results (gently-asc)

Decision-trace prototype run against two already-fixed divergences, per the
brief and the D7 decision rule. Pre-registration commit (anti-goal guard,
committed BEFORE any fix diff was read): **`65595b0`**
(`tests/repro/trace/e1-preregistration.md`). The orchestrator's
pre-registered stage choices and my independent choices agreed on both
cases; no disagreement to record.

**Verdict: PASS** — for both cases the reported minimal cut's events belong
to the pre-registered stage, and the trace route cost a fraction of the
original hunts.

## Case 1 — gently-awd (so-flowchart-size / size-attr)

- Fix: `8a0d56b` (span slice). Pre-registered stage: `layout.place`.
- Divergence confirmation: pre-fix renders `size-attr.txt` diverging from
  `size-attr.ascii.expected`; post-fix is byte-identical (mod the pinned
  `# oracle:` header lines, which the driver strips).
- **Minimal cut** (pre-fix trace vs post-fix trace, `layout.place` stage,
  `size-attr.txt`): the per-node placement events
  `{place A2, place B2, place A3, place B3, place A4, place B4, place A5,
  place B5}` — band-row assignment (`y`) and rank→col span accumulation
  (`x`) all differ; the 1×1 `A`/`B` chain events agree. The cut is an
  antichain over independent per-node placement decisions (D3), not a
  single "first event": removing any one placement event does not make the
  traces agree.
- **Oracle-side cut** (pre-fix trace vs aligned E2 dump
  `so-flowchart-size.dump.txt`, same SO input the dump was taken from;
  driver alignment shim maps `  Client @ (8,0) 10x10 cx=5 cy=10` →
  `place Client x=8 y=0 cx=5 cy=10`, single-celled nodes → cx=1 cy=1):
  `{place Client, place Msg Server, place Server}` — pre-fix Client sits at
  `x=4` (flat flow) where the oracle puts `x=8` (2×2), plus a `y` offset of
  one row. **All cut events are `place` events.** Post-fix, Client's `x=8`
  matches the oracle; the `y` offset remains — the known residual
  (`parallel-channel-as-rows`, per the fix commit's docs row), not part of
  this cut's meter.
- **PASS** vs the pre-registered meter: cut belongs to `layout.place`.
- Honest observation: fix^ (`cf8d30c`, the RED commit) already READS the
  size/rows/columns attrs (the trace's `cx`/`cy` are non-1 at fix^) — the
  fix's own delta is the band-row reservation and rank→col accumulation,
  still squarely inside `layout.place`.

## Case 2 — gently-4ga (bg-comma-lists comma-chain fan)

- Fix: `280f9dc` (fan-stack vertical walks jog around occupied band rows).
  Pre-registered stage: `render.route`.
- Symptom confirmation: pre-fix binary on `[ A ] -> [ B ], [ C ], [ D ]`
  exits with `render error: edge 2 path cells (0, 1) -> (0, 3) are not
  orthogonally adjacent` — the bead's exact message.
- **Minimal cut** (pre-fix trace vs post-fix trace, `render.route` stage):
  the single event `{route e2 A->D}` — pre-fix `cells=2 bends=0` (a
  2-cell path whose endpoints sit 2 grid rows apart: the jump), post-fix
  `cells=5 bends=2` (the jog around the occupied band row). The `e0`/`e1`
  route events agree. The route trace captures the defect *before* the
  render-time adjacency error fires — `route_edges` runs ahead of the
  polyline check, so the cut is observable even when rendering fails.
- **Reference**: the oracle has no routing trace; the fix commit's own
  trace is the reference, per the brief.
- **PASS** vs the pre-registered meter: cut belongs to `render.route`.

## Cost comparison

| route | wall-clock | sessions |
|---|---|---|
| trace route (this run) | ~8 s warm / ~20 s cold-ish driver (`tests/repro/trace/e1.sh`, worktrees + shared `/tmp/e1-target`) | 1 subagent run |
| gently-awd original byte-diff hunt | multi-session (see `bd show gently-awd` notes: gently-ni9.4 mining → gently-527 disentanglement → probe5/9 instrumented-oracle decode sessions → RED+feat slice) | several orchestrated sessions |
| gently-4ga original hunt | multi-session (see `bd show gently-4ga` notes: parse-side pin p11, widened gently-6hh star repro, instrumented-oracle decode of the fan claim, root-cause in `layout/paths.rs`) | several orchestrated sessions |

The trace route localized both cuts in one session at second-scale
wall-clock; both original hunts took multiple orchestrated sessions to
reach the equivalent localization. Cost criterion of the meter: met.

## Notes / honesty

- Prototype is env-gated and deletable; corpus byte-identical with
  `GENTLY_TRACE` unset (full `cargo test --workspace` + `just
  examples-verify` green at the instrumentation commit).
- The driver injects the identical emission block (extracted from
  `layout.rs` at HEAD) onto the pre-fix worktrees, since both fix commits
  predate the instrumentation; injections are idempotent.
- One pre-registration-phase correction, recorded for the record: the
  brief's "size-attr.txt.expected" is the TXT canonicalization; the ASCII
  bytes live in `size-attr.ascii.expected` (both committed with the
  slice). The driver compares the ASCII bytes.
- Commit-hook note: `fmt-check` failed once on the unformatted
  `trace.rs`; fixed with `cargo fmt` and re-committed — no `--no-verify`
  used. (Unrelated pre-existing dirt: `rustfmt` cannot resolve
  `tests/scenarios/corpus.rs` — generated at test time.)
- Throwaway worktrees removed after the run (`git worktree remove`).
