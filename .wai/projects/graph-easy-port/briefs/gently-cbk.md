# Subagent brief: gently-cbk — shape:edge nodes render (+ bonus: bypass shape routing)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-cbk` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The bugs

1. **gently-cbk** (primary): `shape: edge` nodes — gently errors
   "edge path cells are not orthogonally adjacent"; the pinned oracle
   renders the edge-shaped node. Evidence: bloodgate tutorial_edges.html
   (`[ ] -- buy --> [ train ] { shape: edge; }`) — check
   `tests/repro/claims/` for the pinned bytes; if no claim pins THIS
   fixture, author the probe first (each example its OWN graph).
2. **gently-jrr** (bonus): shared-middle bypass shape — oracle ranks the
   middle node inline and routes the skip-edge over the top; gently
   attaches it below. Evidence:
   `tests/repro/claims/bg-minlen-bypass.observed` — re-derived by the klf
   run: renders byte-identically WITHOUT `minlen: 3` (minlen incidental);
   the divergence is the bypass routing itself.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-cbk` + `bd show gently-jrr`
- Read the ascii_render spec (c1–c8, freshly re-derived) and the layout
  spec (`ls specs/`).
- Read `crates/gently-core/src/render/ascii/` — `polyline.rs`
  (`waypoints`, `push_corner`, orthogonality checks — the error site),
  `edges.rs`, `geometry.rs`; and `crates/gently-core/src/layout/` —
  `placement.rs` (ranks, channels — the bypass routing).
- Read the pinned-oracle source (Graph::Easy v0.69 at /var/tmp/ge0.69):
  how a `shape: edge` node is placed/routed (upstream `Graph::Easy::Node`
  shape handling — the node IS an edge path; what cells it claims) and
  the bypass ranking (`Graph::Easy::Layout::Scan` — how the skip-edge
  goes over the top while the middle node stays inline).
- wai search "shape edge bypass" for accumulated patterns.

## What to build

1. **Probe**: ensure the cbk fixture bytes are pinned (author the probe if
   not); jrr's bytes are pinned. Update `serves:` comments to
   gently-cbk (+ gently-jrr).
2. **RED**: extend scenarios from pinned bytes — must fail (or error, for
   cbk) on the current tree.
3. **GREEN**:
   - cbk: route edge-shaped nodes as edge paths per the oracle — cells
     claimed orthogonally (trace the exact cell sequence; the current
     error path fires from a non-orthogonal waypoint computation — fix
     the geometry, don't just suppress the error).
   - jrr: bypass routing per the oracle's rank/scan rule (middle inline,
     skip-edge over the top).
   Extend, don't fork; keep all existing byte pins green.
4. **Spec re-derivation**: amend the ascii_render spec (+ layout spec if
   the bypass changes ranking rows); regenerate + stage `openspec/`
   mirror and `specodelic/` artifacts IN THE SAME COMMIT as the corpus
   change (stash dance if the drift gate complains).
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
6. **Bonus close**: `bd close gently-jrr` when verified.
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-cbk` (+ bonus) → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/**/*.rs`,
  `crates/gently-core/src/layout/**/*.rs` (bypass ranking, edge-shaped
  node placement), `crates/gently-core/src/model/**` (only if shape:edge
  needs attr plumbing — justify), `crates/gently-core/tests/scenarios/*.rs`,
  `tests/scenarios.rs` (registration only if needed),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  the ascii_render (+ layout if touched) specs in `specs/`,
  `openspec/` (via deploy), `specodelic/` regenerated artifacts,
  `docs/known-hard-cases.md` (move resolved rows),
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

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed (incl. bonus closes), and any deviations.
