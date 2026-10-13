# Subagent brief: gently-cbk — shape:edge nodes as Edge::Cell (full-pipeline feature)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator holds ticket `gently-cbk` claimed (in_progress) and is
hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## The bug

`shape: edge` nodes: gently errors "edge path cells are not orthogonally
adjacent"; the pinned oracle renders the edge-shaped node. Evidence:
`tests/repro/claims/bg-edge-node.observed` (bloodgate tutorial_edges.html:
`[ ] -- buy --> [ train ] { shape: edge; }`).

## Prior re-derivation (gently-cbk run 1, b206bdb — do not redo)

A `shape: edge` node becomes an `Edge::Cell` (EDGE_HOR + EDGE_LABEL_CELL)
in the oracle: edge endpoints treat it as an edge-cell, Scout.pm strips
start/end flags, the edge dashes continue THROUGH the node, the label
rides above the cell line. This is a full-pipeline feature — placement
through the node's cells + renderer shape class — not a waypoint patch.
A prior scenario reproduced the error but was NOT committed; re-author it.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-cbk`
- Read `specs/ge-ascii_render.md` (c1–c8) and `specs/ge-layout.md` —
  the placement/shape contract rows.
- Read `crates/gently-core/src/layout/placement.rs` (ranks, channels,
  the jrr `extend_or_below` east-rank rule just landed),
  `render/ascii/{polyline,edges,geometry}.rs` (orthogonality checks,
  cell claiming, the error site).
- Read the pinned-oracle source (Graph::Easy v0.69 at /var/tmp/ge0.69):
  `Graph::Easy::Node` shape:edge handling, `Scout.pm` flag stripping,
  how the node's cells merge into the edge path grid, label placement.
- wai search "shape edge" for accumulated patterns.

## What to build

1. **RED**: re-author the scenario from `bg-edge-node.observed` bytes —
   must reproduce the current error (then fail with wrong bytes).
2. **GREEN**: implement shape:edge placement + rendering per the
   re-derivation: node cells claimed as edge cells, dash continuation,
   label above, endpoint flag stripping. Fix the orthogonality geometry —
   never suppress the error. Keep ALL existing byte pins green.
3. **Spec re-derivation**: amend the relevant spec rows; regenerate +
   stage `openspec/` mirror and `specodelic/` artifacts IN THE SAME
   COMMIT as the corpus change (stash dance if the drift gate complains).
4. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
5. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-cbk` → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/layout/**/*.rs`,
  `crates/gently-core/src/render/**/*.rs`,
  `crates/gently-core/src/model/**` (shape plumbing — justify),
  `crates/gently-core/tests/scenarios/*.rs`, `tests/scenarios.rs`
  (registration only if needed), `tests/repro/probes.pl`,
  `tests/repro/claims/*`, the ascii_render (+ layout) specs in `specs/`,
  `openspec/` (via deploy), `specodelic/` regenerated artifacts,
  `docs/known-hard-cases.md` (move the resolved row),
  `.beads/issues.jsonl` (via bd close/export).
- Never edit: unrelated `specs/*.md`, `.espectacular/` (unless a new flag
  is truly needed), `pretender.toml`, `.wai/resources/**`,
  `.wai/projects/graph-easy-port/runs/*`.
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- **Commit hygiene** (standing): before ANY `git commit`, `git status
  --short` and stage only files YOU authored.
- **Spec mirror gotcha**: deployed `openspec/` copy staged in the SAME
  commit as the corpus change.
- **Probe claims must make each example its OWN graph** (pod_probe_body
  already does this).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **beads**: run `bd export` after `bd close`.

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
- This is the hardest remaining ticket — if after the budget it is not
  byte-identical, commit the scaffold + findings and report; a third run
  may finish it. Do not break existing pins to force it.

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed, and any deviations.
