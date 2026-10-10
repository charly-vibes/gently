# Subagent brief: gently-liz — remaining spec/tooling errors (cli failure transition + ikm overlap verification)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-liz` and is hands-off: you own
the re-derivation end-to-end. SPEC-RE-DERIVATION ticket.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-liz`
- The ticket's four items — IMPORTANT: gently-ikm closed several of
  them before you; VERIFY rather than redo:
  1. **cli.md model bug (STILL OPEN — your main work):** the only
     transition out of `reading` is t1 reading→rendering; there is no
     `reading→cli_failed` path, so a parse error cannot reach the
     failure state (which also breaks the failure-terminal rule: every
     failure terminal state must emit exactly one file-owned effect
     constraint — `cli_failed` declares `emits: [[cli.c4]]` but is
     unreachable). Read `specs/cli.md` model section + the failure
     terminal rule (see specs/linter-failure_shape.md note in AGENTS.md
     specodelic block).
  2. ge-oracle c4 remediation wording — ikm (commit 3d78481) extended
     c4; verify the drifted-oracle remediation is now correct.
  3. justfile `oracle-pin` reference + invalid cpanm syntax — ikm
     fixed; verify.
  4. oracle.pl pin checks version only, not commit — ikm fixed; verify.

## What to do

1. **Fix the cli.md model** (item 1): add the `reading→cli_failed`
   transition (guard: cite the constraint covering parse/render
   failure exit — c3 names "nonzero on any parse or render error", so
   `[[cli.c3]]` is the natural guard; pick what the rows support).
   Ensure `cli_failed` (failure terminal) emits exactly one file-owned
   effect constraint — c4 is the effect row (already declared); if the
   emits list needs adjusting to exactly one, do it. If the model
   needs the parse-error path distinguished (reading fails vs
   rendering fails), keep it minimal: one added transition is the
   floor. Also check whether a `cli_failed→?` state is needed by the
   every-state-used rule (cli_failed must appear in ≥1 transition as
   target — it already does once t5/your new one exists).
2. **Verify items 2-4** against current HEAD (`tools/oracle.pl`,
   `justfile`, `specs/ge-oracle.md`): if fixed by ikm, just record
   verification in the report; if a remnant remains, fix it (same
   allowed-files rules as ikm).
3. Deploy openspec mirrors in the SAME commit as any spec edit
   (`tools/deploy_specs.py`), recompile props via `just gates`.

## Hard scope guard

- Allowed: `specs/cli.md` + mirror + regenerated props; (items 2-4
  remnants only, if verification finds any:) `specs/ge-oracle.md` +
  mirror, `tools/oracle.pl`, `justfile`,
  `tests/fixtures/graph-easy/README.md`.
- NEVER edit: `crates/**`, other specs, `.wai/**`, `.github/**`.
- Do NOT push, do NOT close the bead, do NOT run `wai close`.
- Commit subjects end with `(gently-liz)`. File headers required.

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

**Verdicts** (item 1 fix description; items 2-4 verified-fixed / fixed-now / still-open)

**Commits** (hash + one line)

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, specs/cli.md model, ge-oracle
  c4, justfile, oracle.pl. Max 8 tool calls.
- **Minutes 5–12:** cli.md transition fix + spk lint.
- **Minutes 12–18:** items 2-4 verification.
- **Minutes 18–26:** mirrors + gates.
- **Minutes 26–30:** report.
Never re-read a file you have already read. A committed partial verdict
beats an uncommitted complete one.
