# Subagent brief: gently-emd — textwrap: auto / edge textwrap honored (+ bonus: stem column off-by-one)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-emd` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The bugs

1. **gently-emd** (primary): `graph { textwrap: auto; }` and edge
   `textwrap: 10` are ignored — the oracle wraps long labels. Evidence:
   `tests/repro/claims/bg-textwrap.observed` (bloodgate manual/syntax.html,
   gently-ni9.2). The gently-527 run just landed the variable-band
   renderer (`labels.rs` port of `_aligned_label`, byte grid in
   `canvas.rs`, `band_tops` per-row heights, node boxes grow to
   lines+2) — wrapping changes node box heights, which this renderer now
   supports. Read the 527 spec rows (c4/c5 re-derivation) first.
2. **gently-5vf** (bonus): SO 71869025 URI tree — the `/c` branch stem
   under `/` renders with a 2-space indent; the oracle uses 3 (stem
   column off by one). Evidence: `tests/repro/claims/so-graph-convert.observed`.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-emd` + `bd show gently-5vf`
- Read `specs/ge-ascii_render.md` IN FULL — especially the freshly
  re-derived c4/c5 (527: multiline aligned label rows, byte-width rule)
  and the layout spec (`ls specs/`) if textwrap touches layout.
- Read `crates/gently-core/src/render/ascii/` — `labels.rs` (527's
  `_aligned_label` port), `canvas.rs` (byte grid, `band_tops`,
  `draw_box` growth), `geometry.rs` (`byte_width`, gap rule),
  `edges.rs` (edge label rows).
- Read the pinned-oracle source (Graph::Easy v0.69 at /var/tmp/ge0.69):
  `Graph::Easy::Node::_label` / textwrap handling — how `auto` picks the
  wrap width (node size? graph width?), how `textwrap: N` wraps (word
  boundaries, hyphenation?), where the wrap happens (label generation vs
  render). For 5vf: `Graph::Easy::as_ascii` / tree stem rendering — the
  exact stem column rule.
- wai search "textwrap" for accumulated patterns.

## What to build

1. **Probe check**: bg-textwrap.observed and so-graph-convert.observed are
   pinned. Add a claim probe ONLY if a rule is ambiguous (e.g. textwrap
   interplay with explicit \n, auto on tiny graphs). Update `serves:`
   comments to gently-emd (+ gently-5vf).
2. **RED**: extend scenarios from pinned bytes — must fail on the current
   tree (side-module split if the scenario file hits the pretender
   400-line ratchet).
3. **GREEN**: implement label wrapping per the oracle rule (graph-scope
   `textwrap: auto`, edge `textwrap: N`) — wire the attribute through
   model/attr storage into the label row generation; and the 5vf stem
   column rule. Extend, don't fork; keep 527's alignment + byte-width
   behavior byte-identical on single-line pins.
4. **Spec re-derivation**: amend the ascii_render spec (+ layout spec if
   wrap feeds layout); regenerate + stage `openspec/` mirror and
   `specodelic/` artifacts IN THE SAME COMMIT as the corpus change
   (stash dance if the drift gate complains).
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
6. **Bonus close**: `bd close gently-5vf` when verified.
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-emd` (+ bonus) → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/**/*.rs`,
  `crates/gently-core/src/layout/**` (only if wrap feeds layout —
  justify), `crates/gently-core/src/parse/**` / model attr storage (only
  to plumb textwrap through), `crates/gently-core/tests/scenarios/*.rs`,
  `tests/scenarios.rs` (registration only if needed),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  the ascii_render (+ layout if touched) spec in `specs/`,
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
bd show gently-emd                        # shows closed
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
