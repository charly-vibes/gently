# Subagent brief: gently-4ig — wire `--as dot/html/txt` per the upstream format map

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-4ig` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is complexity:L (CLI + spec + pins) — if you are not writing the
RED pin within 10 minutes, you are off track. Ship rather than polish: the
wiring itself is small; the spec discipline is the bulk.

## Orientation (max 5 tool calls)

- `bd show gently-4ig`
- Read `crates/gently-cli/src/pipeline.rs` (SUPPORTED_FORMATS line 28,
  EXTENSION_FORMATS line 33) and `crates/gently-cli/src/main.rs`.
- Read `specs/cli.md` (c2/c4/p2/p4) and `tests/scenarios/cli.rs`.
- Oracle methods check: `grep -n "as_html\|as_graphviz\|as_txt\|as_svg" /var/tmp/ge0.69/Graph-Easy-0.69/bin/graph-easy`

## Facts established (do not re-litigate)

- Upstream v0.69 bin has `as_html`, `as_graphviz`, `as_txt` methods — html,
  txt, dot(->graphviz) ARE upstream CLI formats. `svg` has NO as_svg method
  (dies → 255); `boxart` is NOT an upstream format at all. vcg/gdl/graphml:
  check the oracle before claiming — the extension map is not the method map.
- Gently's library renderers all exist and are spec-pinned: ge.graphviz_render,
  ge.html_render, ge.txt_render. The CLI just doesn't route to them.
- TDD discipline (ge.oracle c7 tiers): byte-compat pins only where the oracle
  probe is hash-stable and within the 10s envelope; scope-offs documented.

## Work (red → green → spec → ship)

1. **RED**: extend `tests/scenarios/cli.rs` (scenarios::cli) with pins per
   wired format: `--as dot`, `--as html`, `--as txt` on a corpus fixture —
   expectations = oracle bytes captured via the probe harness
   (`PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl
   claim <name>`, one claim per graph, register the claim serving
   gently-4ig, commit the `.observed`). Pin exit-255 for `--as boxart`,
   `--as svg` (+ vcg/gdl/graphml if the oracle confirms they die).
2. **GREEN**: route `--as dot|graphviz` → graphviz renderer, `html` → html
   renderer, `txt` → txt renderer in pipeline.rs/main.rs. Keep exit-255
   message shape for upstream-unsupported formats. Extension inference
   (output filename without --as) must now resolve html/txt/dot to real
   renders — follow upstream's resolution order.
3. **SPEC same-commit**: amend `specs/cli.md` c2/c4 (+ p2/p4 predicates) to
   the new surface, deploy the openspec mirror via `tools/deploy_specs.py`,
   regenerate `specodelic/` artifacts. **Known trap**: spec-drift-gate
   compares worktree `openspec/` against the INDEX — stage the spec + mirror
   + artifacts IN THE SAME COMMIT as the corpus change (the established
   stash dance), or the hook fails on intermediate commits.
4. Check the README formats table row ("render via CLI ... CLI wiring
   pending" note) — update the one line that is now stale (small edit, do
   not restructure; leave the gallery/SVG pipeline description as-is, it
   already says the generator may be retired once --as dot lands — do not
   retire it in this ticket).

## Gates (run all; all must be green)

- `cargo test` (all scenarios green), `cargo clippy` clean
- `ah check` (0 findings) — and the cli contract tests execute
- `just gates`, `just examples-verify`, `just gallery-verify` (all must stay green)

## Ship

1. Commit(s): RED pin commit, then feat+spec+mirror+artifacts+docs —
   follow the same atomic-content discipline as previous runs
   (`feat(cli): wire --as dot/html/txt ... (gently-4ig)`)
2. `bd close gently-4ig`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: formats wired, exit-255 formats confirmed, spec
   delta, gate results, commit shas.
