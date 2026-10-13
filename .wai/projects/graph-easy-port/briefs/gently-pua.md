# Subagent brief: gently-pua — shape:edge ranking divergence (layout only)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator holds ticket `gently-pua` claimed (in_progress) and is
hands-off: you own the implementation end-to-end. The orchestrator will
verify with real gates — never claim work the diff doesn't contain.

## The bug

`shape:edge` ranking divergence on the bloodgate tutorial_edges graph.
Oracle node dump (gently-cbk run 2, re-derive to confirm):
`Bonn(0,2), train(2,2), Ulm(4,2), #anon-buy(2,4)` — the box chain
Bonn–train–Ulm lands INLINE on one row (east flow) and the anon buy node
ranks one row BELOW train. gently currently rotates all three:
`Bonn(4,2), Ulm(2,4), #anon(0,2)`.

**This ticket is the ranking/placement fix ONLY.** The Edge::Cell render
(dash continuation, label above the cell, Scout.pm flag stripping) is
ticket `gently-cbk`'s scope — already claimed in_progress, do NOT touch it.
The cbk RED scaffold lives at
`crates/gently-core/tests/scenarios/ge_ascii_render_edge_node.rs`
(committed 9776390): the error-pin test `p2_edge_node_currently_errors_on_
path_orthogonality` and the `#[ignore]`d oracle-bytes test.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-pua` and `bd show gently-cbk` (run-2 notes)
- Read `specs/ge-layout.md` — the placement/rank contract rows.
- Read `crates/gently-core/src/layout.rs` (Layout struct: `node_cells:
  Vec<(usize,usize)>` indexed by node index; `pub fn layout`), and the
  placement internals there (ranks, channels, the jrr `extend_or_below`
  east-rank rule, the 89r port-side rules).
- Read the scaffold `crates/gently-core/tests/scenarios/
  ge_ascii_render_edge_node.rs` and its registration
  (`#[path]` mod in `ge_ascii_render.rs`) — copy that pattern.
- Pinned oracle: Graph::Easy v0.69 at `/var/tmp/ge0.69`. Re-derive the
  node dump yourself, e.g.:
  ```
  PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl -MGraph::Easy -e '
    my $g = Graph::Easy->new(); $g->from_text("[ car ] { shape: edge; }\n[ train ] { shape: edge; }\n\n[ ] -- rent --> [ car ]\n[ ] -- buy --> [ train ]\n[ Bonn ] -- [ train ]\n[ train ] --> [ Ulm ]\n[ Ulm ] -- [ car ]\n[ car ] --> [ Hof ]\n");
    $g->layout();
    for my $n (sort { $a->{name} cmp $b->{name} } $g->nodes()) {
      my $p = $n->{pos} || $n->{pos_abs} || [];
      printf "%s (%s,%s)\n", $n->{name}, $p->[0] // "?", $p->[1] // "?";
    }'
  ```
  (adjust the dump accessor to what the pinned source actually sets —
  `Layout.pm`/`Node.pm` write `pos`; read them, don't guess). If the dump
  disagrees with the bead's numbers, the FRESH DUMP wins and the bead is
  stale — note it in your report.
- `wai search "ranking"` / `wai search "flow"` for accumulated patterns.

## What to build

1. **RED**: a layout-level scenario pinning the oracle node cells for the
   bg-edge-node graph — new file
   `crates/gently-core/tests/scenarios/ge_layout_edge_node_ranking.rs`,
   registered like the cbk scaffold (`#[path]` mod inside
   `tests/scenarios/ge_layout.rs`, keeping the ge.layout row numbering —
   name the test fns after the serving row, e.g. `pN_*`, and wire a
   `.espectacular/ge-layout/pN.toml` contract ONLY if you amend that row;
   otherwise leave contracts alone). One test asserts
   `layout::layout(&g).node_cells` matches the oracle dump (map node
   NAME → cell via the model, not raw indices). A companion test pins the
   CURRENT wrong cells as the RED-state documentation (mirrors the cbk
   error-pin pattern).
2. **GREEN**: fix ranking/placement so the chain Bonn–train–Ulm lands
   inline (undirected `--` keeps the west slot on the flow row; directed
   `-->` continues east on the SAME row) and the anon buy node ranks below
   train. Trace upstream `Layout.pm`/`Scout.pm` rank assignment for
   undirected endpoints — the rule you implement must be a rule, not a
   special-case for this graph; keep ALL existing pins green.
3. **Scaffold truth check**: after the ranking fix, run the cbk scaffold
   test `p2_edge_node_currently_errors_on_path_orthogonality`. If the
   orthogonality error disappears (cells now adjacent) or its message
   changes, amend that ONE test so it pins the new truthful RED state
   (e.g. render now succeeds but bytes differ) — document the change in
   your report. NEVER un-ignore the `#[ignore]`d oracle-bytes test
   (render is cbk's scope) and never suppress the geometry error.
4. **Spec re-derivation**: amend the ge-layout row(s) your fix lands
   under (the rank/undirected-endpoint contract); regenerate + stage the
   `openspec/` mirror and `specodelic/` artifacts IN THE SAME COMMIT as
   the corpus change (stash dance if the drift gate complains).
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
6. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-pua` → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/layout.rs` (and `src/layout/**`
  if the split exists), `crates/gently-core/tests/scenarios/
  ge_layout_edge_node_ranking.rs` (new), `tests/scenarios/ge_layout.rs`
  (registration), `crates/gently-core/tests/scenarios/
  ge_ascii_render_edge_node.rs` (ONLY the truth-check amendment from
  step 3), `specs/ge-layout.md`, `openspec/` (via deploy),
  `specodelic/` regenerated artifacts, `docs/known-hard-cases.md` (move
  the resolved row), `.beads/issues.jsonl` (via bd close/export).
- Never edit: `render/**` (cbk's surface — layout.rs cell geometry that
  render consumes is yours, render code is not), unrelated `specs/*.md`,
  `.espectacular/` (unless a new contract is truly needed), `pretender.toml`,
  `.wai/resources/**`, `.wai/projects/graph-easy-port/runs/*`.
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- **Commit hygiene** (standing): before ANY `git commit`, `git status
  --short` and stage only files YOU authored.
- **Spec mirror gotcha**: deployed `openspec/` copy staged in the SAME
  commit as the corpus change.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **beads**: run `bd export` after `bd close`.
- gently-cbk is also claimed (in_progress) — you run FIRST and alone;
  do not wait on it and do not close it.

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
bd show gently-pua                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the ranking rule is not green after 2 attempts,
  or the fresh oracle dump disagrees with the bead's numbers in a way that
  changes the target, STOP, file a follow-up bead, and report.
- Time budget: 25 minutes hard — if the ranking rule is not fully
  byte-matched by then, commit the RED pin + findings and report; a
  continuation run may finish it. Do not break existing pins to force it.

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed, the fresh oracle dump vs the bead's numbers, any
amendment made to the cbk scaffold test, and any deviations.