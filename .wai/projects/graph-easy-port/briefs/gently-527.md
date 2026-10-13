# Subagent brief: gently-527 — newline edge-label split (+ bonus: alignment escapes, byte-width labels)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-527` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The cluster (label rendering, three pinned/derivable divergences)

1. **gently-527** (primary): literal `\n` in an edge label printed raw —
   oracle splits `'put msg\n via http'` into two centered rows above the
   arrow. Evidence: `tests/repro/claims/so-flowchart-size.observed`
   (SO 47323345, gently-ni9.4). NOTE: that fixture also pins `{ size: 5,10; }`
   node attrs and a 2x2 layout — pin the LABEL split effect on its own if
   entangled; the `{ size:` handling is NOT in scope (file a follow-up if
   it diverges independently).
2. **gently-y7i** (bonus): `\l`/`\r`/`\c` alignment escapes rendered raw —
   oracle interprets them as left/right/center alignment in multiline
   labels. Evidence: `tests/repro/claims/bg-multiline-align.observed`.
3. **gently-bvb** (bonus): label width counted in Unicode chars — oracle
   counts UTF-8 BYTES (`[ Köln ]` box is 7 dashes wide in the oracle).
   Evidence: partially in `tests/repro/claims/ascii-render-tables.observed`
   (wide-glyph pins); probe `[ Bonn ] --> [ Köln ]` specifically if not
   pinned. This is the gently-dcp-flagged divergence (the c5 letter was
   kept display-width; dcp owns the re-derivation — if your byte-width fix
   naturally closes dcp, bonus-close it too).

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-527` + `bd show gently-y7i` + `bd show gently-bvb`
- Read `specs/ge-ascii_render.md` IN FULL (recently re-derived: oy2/7qz/
  51d/b0z/klf — c2 runs, c5 letter, c7 anon, c8 groups). Your change
  amends/extends rows (label rows, alignment, byte-width).
- Read `crates/gently-core/src/render/ascii/` — `canvas.rs` (draw_box,
  centered labels, `display_width`), `geometry.rs`/`groups.rs` (b0z split),
  `edges.rs` (edge label hosting — 7qz's label placement).
- Read the pinned-oracle source (Graph::Easy v0.69 at /var/tmp/ge0.69):
  label splitting (`\n` → row array), alignment escapes (`\l`/`\r`/`\c` →
  alignment fields, `_printfb` centering), and the byte-vs-char width rule
  (`length()` on bytes in Perl).
- wai search "label" for accumulated patterns. NOTE: label-wrap
  (auto/textwrap) is gently-emd's — OUT of scope; do not implement
  wrapping, only the literal `\n` split and alignment escapes.

## What to build

1. **Probe check**: so-flowchart-size.observed and bg-multiline-align.observed
   are pinned; add a minimal claim for the Köln byte-width case if not
   pinned. Update `serves:` comments to gently-527 (+ y7i/bvb/dcp).
2. **RED**: extend scenarios from pinned bytes — must fail on the current
   tree (side-module split if the scenario file hits the pretender
   400-line ratchet — klf/89r pattern).
3. **GREEN**: implement in `crates/gently-core/src/render/ascii/`:
   edge-label `\n` split into multiple centered rows, alignment escape
   interpretation, byte-width box arithmetic. Extend, don't fork; keep
   the 8ol quote rules and 7qz label placement intact.
4. **Spec re-derivation**: amend `specs/ge-ascii_render.md`; regenerate +
   stage `openspec/` mirror and `specodelic/` artifacts IN THE SAME
   COMMIT as the corpus change (stash dance if the drift gate complains).
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
6. **Bonus close**: `bd close gently-y7i` / `bd close gently-bvb` (+
   `bd close gently-dcp` if byte-width covers it) when verified.
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-527` (+ bonus) → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/**/*.rs` (label surface),
  `crates/gently-core/src/layout/**` ONLY if label rows change geometry
  (justify in commit message), `crates/gently-core/tests/scenarios/*.rs`,
  `tests/scenarios.rs` (registration only if needed),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-ascii_render.md`, `openspec/` (via deploy), `specodelic/`
  regenerated artifacts, `docs/known-hard-cases.md` (move resolved rows),
  `.beads/issues.jsonl` (via bd close/export).
- Never edit: unrelated `specs/*.md`, `.espectacular/` (unless a new flag
  is truly needed), `pretender.toml`, `.wai/resources/**`,
  `.wai/projects/graph-easy-port/runs/*`. textwrap/auto wrapping (emd)
  and `{ size:` attrs are out of scope.
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
bd show gently-527                        # shows closed
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
