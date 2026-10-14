# Subagent brief: gently-fb1 — dot_parser: bare scope nested in a named subgraph joins the enclosing group

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-fb1` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The divergence

Oracle Graph::Easy v0.69 (@ ededa3d7): `digraph G { subgraph outer { { x } } }`
puts `x` INTO group `outer` (`GROUP(outer: x)`) — the oracle's group_stack
survives bare-scope boundaries (`_new_nodes` receives the enclosing group
regardless of scope nesting). gently keeps such nodes ungrouped: its
`ScopeSt` carries `group: None` for bare scopes, and `intern()` reads only
`self.scopes.last().and_then(|s| s.group)`.

Only diverges INSIDE a named subgraph — top-level bare scopes are ungrouped
in both implementations (pinned by spec `ge.dot_parser` c4; that pin stays
true because no named group encloses a top-level bare scope).

Evidence: oracle probe output recorded during gently-a0k (session log);
spec c4 pins bare-scope ungrouping only at graph level.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-fb1`
- Read `crates/gently-core/src/parse/dot/mod.rs` — `ScopeSt`, `intern`
  (group membership at creation), `P.scopes`.
- Read `crates/gently-core/src/parse/dot/scope.rs` — `scope_body` pushes
  `ScopeSt`; NOTE `graph_attr_stmt` ALREADY implements the oracle's
  enclosing-group lookup (`self.scopes.iter().rev().find_map(|s| s.group)`),
  which is the exact fix shape for `intern`.
- Read `crates/gently-core/src/parse/dot/stmt.rs` — bare `{ }` scope
  statement handling and how `scopes.last().is_group` drives
  replace-vs-push stack semantics (do NOT change stack semantics).
- Read `specs/ge-dot_parser.md` IN FULL — the binding contract. Your
  change amends the c4 row (bare-scope grouping) or adds a new row; also
  check the p-row that pins top-level bare-scope ungrouping.
- Confirm the oracle behavior yourself before coding:
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl claim bare-scope-in-named-subgraph`
  (add the claim to probes.pl first — see below).

## What to build (strict TDD)

1. **RED — probe + scenario test first.**
   - Add claim `bare-scope-in-named-subgraph` to `tests/repro/probes.pl`,
     serving gently-fb1. Each example its OWN graph (pod_probe_body
     convention). Cases to pin:
     - `digraph G { subgraph outer { { x } } }` → x joins group outer
       (oracle GROUP(outer: x)).
     - top-level bare scope `digraph G { { x } }` → x ungrouped (both).
     - bare scope nested in bare scope inside named group → still joins
       the named group (group_stack is transitive).
     - bare scope whose group-enclosing boundary was closed:
       `subgraph a { q } { x }` → x ungrouped (group popped).
   - Capture `.observed` against the pinned oracle
     (`PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib`).
   - Write the failing scenario test in
     `crates/gently-core/tests/scenarios/ge_dot_parser.rs` (find the
     existing c4/p4 wiring; follow the pattern). RED: run it, watch it
     fail on the nested case only.

2. **GREEN — the fix.**
   - In `intern()` (`parse/dot/mod.rs`): replace
     `self.scopes.last().and_then(|s| s.group)` with the enclosing-named-
     group lookup `self.scopes.iter().rev().find_map(|s| s.group)` —
     the same expression `graph_attr_stmt` already uses. This makes
     membership follow the oracle's group_stack (survives bare scopes,
     pops at named-group close) while top-level bare scopes stay
     ungrouped.
   - Do NOT change `ScopeSt` fields, stack replace-vs-push semantics, or
     `graph_attr_stmt`.

3. **Spec + artifacts (same-commit discipline).**
   - Amend `specs/ge-dot_parser.md`: update the c4 row (or add a row)
     stating bare-scope nodes join the innermost ENCLOSING named group;
     top-level bare scopes remain ungrouped. Extend/re-derive the
     matching predicate. Keep EARS/`spk lint` clean.
   - Deploy the mirror: `python3 tools/deploy_specs.py` (or the repo's
     deploy recipe) and stage `openspec/` in the SAME commit as the spec.
   - Recompile specodelic artifacts: `spk compile` per repo workflow, then
     `rustfmt --edition 2021 specodelic/*.rs` (specodelic#15 — output is
     not fmt-clean). Stage regenerated `specodelic/` files in the same
     commit.
   - If a new contract test row needs a `.espectacular` TOML, wire it to
     the real cargo test path (follow existing p-rows).

4. **Commit atomically** (probe+observed → test pins RED → feat+spec
   single commit is ALSO acceptable given the spec-drift-gate gotcha
   below; stage only your files; pretender ratchets — if a function
   breaches abc>30 or 40 lines, split helpers, never raise limits).

5. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-fb1` → `bd export` → commit `.beads` export → `git push`.

## GOTCHAS (from whisper/bd notes — read carefully)

- **spec-drift-gate blocks ANY commit while mirrors are worktree-modified.**
  If specs are amended before the RED test commit, the test-only commit
  cannot land. Proven workaround (gently-cij): fold test+feat+spec into a
  SINGLE feat commit. Prefer that here.
- **Pre-push hook stalls nondeterministically** (bd TTY query under
  lefthook). If it stalls >2 min: abort, run the gate suite manually
  (they're all fast), then `LEFTHOOK=0 git push` with a note in the
  commit/report that gates were run manually.
- clippy pre-push denies `single_match` — use `if let`.
- Non-interactive shells: use `-f`/`-rf`/`-y` flags.

## Hard scope guard

- Allowed files: `crates/gently-core/src/parse/dot/*.rs`,
  `crates/gently-core/tests/scenarios/ge_dot_parser.rs`,
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-dot_parser.md`, `openspec/` (via deploy), `specodelic/`
  regenerated artifacts, `.espectacular/ge-dot_parser/*.toml` (only if a
  new contract row truly needs it), `.beads/issues.jsonl` (via bd).
- Never edit: other `specs/*.md`, `pretender.toml`, `.wai/**`,
  `.whisper/**`, justfile.
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- Commit hygiene: before ANY `git commit`, `git status --short` and stage
  only files YOU authored.
- Probe claims must make each example its OWN graph.
- `bd export` after `bd close`.

## Completion criteria (all must exit 0)

```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
ah check --run-tests
just gates
just examples-verify
just oracle-verify
spk lint openspec
git status --short                        # clean (only session files)
git log origin/main..HEAD                 # empty (pushed)
bd show gently-fb1                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the fix is not green after 2 attempts, or the
  oracle evidence disagrees in a way that implies a NEW rule you cannot
  pin, STOP, file a follow-up bead, and report. The OBSERVED file wins.

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed, and any deviations.
