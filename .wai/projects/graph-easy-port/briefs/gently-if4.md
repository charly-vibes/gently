# Subagent brief: gently-if4 — pretender baseline-adopt: gate mode must pass when only known (mechanically-shifted) violations remain

You are a pi subagent. You own ticket `gently-if4` end-to-end across TWO
repos. The orchestrator will verify with real gates — never claim work
the diff doesn't contain.

## The problem (from the bead)

Discovered during gently-go8's tree-wide rustfmt reformat: line counts
shifted, tripping 11 `file_lines`/`function_lines` violations against
pretender's recorded history. `pretender check --mode gate` hard-fails
even when the resolution report itself says "0 fixed, 11 still open,
0 new" — a legitimate mechanical reformat cannot pass gate mode without
refactoring or bypassing. Current accommodation in gently: lefthook runs
plain `pretender check` so the toml's lease governs.

## The fix (TWO repos)

### Repo A: `/var/home/sasha/para/areas/dev/gh/charly/pretender` (you are upstream)

Current HEAD `7237ee1` (v0.9.0 line). Implement baseline adoption:

1. **Baseline artifact**: gate-mode violation fingerprinting persists a
   baseline (e.g. `pretender-baseline.json` next to `pretender.toml`,
   or a config-declared path). Fingerprint = stable per-violation id
   (the resolution tracking already computes stable finding IDs —
   `pretender/src/history.rs`, feature pretender-u8a; REUSE that id
   scheme, do not invent a second one).
2. **Gate semantics**: with a baseline present, `check --mode gate`
   fails ONLY on violations whose stable id is NOT in the baseline
   (genuinely new). Known violations (still-open) pass with a summary
   line ("N known (baselined), 0 new"). No baseline → current behavior
   (fail on any violation) — fail-closed preserved.
3. **Adopt command**: `pretender adopt` (or `check --adopt`) writes/
   refreshes the baseline from the current scan, prints what it adopted
   (count + kind breakdown). Documented in `--help` and the book/docs
   (`book/`, `docs/` — follow repo conventions).
4. **TDD**: the repo has `tests/` — write the failing test first
   (gate fails on a shifted file with no baseline; adopt writes
   baseline; gate passes with only-baselined violations; a genuinely
   new violation still fails). Follow the repo's own AGENTS.md +
   justfile gates (`just gates` inside the repo — read the justfile
   first). Commit RED then GREEN there.
5. **Install**: `cargo install --path pretender` so gently's hooks pick
   the new binary (verify `pretender --version` bump or feature flag —
   do NOT downgrade; HEAD is 0.9.0).
6. Push the pretender repo (it's the user's own upstream; commit +
   push is authorized as part of this ticket).

### Repo B: `/var/home/sasha/para/areas/dev/gh/charly/gently` (meter + adoption)

1. **Meter (acceptance criterion)**: demonstrate
   `pretender check --mode gate` exits 0 when only baselined
   line-shift artifacts remain, and still fails when a genuinely new
   violation appears (verify both sides). Record the commands + output
   in the report.
2. Decide gently's hook posture: with gate mode now usable, consider
   restoring `--mode gate` in `lefthook.yml`'s ddl-pretender-gate ONLY
   if the meter passes consistently; otherwise keep the current
   accommodation and document why in the bead + a short note where the
   accommodation is described.
3. `bd close gently-if4`, `bd export`, push gently (LEFTHOOK=0 only
   after gates run green manually — the pre-push hook stalls on the
   bd TTY query; that's known).

## GOTCHAS

- The pretender repo worktree has PRE-EXISTING unrelated dirt
  (`.dont/db.cozo`, `.github/workflows/ci.yml`, deleted `.wai/.pipeline-run`
  etc.). Commit ONLY files you author — never `git add -A` there.
- clippy denies `single_match` in gently; follow each repo's own
  ratchets/lints.
- In gently, `just gates` re-dirties `specodelic/*_props.rs` (known
  regen-vs-rustfmt drift) — ignore/restore, don't commit.
- Do NOT touch gently's `specs/`, `openspec/`, `.espectacular/` for
  this ticket (no spec surface).
- Do NOT run `wai close` — the orchestrator owns session close.

## Orientation (do this first)

- `bd show gently-if4`
- pretender repo: read `AGENTS.md`, `justfile`, `pretender/src/history.rs`
  (stable finding IDs), `pretender/src/main.rs` (mode dispatch ~L1185,
  resolution delta ~L2770), `pretender/src/engine.rs`,
  `pretender/src/config.rs` (mode/baseline config surface).
- gently: `lefthook.yml` (ddl-pretender-gate), `pretender.toml`.
- wai search "if4" / "pretender" for patterns.

## Budget

60 minutes wall clock; commit incrementally (RED commit when the failing
test is real, GREEN when it passes, docs commit after, gently meter
commit last).

## Completion criteria

```
# pretender repo (cd ~/para/areas/dev/gh/charly/pretender)
just gates            # repo's own gates green
cargo test -p pretender
git log origin/main..HEAD   # empty (pushed)

# gently repo (cd ~/para/areas/dev/gh/charly/gently)
pretender --version   # new binary installed
# meter: adopt then gate passes on baselined-only violations (exit 0)
# meter: gate fails on a genuinely new violation (exit != 0)
cargo test --workspace
just gates
git status --short    # clean of authored files
git log origin/main..HEAD   # empty (pushed)
bd show gently-if4    # shows closed
```

## Reporting

End with a report listing: commits per repo (sha + one-line), the
fingerprint/id scheme chosen, meter evidence (exact commands + exit
codes), tickets closed, deviations.
