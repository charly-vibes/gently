# Subagent brief: gently-ni9.5 — build the human-facing examples surface

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-ni9.5` and is hands-off: you own it
end-to-end.

## TIME BUDGET (binding)

HARD 30-minute budget (`timeout 1800` enforced). Orientation ≤5 tool calls.
Build incrementally; commit once at the end (single slice).

## Orientation (max 5 tool calls)

- `bd show gently-ni9.5` and `bd show gently-ni9` (epic: source rules)
- `ls tests/repro/claims/` — pod-* probes (gently-ni9.1) and gh-4-nodes-first
  (gently-ni9.3) hold ~60 oracle-verified input/output pairs ALREADY. Mine
  them for examples; do not invent new ones. `tests/fixtures/graph-easy/` too.
- `.wai/projects/graph-easy-port/research/2026-10-11-pod-mining-notes.md` and
  `2026-10-11-gh-issues-triage.md` — hard-case findings.
- `.wai/projects/graph-easy-port/designs/2026-10-09-decision-byte-compat-scope.md`
  — what belongs on the known-hard-cases page.
- `justfile` — recipe style; `bd list` — open divergence bugs for the
  hard-cases page.

## Work

1. `examples/` dir: ≥5 examples, each a directory `examples/<name>/` with
   `input.gv` (graph text) + `expected.txt` (oracle-verified output, exactly
   the bytes from the recorded probes/fixtures) + one-line comment in a
   shared `examples/README.md` describing it. Pick diverse coverage: a
   simple chain, inline styles, groups, DOT input, a known hard case.
   Every example MUST be byte-verified: generate expected output from gently
   CLI (`cargo run -- --as ascii <input`) and cross-check it equals the
   recorded oracle bytes from the claims/fixtures. If gently diverges on an
   example you want, skip it (it's one of the filed bugs) — pick another.
2. `examples/README.md` at top: one-liners + how they were verified.
3. README.md: replace/extend the status area with a tiny quickstart —
   install/build, one parse-to-ascii how-to (using example 1), one `--as`
   example, pointer to examples/ and the hard-cases page. Keep it lean;
   keep the wai badge; do not restructure beyond that.
4. `docs/known-hard-cases.md` (or examples/HARD-CASES.md if docs/ is unwanted):
   table of the open divergence beads (id, one-line behavior delta, oracle
   vs gently) + scope-off items from the byte-compat decision. Title it as
   compatibility status, not a to-do list.
5. Add justfile recipe `examples-verify`: for every examples/<name>/, render
   input.gv with the CLI and diff against expected.txt; fail on mismatch.
   Keep the recipe POSIX-ish (match existing justfile style).

## Must gate (from bd acceptance)

- README quickstart + ≥5 examples each with committed expected output
- `just examples-verify` passes (renders + diffs every example)
- `ah check --run-tests` and `cargo test` stay green (146)
- Meter: `just examples-verify && just gates`
- Anti-goal: no example published without its expected-output file.

## Ship

1. One commit: `docs(examples): human-facing examples surface + verify gate (gently-ni9.5)`
2. `bd close gently-ni9.5`
3. Do NOT push — orchestrator pushes.
4. Print final summary: examples added, gates, commit sha.