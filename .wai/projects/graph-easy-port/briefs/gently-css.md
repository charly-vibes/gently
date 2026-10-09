# Subagent brief: gently-css — implement ge.boxart_render

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-css` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## Why

Delegating because this is a full capability implementation (new renderer
module + probe round + red/green/refactor cycle) that fits a fresh
session's budget, while the orchestrator session is conserving context
for the run loop. The ascii renderer groundwork (gently-0cq, just
closed) gives you the canvas/glyph module pattern to follow.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-css`
- Read `specs/ge-boxart_render.md` IN FULL — c1–c4 + model + p1–p4 are
  the binding contract. Every property row becomes a passing test named
  for its id.
- Read the existing probe evidence — captured from Graph::Easy v0.69 @
  `ededa3d787ad89ac532c578c06390e8a7b270499`:
  - `tests/repro/claims/shape-outline-collapse.observed` — boxart shape
    vocabulary behavior: rect/rounded/ellipse/circle/diamond/hexagon/
    house/parallelogram/triangle all pass through as `html-class=graph`;
    `parallelogram_alt`, `trapezoid`, `trapezoid_alt` are REJECTED;
    border-style=bold → `border: solid 4px`, wide → `solid 1em`.
  - Generate YOUR OWN probes for the boxart glyph tables (the c1/c3
    Unicode tables are NOT yet probed): extend `tests/repro/probes.pl`
    to capture exact boxart output bytes for all border styles, all edge
    styles, and junction/neighbourhood combinations, and commit them as
    `tests/repro/claims/boxart-*.observed`. The probe harness pattern is
    established — follow `probes.pl`'s existing probe + observed-file
    conventions (pinned oracle commit in the header line).
- Study the just-closed ascii renderer for the module pattern:
  `crates/gently-core/src/render/ascii/{mod,canvas,styles}.rs` —
  glyph tables in `styles.rs`, char-grid canvas in `canvas.rs`, thin
  `mod.rs`. The boxart renderer consumes the SAME `ge.layout` grid
  contract (see `canvas.rs` doc comments and how `ascii/mod.rs` builds
  the blank grid from the layout cell grid).
- Existing wiring stubs: `.espectacular/ge-boxart_render/p1..p4.toml`
  already point at `scenarios::ge-boxart_render::p1..p4` — create
  `tests/scenarios/ge_boxart_render.rs` with those exact test names and
  register it in `tests/scenarios.rs`.
- `wai search "boxart"` and `wai search "ascii_render"` for accumulated
  patterns (CJK byte-length divergence is gently-dcp's, label-wrap is
  out by design — same exclusions apply here unless boxart probes say
  otherwise).

## What to build

The full `ge.boxart_render` capability as a new module
`crates/gently-core/src/render/boxart/`:

- **c1/p1** — Unicode box-drawing glyph set per border style (single,
  double, dotted, dashed, dot-dash, dot-dot-dash, wave, bold,
  double-dash …), one glyph per cell.
- **c2/p2** — junction glyphs chosen from the actual neighbouring cells
  (corners + T-junctions always combine correctly).
- **c3/p3** — edge-style → Unicode glyph table; horizontal repeat units
  may span several columns per cell width, matching upstream column
  counts exactly.
- **c4/p4** — shape attribute changes the outline per the upstream
  Unicode shape table, for every shape in the upstream vocabulary
  (including the REJECTED names recorded in shape-outline-collapse).
- Properties p1–p4 as `ge_boxart_render::p1..p4` scenario tests.
- Wire the renderer into the render dispatch wherever ascii_render is
  dispatched (find the format/dispatch site in `render/` and mirror it).

## Red/green/refactor sequence (TDD, commits in this order)

1. `test:` probe commit — extended `probes.pl` + new `boxart-*.observed`
   claim files.
2. `test:` RED scenarios — `ge_boxart_render.rs` p1–p4 failing against a
   minimal/stub module skeleton (if the skeleton needs to exist to
   compile), registered in scenarios.rs.
3. `feat:` GREEN implementation — boxart module + dispatch wiring.
4. `refactor:` (only if needed) — separate commit, no behavior change.

Never hand-wave a byte mismatch: the `.observed` files are binding. If a
probe surprises you (oracle does something the spec prose doesn't
predict), follow the OBSERVED behavior and note the divergence in the
test's doc comment + report.

## Completion criteria (all must exit 0)

```
cargo test -p gently-core                 # corpus + scenarios green
cargo clippy --workspace --all-targets -- -D warnings
pretender check --mode gate               # no new violations
ah check --run-tests                      # contract tests pass
just gates                                # repo gate suite
spk lint openspec                         # 0 findings
git status --short                        # clean
git log origin/main..HEAD                 # empty (pushed)
bd show gently-css                        # shows closed
```

## Spawn model

- Model: same family as orchestrator (default `pi` model).
- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the boxart probes reveal the oracle behaves
  fundamentally differently from the spec's c1–c4 (not just glyph
  details), STOP after the probe commit, file a follow-up bead describing
  the divergence, and report — do not improvise a spec amendment.

## Reporting

End with a report listing: probe files added, commits (sha + one-line),
gate command results, scenario p1–p4 status, any spec divergences found,
and any follow-up beads filed.