# Subagent brief: gently-4sq — oracle.pl fail_pin: verify the pinned checkout's git commit

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-4sq` and is hands-off: you own
the fix end-to-end.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-4sq`
- Context: tools/oracle.pl `fail_pin` (line ~26) + the pin checks
  around line 40-60. gently-ikm landed version + checkout-path +
  seed checks; the checkout's actual git COMMIT is still unverified —
  a drifted checkout (right version, wrong commit) would be accepted.
- The pin: `PIN_COMMIT` constant in oracle.pl (v0.69 @
  `ededa3d787ad89ac532c578c06390e8a7b270499`).

## What to build

1. **oracle.pl**: when the pinned checkout exists, verify its git
   commit — `git -C $PIN_LIB/.. rev-parse HEAD` (resolve the right
   directory: the checkout root containing Graph/Easy.pm; be careful —
   the lib dir is `<checkout>/lib`) and compare to `PIN_COMMIT`
   (allow the abbreviated `ededa3d7` prefix form — a commit id matches
   when the full sha starts with it). On mismatch → `fail_pin` with a
   typed message naming both shas. Gracefully skip the check when the
   checkout is not a git repo (dist tarball) — in that case the
   existing "cannot be verified" remediation wording already covers
   it, but emit a warning line naming the limitation.
2. **Test arm**: extend `crates/gently-core/tests/scenarios/ge_oracle.rs`
   p4's drift matrix with the wrong-commit case (fake checkout: right
   version, wrong commit → typed error naming both shas) — mirror the
   existing drift-case pattern. If the p4 arm structure makes this a
   separate focused test, keep it inside the p4 family.
3. No spec edits needed (c1 already names the commit in the pin).

## Hard scope guard

- Allowed: `tools/oracle.pl`,
  `crates/gently-core/tests/scenarios/ge_oracle.rs` (p4 arm only).
- Never edit: specs, justfile, fixtures, src/**, .wai/**.
- Do NOT push, do NOT close the bead (close gently-4sq yourself after
  gates — this is the last leaf ticket), do NOT run `wai close`.
- Commit subjects end with `(gently-4sq)`. File headers required
  (perl file header comment if not present).

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
ah check
ah check --run-tests
just gates
```

## Report format (end with this)

## Report

**Commits** (hash + one line)

**Commit-check behavior** (match rule; tarball warning wording)

**Gates run** (the five lines)

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, oracle.pl pin section, p4 arm.
  Max 8 tool calls.
- **Minutes 5–10:** p4 drift-case RED.
- **Minutes 10–20:** GREEN commit check.
- **Minutes 20–27:** gates; `bd close gently-4sq` if green.
- **Minutes 27–30:** report.
Never re-read a file you have already read. A committed RED beats an
uncommitted almost.