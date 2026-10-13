# Subagent brief: gently-89r — start:/end: port attributes honored in layout (+ bonus: indexed ports)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-89r` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The cluster (port attributes, two pinned divergences)

1. **gently-89r** (primary): `start:`/`end:` port attributes are ignored —
   oracle honors `[ Koblenz ] --> { start: right; } [ Trier ]` (lays Trier
   inline on Koblenz's right). Evidence:
   `tests/repro/claims/bg-start-right.observed` (bloodgate
   tutorial_bypass.html, gently-ni9.2). WARNING from the gently-jrr run:
   the tutorial_bypass shape ALSO has a shared-middle bypass divergence
   (jrr owns that) — pin your probe bytes to isolate the PORT effect,
   avoid entangling the bypass shape; if the fixture can't be
   disentangled, document the interaction and keep jrr's shape out.
2. **gently-byq** (bonus): indexed ports `{ start: front,N }` route edges
   to the Nth port on the node's front side (SO 47323345 answer, 12-line
   interleave). Evidence: `tests/repro/claims/so-start-port-index.observed`.
   (Also see `dot-records-ports.observed` for the dot-side record/ports
   pin.)

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-89r` + `bd show gently-byq` (+ `bd show gently-jrr` for
  the bypass entanglement warning)
- Read the layout spec (`ls specs/` — ge-layout.md, re-derived 4nx/89d).
- Read `crates/gently-core/src/layout/{flow,placement}.rs` and the
  attachment/exit side machinery in `render/ascii/` (polyline.rs
  `exit_dir`/`entry_dir`, edges.rs) — ports change WHICH side/index an
  edge attaches at.
- Read the pinned-oracle source (Graph::Easy v0.69 at /var/tmp/ge0.69):
  `Graph::Easy::Node` port handling — `{start: right}` / `{start: front,N}`
  → which side/index the edge attaches, how index N counts, how the
  layout ranks the inline node.
- wai search "ports layout" for accumulated patterns.

## What to build

1. **Probe check**: bg-start-right.observed and so-start-port-index.observed
   are pinned. Add a claim probe ONLY to disentangle port effects from
   the jrr bypass shape, or for an ambiguous rule (e.g. end: ports, back
   sides, index out of range). Update `serves:` comments to gently-89r
   (+ gently-byq).
2. **RED**: extend the layout/render scenarios from pinned bytes — must
   fail on the current tree (side-module split if the scenario file hits
   the pretender 400-line ratchet).
3. **GREEN**: implement port attributes through layout + attachment:
   parse `start:`/`end:` (sides and `side,N` index forms), store on the
   edge, honor at exit/entry attachment and in layering placement per the
   oracle rule. Extend, don't fork.
4. **Spec re-derivation**: amend the layout spec (+ ascii_render
   attachment rows if the geometry changes); regenerate + stage
   `openspec/` mirror and `specodelic/` artifacts IN THE SAME COMMIT as
   the corpus change (stash dance if the drift gate complains).
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
6. **Bonus close**: `bd close gently-byq` when covered and verified.
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-89r` (+ bonus) → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/layout/**/*.rs`,
  `crates/gently-core/src/render/ascii/**` (attachment side),
  `crates/gently-core/src/parse/**` (port attr storage only),
  `crates/gently-core/tests/scenarios/*.rs`, `tests/scenarios.rs`
  (registration only if needed), `tests/repro/probes.pl`,
  `tests/repro/claims/*`, the layout spec in `specs/` (+ ascii_render if
  attachment geometry rows change), `openspec/` (via deploy),
  `specodelic/` regenerated artifacts, `docs/known-hard-cases.md`
  (move resolved rows), `.beads/issues.jsonl` (via bd close/export).
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
bd show gently-89r                        # shows closed
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
