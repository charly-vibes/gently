# Subagent brief: gently-b0z — anonymous nodes render boxless in as_ascii (+ bonus: group dashed border)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-b0z` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The bugs

1. **gently-b0z** (primary): for `[ ] -> [ Bonn ] -> [ ]` the oracle
   as_ascii draws ONLY the Bonn box with arrows to/from empty space;
   gently renders visible boxed `#1`/`#3` anonymous nodes. Evidence
   pinned: `tests/repro/claims/pod-anon-nodes.observed` (also see
   `anon-numbering.observed`, `anon-reference.observed` for the #N
   naming/reuse rules already implemented parser-side — gently-6j0).
   Fixture rejected at `just oracle-verify`.
2. **gently-7rx** (bonus): node groups render without the dashed border
   box — oracle draws `+ - - - - -+` around members with the group title
   (` Cities: `). Evidence pinned: `tests/repro/claims/pod-group.observed`
   (Parser.pm POD groups example).

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-b0z` + `bd show gently-7rx`
- Read `specs/ge-ascii_render.md` IN FULL (c1–c6 + model + properties —
  recently re-derived by dcp/oy2/7qz/51d; your change adds/amends rows
  for anon-node rendering and group borders).
- Read `crates/gently-core/src/render/ascii/` — `canvas.rs` (box
  stamping, `draw_box`, column geometry), `edges.rs` + `polyline.rs`
  (edge attachment points), `styles.rs` (border glyph tables — dashed
  border glyphs likely needed here).
- Read the pinned-oracle source for the true rules (Graph::Easy v0.69 at
  /var/tmp/ge0.69): `Graph::Easy::Node::as_ascii` — how an anonymous
  node (empty name / `{name => ...}` `#N`) renders (no box, no label,
  edges attach to the empty cell); `Graph::Easy::Group` as_ascii
  handling — the dashed group box placement around member nodes + title.
- wai search "ascii_render" for accumulated patterns (CJK byte-width
  divergence is gently-dcp's, do NOT touch).

## What to build

1. **Probe check**: pod-anon-nodes.observed and pod-group.observed are
   pinned. Add a claim probe ONLY if a rule is ambiguous (e.g. anon node
   with edges on multiple sides, group with edge crossing the border,
   group title alignment). Update `serves:` comments to gently-b0z
   (+ gently-7rx).
2. **RED**: extend scenarios in `crates/gently-core/tests/scenarios/ge_ascii_render.rs`
   (or the appropriate scenario file) from pinned bytes — must fail on
   the current tree. Respect the pretender structure (new scenario files
   split into child modules matching contract filter paths — m4i run
   pattern).
3. **GREEN**:
   - Anonymous nodes: no box, no label — edges terminate at the empty
     cell the oracle leaves (trace the exact geometry: what x/y the
     arrows point at, whether any space is reserved).
   - Groups: dashed border box (`+ - - - - -+` family) around members
     with the title row (trace glyph set + title placement + padding
     from the oracle source; extend `styles.rs` border tables).
4. **Spec re-derivation**: amend `specs/ge-ascii_render.md`; regenerate +
   stage `openspec/` mirror and `specodelic/` artifacts IN THE SAME
   COMMIT as the corpus change (51d/a0k stash dance if the drift gate
   complains).
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
6. **Bonus close**: `bd close gently-7rx` when covered and verified
   against its evidence. NOTE: gently-58m's residual and gently-4ga are
   OUT of scope (different surface).
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-b0z` (+ bonus) → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/**/*.rs`,
  `crates/gently-core/tests/scenarios/*.rs` (render-related),
  `tests/scenarios.rs` (registration only if needed),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-ascii_render.md`, `openspec/` (via deploy), `specodelic/`
  regenerated artifacts, `docs/known-hard-cases.md` (move resolved rows),
  `.beads/issues.jsonl` (via bd close/export).
- Never edit: other `specs/*.md` (unless a group/model row genuinely
  belongs to another spec — then file a follow-up instead),
  `.espectacular/` (unless a new flag is truly needed), `pretender.toml`,
  `.wai/resources/**`, `.wai/projects/graph-easy-port/runs/*`.
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
bd show gently-b0z                        # shows closed
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
