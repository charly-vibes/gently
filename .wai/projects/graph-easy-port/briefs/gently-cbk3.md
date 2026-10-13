# Subagent brief: gently-cbk run 3 — shape:edge Edge::Cell render (GREEN continuation)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator holds ticket `gently-cbk` claimed (in_progress) and is
hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## Where the previous runs left this

- Run 1 re-derived the oracle rule; run 2 committed the RED scaffold
  (9776390); gently-pua (just shipped, 37cad0b) fixed the RANKING — the
  box chain Bonn–train–Ulm now lands inline and the anon leaf ranks below.
- Current truthful RED state (pinned by
  `ge_ascii_render::p2_edge_node::p2_edge_node_renders_after_ranking_fix_
  but_bytes_diverge`): the graph RENDERS with no geometry error and the
  placement ranks match the oracle dump, but the BYTES still diverge —
  the edge-shaped nodes are still rendered as plain boxes, not edge cells.
- The oracle-bytes target:
  `ge_ascii_render::p2_edge_node::p2_edge_node_renders_as_edge_cell_with_
  label_above` is `#[ignore]`d. **Un-ignore it in the GREEN commit** and
  make it pass byte-identical against
  `tests/repro/claims/bg-edge-node.observed`.

## The GREEN target (from the re-derivation — do not redo it)

A `shape: edge` node becomes an `Edge::Cell` (EDGE_HOR + EDGE_LABEL_CELL):
edge endpoints treat it as an edge-cell, Scout.pm strips start/end flags,
the edge dashes continue THROUGH the node (Bonn --> train --> Ulm draws
ONE line with a single `>` at Ulm), the node's label rides above the cell
line (`train`, `rent`, `car`), and the anon cross node #3 ranks below
train with `buy` entering train's south side (`^`). This is a
full-pipeline feature — placement through the node's cells + renderer
shape class — not a waypoint patch.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-cbk` (run 1+2 notes) and `bd show gently-pua` (close
  reason: what landed)
- Read the full context brief `.wai/projects/graph-easy-port/briefs/
  gently-cbk2.md` — everything not superseded above still applies
  (orientation list, oracle source locations, hard scope guard,
  completion criteria, reporting format).
- Read `specs/ge-ascii_render.md` (c1–c8) and the scaffold
  `crates/gently-core/tests/scenarios/ge_ascii_render_edge_node.rs`.
- Read the pinned-oracle source (Graph::Easy v0.69 at /var/tmp/ge0.69):
  `Graph::Easy::Node` shape:edge handling, `Scout.pm` flag stripping, how
  the node's cells merge into the edge path grid, label placement.
- `wai search "shape edge"` and `wai search "edge cell"` for accumulated
  patterns. Note the oy2 lesson: methods live in reopened package
  `Graph::Easy::Edge::Cell` inside As_ascii.pm; patch THAT stash to trace.

## What to build

1. **RED** is already committed (9776390 + pua's amendment) — do not
   re-author; verify it still pins the truth before starting.
2. **GREEN**: implement shape:edge rendering per the re-derivation: node
   cells claimed as EDGE_HOR + EDGE_LABEL_CELL edge cells, dash
   continuation through the cell, label above, endpoint flag stripping.
   Fix geometry — never suppress the orthogonality error. Un-ignore the
   oracle-bytes test in the GREEN commit. Keep ALL existing byte pins
   green.
3. **Spec re-derivation**: amend the relevant ge-ascii_render row(s);
   regenerate + stage `openspec/` mirror and `specodelic/` artifacts IN
   THE SAME COMMIT as the corpus change (stash dance if the drift gate
   complains).
4. **Commit atomically** (feat → spec/docs; stage only your files;
   pretender ratchets — abc>30 / 40 lines → split helpers).
5. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-cbk` → `bd export` → `git push`.

## Hard scope guard (identical to gently-cbk2.md — restated key points)

- Allowed: `crates/gently-core/src/layout/**`, `render/**`,
  `model/**` (shape plumbing — justify),
  `crates/gently-core/tests/scenarios/*.rs`, `tests/scenarios.rs`
  (registration only if needed), `tests/repro/probes.pl`,
  `tests/repro/claims/*`, the ascii_render specs in `specs/`,
  `openspec/` (via deploy), `specodelic/` regenerated artifacts,
  `docs/known-hard-cases.md`, `.beads/issues.jsonl` (via bd close/export).
- Never edit: unrelated `specs/*.md`, `.espectacular/` (unless a new flag
  is truly needed), `pretender.toml`, `.wai/resources/**`,
  `.wai/projects/graph-easy-port/runs/*`.
- Do NOT run `wai close` — the orchestrator owns session close.

## Completion criteria (all must exit 0)

```
cargo test -p gently-core
cargo clippy --workspace --all-targets -- -D warnings
ah check --run-tests
just gates
just examples-verify
just oracle-verify
spk lint openspec
git status --short                        # clean (only .whisper/.wai session files)
git log origin/main..HEAD                 # empty (pushed)
bd show gently-cbk                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the oracle rule is not green after 2 attempts,
  or pinned bytes disagree in a way that implies a NEW probe you cannot
  author, STOP, file a follow-up bead, and report. The OBSERVED file wins.
- Time budget: 25 minutes hard — if not byte-identical by then, commit
  the findings and report; a fourth run may finish it. Do not break
  existing pins to force it.

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed, the exact remaining byte delta if any, and any
deviations.