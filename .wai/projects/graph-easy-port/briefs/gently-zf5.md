# Subagent brief: gently-zf5 — "add an example" how-to guide

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-zf5` and is hands-off: you own the
implementation end-to-end.

## TIME BUDGET (binding)

You have a HARD 30-minute budget. Orientation must finish in under 5 minutes.
This ticket is docs-only (complexity:s) — if you are not writing the guide
within 10 minutes, you are off track. Ship early rather than polishing.

## Orientation (max 4 tool calls)

- `bd show gently-zf5`
- Read `examples/README.md` (~36 lines) — the admission rule is stated there.
- Skim `docs/known-hard-cases.md` lines 1-12 (the admission rule cross-ref)
  and `just --list` for the gate names.

## Work

The doc surface has a landing page and reference pages but no how-to guides.
Write the highest-value missing how-to: **how to add a byte-verified
example**. It encodes the currently-implicit admission rule:
input.gv + expected.txt from the pinned oracle, `just examples-verify`
green, publish only when byte-identical.

Shape: 10–15 lines max, Martini-glass style (guided steps first, then
pointers for exploration). Location: extend `examples/README.md` with a
"## Adding an example" section (preferred — it is where the reader already
is), or a short `docs/add-an-example.md` linked from it. Every step must be
a real runnable command:

1. mine/verify the oracle bytes (the differential fixture corpus in
   `tests/fixtures/graph-easy/`, pinned Graph::Easy v0.69, `PERL_HASH_SEED=0`)
2. commit `input.gv` + `expected.txt` (pin header stripped — see the README
   paragraph for exactly what expected.txt is)
3. `just examples-verify` green — only publish when byte-identical
4. cross-reference the admission rule in
   `docs/known-hard-cases.md` (what happens when it is NOT byte-identical:
   it becomes a tracked divergence row, not an example)

Do NOT touch README.md (main), do NOT touch docs/known-hard-cases.md beyond
at most adding a back-reference, do NOT change any gate or script.

## Gates (run all; all must be green)

- `just gates`
- `just examples-verify` (must stay byte-identical — you changed no code)
- `ah check` (0 findings)

## Ship

1. Commit ONLY the guide: `docs: add-an-example how-to — the byte-verified admission rule (gently-zf5)`
2. `bd close gently-zf5`
3. Do NOT push — the orchestrator pushes.
4. Print a final summary: where the guide lives, gate results, commit sha.
