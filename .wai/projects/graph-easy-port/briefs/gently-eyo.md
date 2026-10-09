# Subagent brief: gently-eyo — implement ge.html_render

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-eyo` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## Why

Delegating because this is a full capability implementation (new HTML
renderer module + probe round + red/green/refactor cycle) that fits a
fresh session's budget, while the orchestrator session is conserving
context for the run loop. It is the next blocker on the critical path to
gently-ef5 (cli). Two renderers just landed — `render/ascii/` (0cq) and
`render/boxart/` (css) — giving you the exact module pattern to follow.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-eyo`
- Read `specs/ge-html_render.md` IN FULL — c1–c5 + model + p1–p5 are the
  binding contract. Every property row becomes a passing test named for
  its id.
- Read the existing probe evidence — captured from Graph::Easy v0.69 @
  `ededa3d787ad89ac532c578c06390e8a7b270499`:
  - `tests/repro/claims/href-escaping.observed` — `&` in href is emitted
    RAW (unescaped) in the href attribute while label text goes through
    escaping; `<a href='...'>` wraps the label; single-quoted attrs.
  - `tests/repro/claims/td-colspan.observed` — a node spans
    `colspan=4 rowspan=4` with `class='node'`; multi-line label uses
    `<br>`; document skeleton: `<table class="graph" cellpadding=0
    cellspacing=0>` with `<!-- row N line M -->` comments and empty
    `<tr></tr>` filler rows.
  - Generate YOUR OWN probes for what's not yet observed (c3 edge-style
    CSS classes, c4 embedded CSS rules, c5 color declarations + shape
    classes): extend `tests/repro/probes.pl` following its existing
    conventions and commit as `tests/repro/claims/html-*.observed`.
    The `.observed` files are BINDING — if observed behavior disagrees
    with the spec prose, the OBSERVED bytes win and the divergence gets
    a doc-comment note in the test + your report.
- Study the just-landed renderer patterns:
  - `crates/gently-core/src/render/ascii/{mod,canvas,styles}.rs`
  - `crates/gently-core/src/render/boxart/{mod,styles,columns,canvas,polyline}.rs`
  Both consume the SAME `ge.layout` grid contract. The html renderer is
  a table emitter over that grid (upstream `As_html.pm`).
- Contract stubs: `.espectacular/ge-html_render/p1..p5.toml` point at
  `scenarios::ge-html_render::p1..p5` — these use the STALE dashed
  filter convention. Fix them to the working convention
  `ge_html_render::pN` (see how the css run fixed ge-boxart_render's
  tomls in commit d0e995e), create `tests/scenarios/ge_html_render.rs`
  with those exact test names, register in `tests/scenarios.rs`.
- `wai search "html"` and `wai search "renderer"` for accumulated
  patterns.

## What to build

The full `ge.html_render` capability as a new module
`crates/gently-core/src/render/html/`:

- **c1/p1** — every grid cell → exactly one `td` with the content-kind
  CSS class (node, edge, group, empty).
- **c2/p2** — node cells carry class + label; `link` attribute becomes
  `a href` wrapping the label; label text HTML-escaped; href per the
  OBSERVED escaping rule (raw `&` in href).
- **c3/p3** — edge styles → documented border-image CSS classes; edge
  labels as text on edge cells.
- **c4/p4** — emitted document embeds CSS rules for every class it uses
  (self-contained output).
- **c5/p5** — `fill`/`background`/`color` attributes → CSS color
  declarations via the upstream W3C color-name scheme; `shape` → its
  documented CSS class.
- Properties p1–p5 as `ge_html_render::p1..p5` scenario tests.
- Wire the renderer into the render dispatch (mirror how boxart is
  dispatched). Do NOT touch the CLI `--as` plumbing — that is
  gently-ef5's; cli.md c2 owns it.

## Red/green/refactor sequence (TDD, commits in this order)

1. `test:` probe commit — extended `probes.pl` + new `html-*.observed`
   claim files.
2. `test:` RED scenarios — `ge_html_render.rs` p1–p5 failing against a
   minimal/stub module skeleton (if needed to compile), registered in
   scenarios.rs, `.toml` filters fixed.
3. `feat:` GREEN implementation — html module + dispatch wiring.
4. `refactor:` (only if needed) — separate commit, no behavior change.

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
bd show gently-eyo                        # shows closed
```

## Spawn model

- Model: same family as orchestrator (default `pi` model).
- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the probes reveal the oracle behaves
  fundamentally differently from the spec's c1–c5 (not just details),
  STOP after the probe commit, file a follow-up bead describing the
  divergence, and report — do not improvise a spec amendment.

## Reporting

End with a report listing: probe files added, commits (sha + one-line),
gate command results, scenario p1–p5 status, any spec divergences found,
and any follow-up beads filed.