# Subagent brief: gently-cof — define/link project jargon (Specodelic, beads, wai)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-cof` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is docs-only (complexity:s) — if you are not editing within
10 minutes, you are off track. Ship early rather than polishing.

## Orientation (max 4 tool calls)

- `bd show gently-cof`
- Read `README.md` (64 lines) and `docs/known-hard-cases.md` lines 1-12.
- Check `ls .wai/resources/pipelines/` / `spk explain` if you need the exact
  Specodelic phrasing (or reuse what README already says: "capability corpus
  in `specs/` (Specodelic four-layer format)").

## Work

Three bits of project jargon are unresolvable by a human reader from the docs:

1. **Specodelic** — README (Status/paragraph and/or the corpus mention) drops
   "Specodelic four-layer format" with no definition or link. Add a one-line
   definition (what a spec is, that the capability corpus lives in `specs/`)
   plus a link to the tool (crates.io: specodelic —
   https://crates.io/crates/specodelic). Inline parenthetical is fine; do NOT
   build a glossary page.
2. **beads** — `docs/known-hard-cases.md` line ~6 says "tracked in the issue
   tracker (beads)" with no link. Link beads to its repo:
   https://github.com/gastownhall/beads
3. **wai** — README carries the wai badge (links to
   https://github.com/charly-vibes/wai) but the workflow terms are never
   explained for readers. Add a link (same target as the badge) at first
   workflow mention, or a one-line parenthetical.

Keep it to first-use definition, not a glossary section. Touch ONLY
`README.md` and `docs/known-hard-cases.md` (and only where the jargon
appears). Do NOT reorder sections (gently-lkf just stabilized them), do NOT
add a gallery (gently-irk's job), do NOT change any facts.

## Gates (run all; all must be green)

- `just gates`
- `just examples-verify` (must stay byte-identical — you changed no code)
- `ah check` (0 findings)

## Ship

1. Commit ONLY the docs: `docs: define/link Specodelic, beads, wai jargon (gently-cof)`
2. `bd close gently-cof`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: what was linked where, gate results, commit sha.
