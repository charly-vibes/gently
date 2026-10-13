# Subagent brief: gently-irk — visual gallery: CLI ASCII + SVG per example, gallery-verify byte gate

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-irk` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is a self-contained docs+bin slice (complexity:m) — if you are not
writing the generator within 10 minutes, you are off track. Ship early rather
than polishing.

## Orientation (max 5 tool calls)

- `bd show gently-irk`
- `ls examples/` (7 example dirs, each has input.gv + expected.txt)
- Read `crates/gently-core/src/render/graphviz/mod.rs` (public entry point)
  and one `examples/simple-chain/` dir.
- `dot -V` (graphviz 16.1.0 present on this host).

## Facts already established (do not re-litigate)

- Upstream Graph::Easy v0.69 has NO SVG module — SVG is out of scope for v1.
  The visual path is gently's OWN library graphviz renderer → `dot -Tsvg`.
- The gallery SVGs are generated from gently's DOT render (NOT upstream).
- CLI does not yet wire --as dot (that is gently-4ig's job) — hence a
  standalone generator bin, NOT a CLI flag. Do NOT touch
  crates/gently-cli/src/pipeline.rs.

## Work

1. **Generator bin**: a small cargo example under
   `crates/gently-core/examples/gallery.rs` (or `crates/gently-cli/src/bin/`
   if the core example can't reach layout+render) that: reads an input.gv,
   parses (text or dot per the example), lays out, renders DOT via
   `gently_core::render::graphviz`, writes it. Keep it tiny and
   deterministic.
2. **SVG generation**: for each of the 7 examples, run the generator and pipe
   DOT through `dot -Tsvg` → commit as `docs/gallery/<name>.svg`.
3. **docs/gallery.md**: one section per example (all 7): the input snippet,
   the CLI ASCII render (quote `examples/<name>/expected.txt` — already
   byte-verified), and the committed SVG image (`![](gallery/<name>.svg)`).
   Short intro: what the gallery is, how SVG is produced (gently DOT →
   dot -Tsvg), link back to examples/ and docs/known-hard-cases.md.
4. **Drift gate**: `just gallery-verify` — regenerates every SVG via the
   same path and fails on byte mismatch (same culture as examples-verify).
   Wire it into `just gates` or list it in the README gates block.
5. **Links**: add a gallery link to README.md (near the examples pointer)
   and to examples/README.md (one line). Section anchors are stable — do not
   reorder anything.

## Gates (run all; all must be green)

- `just gates` (now including gallery-verify if you wired it in)
- `just examples-verify` (must stay byte-identical)
- `ah check` (0 findings)
- `cargo clippy` clean

## Ship

1. Commit: `docs: visual gallery — CLI ASCII + committed SVG per example, gallery-verify byte gate (gently-irk)`
2. `bd close gently-irk`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: gallery structure, generator location, gate
   results, commit sha.
