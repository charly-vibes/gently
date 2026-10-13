# Subagent brief: gently-7qz — start-point arrowheads for bidirectional edges + inline edge-label placement

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-7qz` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## Why

Delegating because the work needs a focused oracle-tracing session
(Perl source reading + probe authoring + TDD) and the orchestrator session
is at elevated context. Prior sessions solved the sibling divergence
gently-oy2 the same way — the loop below is proven; follow it literally.

## The bug

Input `[ A ] <-- label --> [ B ]`: the oracle as_ascii draws arrowheads at
BOTH endpoints (`v` at the source, matching the `<--` start direction) with
the edge label placed per the oracle's label rule; gently draws NO start
arrowhead and always hosts the label inline in the widened gap.
Oracle evidence is already captured: `tests/repro/claims/pod-bidir-label.observed`
(two seeds — note the oracle legitimately produces DIFFERENT layouts per
seed; both are pinned oracle bytes).

Binding constraint: specs/ge-oracle.md c7 byte-compat scope. This fixture
class is hash-stable (the probe pins seeds), so byte-compat applies for
each pinned seed. If you find a part of the output is NOT hash-stable, do
not force it — record the divergence and stop that sub-fix (follow-up
threshold below).

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-7qz`
- Read `specs/ge-ascii_render.md` IN FULL — row c2 + property p2 are the
  edge-run contract (re-derived by gently-oy2); your change amends/extends
  this contract for start arrowheads + label placement.
- Read the probe harness: `tests/repro/probes.pl` — claims `pod-bidir-label`
  and `pod-bidir` (Easy.pm `<->` example, same family) already exist and
  serve this ticket's family; `.observed` files under `tests/repro/claims/`
  are the pinned oracle bytes.
