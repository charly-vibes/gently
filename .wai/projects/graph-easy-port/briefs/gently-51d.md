# Subagent brief: gently-51d — wide/broad edge styles render as bold double-line `##>`

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-51d` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The bug

Input `[ AB ] --> { style: wide; } [ ABC ]` (and `style: broad`): the
oracle as_ascii renders the edge as `##>` (double-line, identical to
`style: bold`); gently renders plain `-->`. Evidence already pinned:
`tests/repro/claims/pod-style-family.observed` — seed 0 AND seed 1 both
show bold/wide/broad as `##>` (this fixture is hash-stable across seeds).

Root cause is almost certainly `edge_pattern` in
`crates/gently-core/src/render/ascii/styles.rs`: `"bold" => "##"` exists
but `"wide"` and `"broad"` fall through to the `_ => "-"` default.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-51d`
- Read `specs/ge-ascii_render.md` row c2 + property p2 — the edge-run
  contract (re-derived gently-oy2, extended gently-7qz). Your change is a
  small amendment to that contract.
- Read `crates/gently-core/src/render/ascii/styles.rs` — `edge_pattern`,
  `edge_run_at`, `edge_gap_width` (gently-7qz split canvas helpers into
  `edges.rs`; the glyph tables remain in styles.rs).
- Confirm from the pinned oracle source (Graph::Easy v0.69 @
  /var/tmp/ge0.69) that wide/broad alias the bold `##` glyph — check
  `Graph::Easy::Edge::Cell` style handling / `Graph::Easy` EDGE_STYLE_*
  constants; also check `edge_gap_width` (does a wide/broad gap widen like
  bold?) and what an UNDIRECTED wide/broad run looks like (probe it if the
  committed evidence doesn't cover the arrowless case — see step 1).

## What to build

1. **Probe check**: if the arrowless wide/broad run (`###`?) is not pinned,
   add a small claim probe to `tests/repro/probes.pl` ([ A ] -- { style:
   wide; } [ B ]) and capture the `.observed` (update its `serves:` to
   gently-51d). If pod-style-family.observed already answers every
   question, skip new probes — do not re-record what is pinned.
2. **RED**: extend the existing p2 scenario tests in
   `crates/gently-core/tests/scenarios/ge_ascii_render.rs` with wide/broad
   expectations from the pinned bytes — must fail on the current tree.
   (New tests are named `p2_*` under the existing p2 contract flag —
   gently-7qz convention; no new `.espectacular` flags needed.)
3. **GREEN**: add `"wide" | "broad"` to the bold arm of `edge_pattern`
   (or whatever the oracle source confirms). Check `edge_gap_width` and
   `_correct_size` widening rules for wide/broad against the oracle.
4. **Spec re-derivation**: amend specs/ge-ascii_render.md c2's style list
   (wide/broad = bold `##`) if the contract text enumerates styles —
   regenerate + stage `openspec/` mirror and the `specodelic/` artifacts
   (`*.toml`, `*_props.rs`, `*.check.json`) IN THE SAME COMMIT as the
   corpus change (spec-drift-gate compares worktree openspec/ against the
   INDEX; mid-flight deploys make every intermediate commit red).
5. **Commit atomically** (order: test pins → feat → spec/docs; each
   commit: `git status --short`, stage ONLY your files).
6. **Gates, then close, then push**: all completion criteria exit 0 →
   `bd close gently-51d` → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/ascii/{styles,edges}.rs`,
  `crates/gently-core/tests/scenarios/ge_ascii_render.rs`,
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-ascii_render.md`, `openspec/` (via deploy),
  `specodelic/` regenerated artifacts,
  `docs/known-hard-cases.md` (move the resolved row out),
  `.beads/issues.jsonl` (via bd close/export).
- Never edit: other `specs/*.md`, `.espectacular/` (unless a new flag is
  truly needed), `pretender.toml`, `.wai/resources/**`,
  `.wai/projects/graph-easy-port/runs/*`.
- Respect pretender ratchets (abc > 30, function_lines > 40 — split
  helpers, never raise limits).
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- **Commit hygiene** (standing): before ANY `git commit`, `git status
  --short` and stage only files YOU authored.
- **Spec mirror gotcha**: deployed `openspec/` copy staged in the SAME
  commit as the corpus change.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **beads**: run `bd export` after `bd close` or the close is invisible
  to git.

## Completion criteria (all must exit 0)

```
cargo test -p gently-core
cargo clippy --workspace --all-targets -- -D warnings
ah check --run-tests
just gates
just examples-verify
just oracle-verify
spk lint openspec
git status --short                        # clean (only .whisper notes allowed)
git log origin/main..HEAD                 # empty (pushed)
bd show gently-51d                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the oracle rule is not green after 2 attempts,
  or pinned bytes disagree with the source in a way that implies a NEW
  probe you cannot author, STOP, file a follow-up bead, and report.
  The OBSERVED file wins over any reasoning.

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed, and any deviations.
