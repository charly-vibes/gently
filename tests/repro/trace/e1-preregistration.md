# E1 pre-registration — stage selection (anti-goal guard)

Committing this file BEFORE reading any `git show`/`git diff`/`git log -p`
on the fix commits (8a0d56b, 280f9dc) or their parents. Stages below were
chosen from OPEN-divergence evidence only: bead symptom descriptions
(`bd show gently-awd`, `bd show gently-4ga`), the `gently-4ga` row in
`docs/known-hard-cases.md`, and the fixture inputs. Preregistration commit
sha: self-reference is impossible inside the commit it names — the actual
sha is recorded in `tests/repro/trace/README.md`.

## Case 1 — gently-awd (so-flowchart-size)

- **Pre-registered stage:** `layout.place` — grid band assembly + per-node
  cell placement (which grid cell and span each node occupies).
- **Rationale:** the bead symptom is a placement-shape divergence, not a
  routing or rendering one: the oracle honors `{ size: 5,10; }`-style node
  attribute blocks and lays Server/Msg Server/Client out 2x2, while gently
  lays a 1-3-2 flow (`tests/repro/claims/so-flowchart-size.observed`,
  gently-ni9.4). Node size (span) and band/cell assignment are layout
  decisions; the render error class is not implicated (nothing fails to
  render, it renders the wrong shape). Evidence: bead description +
  `.observed` claim, both open-divergence artifacts.

## Case 2 — gently-4ga (bg-comma-lists comma-chain fan)

- **Pre-registered stage:** `render.route` — edge path cell walking (the
  vertical/horizontal walk that emits edge cells between endpoints).
- **Rationale:** the symptom is a routing divergence: rendering fails with
  `edge 2 path cells (0, 1) -> (0, 3) are not orthogonally adjacent` —
  three edges leaving A land in one row with a 2-column gap the polyline
  cannot fill; the fan-in star variant shows the jump offset scales with
  graph width. Parser + model side was already done and pinned (p11); the
  defect is in path-cell walking (`layout/paths.rs` /
  `render/ascii/edges.rs` per bead metadata). Evidence: bead description +
  `docs/known-hard-cases.md` row.
- **Fixture input:** the `bg-comma-lists` comma-chain fan example texts,
  extracted from the probe body in `tests/repro/probes.pl` (lines ~277ff):
  `[ A ] -> [ B ], [ C ], [ D ]` and friends.

## Independent agreement with orchestrator preregistration

The brief records the orchestrator's pre-registered choices from the same
open evidence. My independent choices above AGREE with both (awd:
`layout.place`; 4ga: `render.route`). No disagreement to record.

## Meter (copied, not deviated)

- PASS iff, for BOTH cases, the reported minimal cut's events belong to the
  pre-registered stage for that case.
- KILL iff any cut points elsewhere, or the trace route cost more than the
  original byte-diff hunt.