- Run the probe to see live oracle behavior (fixture exists):
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl claim pod-bidir-label`
  (`probes.pl claim` with no name lists all claims; `probes.pl` bare prints
  the remediation preamble).
- Read the oracle internals for the rule (pinned at /var/tmp/ge0.69):
  - `Graph::Easy::Node` `_draw_hor` / `_correct_size` — arrowhead glyph
    placement at both ends of a horizontal run (gently-oy2 already
    mirrored the END-side rule; the START-side rule is the missing half).
  - How the oracle renders a start arrowhead for `<--` (`v` at source for
    horizontal edges — confirm the exact glyph + position rule from
    source, don't guess).
  - Label placement: when the oracle hosts an edge label inline in the gap
    (`+---+  label    +---+`) vs above the edge on its own row (seed 0 of
    pod-bidir-label). Trace the actual condition in the oracle source.
- **Decode method (proven pattern from gently-oy2):** the implementation
  methods live in the reopened package `Graph::Easy::Edge::Cell` inside
  `As_ascii.pm` — patch THAT stash copy to trace (the `As_ascii::` stash is
  wrong). `_printfb_line` is in package `Graph::Easy::Node`. Probe claims
  must make each example its OWN graph (pod_probe_body already does this —
  merging lines into one graph produces hash-dependent parallel edges).

## What to build

1. **Probe**: add/extend claim probes in `tests/repro/probes.pl` to pin the
   exact rule — start-arrowhead glyphs for `<--` (and `<->`, which is
   claim `pod-bidir`) at each direction, and the label-placement condition
   (inline vs above-row). Capture `.observed` files. Update the `serves:`
   comments to `gently-7qz`.
2. **RED**: add scenarios to `tests/scenarios/ge_ascii_render.rs` (register
   in `tests/scenarios.rs` if adding a new file) whose expectations come
   from the `.observed` bytes — they must fail on the current tree. Wire
   any new `.espectacular/ge-ascii_render/*.toml` scenario flags to real
   cargo paths.
3. **GREEN**: implement in `crates/gently-core/src/render/ascii/`:
   - `polyline.rs`: `entry_dir` currently yields only the target-side
     arrow char; `draw_runs` stamps one arrow. Add the start-side arrowhead
     (mirror of the entry rule at the source attachment — direction glyph
     `v`/`^`/`>`/`<` per the oracle rule) — the edge STYLE drives whether a
     start arrowhead exists at all (`--` arrowless draws none — see bonus
     close below).
   - `canvas.rs` / `styles.rs`: label placement per the oracle condition
     (the current code always hosts inline in the widened gap, canvas.rs
     `column_geometry`/`draw_box` area).
   - Mind the existing glyph tables in `styles.rs` (`edge_run_at`,
     `edge_pattern`, `edge_gap_width`) — extend, don't fork.
4. **Spec re-derivation** (same discipline as oy2): amend `specs/ge-ascii_render.md`
   (c2 and/or a new row for bidirectional arrowheads + label placement),
   redeploy the openspec mirror IN THE SAME COMMIT as the corpus change
   (`tools/deploy_specs.py` — the spec-drift-gate compares worktree
   `openspec/` against the INDEX; a mid-flight deploy makes every
   intermediate commit red), and regenerate + stage the `specodelic/`
   artifacts (`*.toml`, `*_props.rs`, `*.check.json`) WITH the spec change
   (last session's note: gates regenerate specodelic/ corpus artifacts —
   stage them with the spec change or a follow-up commit is needed).
5. **Commit atomically** (this order, matching the oy2 pattern):
   - commit 1: `test:` probe + observed claims (probes.pl, claims/*.observed)
   - commit 2: `feat:` implementation + scenarios (+ `.espectacular` flags)
   - commit 3 (or fold into 2): `docs/chore(spec):` re-derived spec + openspec mirror + specodelic artifacts — mirror MUST be staged in the same commit as the corpus change.
   Before ANY commit: `git status --short`, stage ONLY files you authored.
6. **Bonus close check**: gently-bpq ("gently draws a target arrowhead on
   arrowless '--' edges; oracle draws none") is the same `draw_runs` arrow
   logic. If your fix covers it, verify against its evidence and
   `bd close gently-bpq` too (the oy2 run bonus-closed gently-0d6 + gently-1kv
   this way). Do not chase gently-51d ('##>' double-line arrowhead) — out
   of scope unless trivially already covered.
7. **Gates, then close, then push**: all gates below exit 0 →
   `bd close gently-7qz` (and bonus closes) → `bd export` if required →
   `git push` (project policy mandates push at completion).

## Hard scope guard

- Allowed files: `crates/gently-core/src/render/ascii/*.rs`,
  `tests/scenarios/ge_ascii_render.rs`, `tests/scenarios.rs`,
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-ascii_render.md`, `.espectacular/ge-ascii_render/*`,
  `specodelic/` regenerated artifacts, `docs/known-hard-cases.md` (only to
  move resolved rows out of the open-divergence tables),
  `.beads/issues.jsonl` (via bd close/export).
- Never edit: `openspec/specs/` directly (it's generated via deploy),
  `.espectacular/` outside `ge-ascii_render/`, `pretender.toml`, other
  `specs/*.md`, `.wai/resources/**`, `.wai/projects/graph-easy-port/runs/*`.
- Respect pretender ratchets: run the pretender gate before committing; if
  a pinned file breaches (abc > 30, function_lines > 40), split helpers —
  never raise limits.
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- **Commit hygiene** (standing): before ANY `git commit`, `git status --short`
  and stage only files YOU authored; attribute the message only to what the
  diff contains.
- **Spec mirror gotcha**: deployed `openspec/` copy must be STAGED in the
  same commit as the corpus change, or spec-drift-gate goes red on every
  intermediate commit.
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags; commands may be
  aliased with `-i` and will hang on prompts.
- **beads**: issues live in `.beads/issues.jsonl`; run `bd export` after
  `bd close` or the close is invisible to git.
- **Probe claims must make each example its OWN graph** (pod_probe_body
  does this — keep it that way; merged graphs give hash-dependent
  parallel edges).

## Completion criteria (all must exit 0)

```
cargo test -p gently-core                 # corpus + scenarios green
cargo clippy --workspace --all-targets -- -D warnings
ah check --run-tests                      # contract tests pass
just gates                                # repo gate suite
just examples-verify                      # byte-identical examples
just oracle-verify                        # oracle pipeline parity
spk lint openspec                         # 0 findings
git status --short                        # clean (only .whisper notes allowed)
git log origin/main..HEAD                 # empty (pushed)
bd show gently-7qz                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the oracle rule is not green after 2 attempts,
  or a scenario disagrees with `.observed` bytes in a way that implies a
  NEW oracle probe you cannot author, STOP, file a follow-up bead
  (`bd`), and report. Do not hand-wave a mismatch: the OBSERVED file wins.

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed (incl. bonus closes), and any deviations.
