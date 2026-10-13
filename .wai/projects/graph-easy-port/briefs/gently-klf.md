# Subagent brief: gently-klf — triangle back-edge layout divergence (+ bonus: minlen, textwrap)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-klf` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The cluster (one layout surface, three pinned divergences)

All in gently's layout (`crates/gently-core/src/layout/{flow,placement}.rs`):

1. **gently-klf** (primary): the POD "Three nodes, interlinked" example
   (`[ Bonn ] -> [ Berlin ] -> [ Hamburg ]` + `[ Bonn ] -> [ Hamburg ]`)
   is hash-STABLE in the oracle across 4 seeds (`tests/repro/admission.tsv`
   — byte-compat tier 1 per specs/ge-oracle.md c7), yet gently's layout
   diverges by 2 columns on the back-edge routing/indent. Evidence:
   `tests/repro/claims/pod-triangle.observed`.
2. **gently-yhg** (bonus): `minlen` edge attribute ignored — oracle
   stretches the edge (`[ Bonn ] --> [ Koblenz ] --> { minlen: 3; }
   [ Frankfurt ]`). Evidence: `tests/repro/claims/bg-minlen-bypass.observed`.
3. **gently-emd** (bonus): `textwrap` graph/edge attribute ignored —
   oracle wraps long labels (`graph { textwrap: auto; }`, edge
   `textwrap: 10`). Evidence: `tests/repro/claims/bg-textwrap.observed`.
   (Source for both: bloodgate.com manual — gently-ni9.2 mining.)

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-klf` + `bd show gently-yhg` + `bd show gently-emd`
- Read `specs/ge-layout.md` IN FULL (or its actual filename — check
  `ls specs/`; the layout spec re-derived by 4nx/89d has the flow/layering
  contract). Your change amends/extends rows.
- Read `crates/gently-core/src/layout/flow.rs` (Flow parsing, layering
  ranks) and `placement.rs` (axis transforms, channel assignment).
- Read the pinned-oracle source (Graph::Easy v0.69 at /var/tmp/ge0.69):
  `Graph::Easy::Layout::{Scan,Path,Rank}` — back-edge routing (which
  layer/column a back-edge takes), `minlen` handling (extra gap
  columns?), `textwrap` (where wrapping happens — layout or render?).
- wai search "layout" for accumulated patterns. NOTE: known carve-out —
  the layout c3 strict-earlier falsification and the groupless-layout
  notes from the b0z run (bg-anon-chain not reproducible from gently's
  layout) may matter here.

## What to build

1. **Probe check**: the three `.observed` files are pinned. Add a claim
   probe ONLY if a rule is ambiguous (e.g. minlen on other edge styles,
   textwrap interplay with wide glyphs). Update `serves:` comments to
   gently-klf (+ yhg/emd).
2. **RED**: extend the layout scenarios in `crates/gently-core/tests/scenarios/`
   (find the ge_layout scenario file) from pinned bytes — must fail on
   the current tree.
3. **GREEN**: implement in `crates/gently-core/src/layout/`:
   back-edge routing/indent per the oracle rule, `minlen` gap columns,
   `textwrap` (if it turns out to be a RENDER concern — label cell
   wrapping — implement it in `render/` and note the layering in the
   spec). Keep the layering/transform structure — extend, don't fork.
4. **Spec re-derivation**: amend the layout spec; regenerate + stage
   `openspec/` mirror and `specodelic/` artifacts IN THE SAME COMMIT as
   the corpus change (51d/a0k stash dance if the drift gate complains).
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — abc>30 / 40 lines → split helpers).
6. **Bonus close**: `bd close gently-yhg` and/or `bd close gently-emd`
   when covered and verified against their evidence.
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-klf` (+ bonus) → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/layout/**/*.rs`,
  `crates/gently-core/src/render/**` ONLY IF textwrap lands render-side
  (justify in the commit message), `crates/gently-core/tests/scenarios/*.rs`
  (layout-related), `tests/scenarios.rs` (registration only if needed),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  the layout spec in `specs/`, `openspec/` (via deploy), `specodelic/`
  regenerated artifacts, `docs/known-hard-cases.md` (move resolved rows),
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
bd show gently-klf                        # shows closed
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
