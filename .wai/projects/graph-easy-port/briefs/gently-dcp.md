# Subagent brief: gently-dcp — verify renderers report findings 8a-c against the implementations

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-dcp` and is hands-off: you own
the re-derivation end-to-end. SPEC-RE-DERIVATION ticket: the deliverable
is verdicts + amended spec rows (+ follow-up beads only if the
implementations diverge from the amended contract).

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-dcp`
- The claims (origin: a review report finding 8, probed against UPSTREAM
  only — at probe time gently had no renderer code):
  1. html c2 (line 18): "link URLs are HTML-escaped" — probe says
     upstream href keeps RAW ampersands (probe:
     `tests/repro/claims/href-escaping.observed`).
  2. html c4 (line 20): "emitted document embeds the CSS rules for every
     class, self-contained" — probe says upstream `as_html` emits NO CSS
     (only `as_html_file` does; probe: html-css-rules.observed).
  3. html "td count equals cell count" — probe says colspan/rowspan make
     td count differ (probe: td-colspan.observed).
  4. ascii: "fixed edge glyphs" (....>/- -> shapes) — probe says edge
     glyphs depend on cell width (probe: ascii-render-tables.observed);
     "14 shapes" collapse into 5 distinct outlines (probe:
     shape-outline-collapse.observed); "bold/wide/broad distinct" —
     probe says they render identically.
- **The implementation landscape has CHANGED**: html renderer shipped
  (gently-eyo), ascii renderer shipped (gently-0cq), boxart shipped
  (gently-css), graphviz shipped (gently-b4v). Re-derive each claim
  against BOTH the pinned oracle and the shipped implementation.

## What to do

For each claim (html c2, html c4, ascii shape/glyph rows):
1. Read the spec row + its deriving property predicate.
2. Verify what the SHIPPED gently implementation does (read
   `crates/gently-core/src/render/html/**` and
   `src/render/ascii/**`; run a scratch test if needed — allowed via
   `cargo test --test scenarios` runs or a temporary test you DELETE
   before committing).
3. Verdict:
   - Row matches oracle + implementation → leave.
   - Row overclaims vs oracle → amend with a re-derivation note (style:
     mirror the gently-0kg amendments — read one amended row first,
     e.g. specs/ge-graphviz_render.md c4) and align the property
     predicate.
   - Implementation diverges from the (amended) contract → file a
     follow-up bead naming the exact gap (do NOT implement; do NOT
     amend the spec to match a wrong implementation silently).
4. Specs you may touch: `specs/ge-html_render.md`, `specs/ge-ascii_render.md`
   ONLY (rows named above or their deriving properties). Deploy
   openspec mirrors in the same commit (`tools/deploy_specs.py`; read
   its help first). Spec + mirror + props recompile (`just gates`
   regenerates specodelic props) must be committed together.

## Hard scope guard

- Allowed: the two spec files + mirrors + regenerated props,
  `tests/repro/**` (new probes only if a genuine question is unpinned —
  record them),
  `.espectacular/ge-html_render/*.toml` / `.espectacular/ge-ascii_render/*.toml` (flags only if needed).
- NEVER edit: `crates/gently-core/src/**`, `crates/gently-cli/**`,
  scenario test files (follow-up beads own implementation changes),
  other specs, `.wai/**`.
- File follow-up beads (`bd create`) for any implementation gap; do NOT
  close gently-dcp; do NOT push.
- Commit subjects end with `(gently-dcp)`. File headers required.

## Exact gates (all green before you report done)

```sh
spk lint specs/
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
ah check
ah check --run-tests
just gates
```

## Report format (end with this)

## Report

**Verdicts** (html c2 escaped-links, html c4 self-contained CSS, html
td-count, ascii edge glyphs, ascii shapes, ascii bold/wide/broad — each
<row-holds|row-amended|impl-gap-bead> + one-line evidence)

**Commits** (hash + one line)

**Probes added** (or "none")

**Follow-up beads filed** (ids + titles, or "none")

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, both spec files, the three
  probe .observed files. Max 8 tool calls.
- **Minutes 5–12:** verify implementation behavior (scratch tests).
- **Minutes 12–20:** verdicts + amendments + mirrors + props recompile.
- **Minutes 20–27:** gates, follow-up beads.
- **Minutes 27–30:** report.
Never re-read a file you have already read. A committed partial verdict
beats an uncommitted complete one.
