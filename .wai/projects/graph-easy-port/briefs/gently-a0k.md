# Subagent brief: gently-a0k — dot_parser graph-scope attribute statements (rankdir=LR)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-a0k` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The bug

The pinned oracle (Graph::Easy v0.69) parses graph-scope attribute
statements inside a digraph body — `rankdir="LR"` maps to `flow: east` —
and the gh-4 fixture as_txt output is already pinned. gently's dot parser
rejects them with `expected a statement`: `stmt.rs` `statement()` starts
at `endpoint()` and has no bare `key=value` graph-attr form. Evidence:
`tests/repro/claims/gh-4-nodes-first.observed` (probe
`tests/repro/probes.pl claim gh-4-nodes-first`, serves gently-ni9.3 —
update serves to include gently-a0k).

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-a0k` and `bd show gently-2n5` (same dot_parser surface:
  bare scope `{a b}` as edge-chain endpoint — bonus-close candidate)
- Read `crates/gently-core/src/parse/dot/stmt.rs` (`statement()`,
  `endpoint()`) — the rejection site; also `scope.rs` (scope handling)
  and `attrs.rs` (attribute parsing on the text side).
- Read the pinned-oracle source for the true rule (Graph::Easy:: v0.69 at
  /var/tmp/ge0.69): `Graph::Easy::Parser` `_parse_attributes` /
  graph-attr handling in dot parsing — how `rankdir` maps to `flow`
  (which rankdir values map to which flow? LR->east; check RL/TB/BT too),
  and what other graph-scope attrs are accepted (colorscheme, title per
  the .observed output).
- Read `specs/ge-dot_parser.md` IN FULL — the binding contract; your
  change amends/re-derives rows (attribute statements + scope endpoints).
- wai search "dot_parser" for accumulated patterns.

## What to build

1. **Probe**: extend probes for the exact rule — graph-scope attr
   statements (`rankdir="LR"`, quoted and bare), bare scope as edge-chain
   endpoint (`{a b} -> baz` links BOTH a and b — gently-2n5), and any
   rankdir→flow mapping values. Capture `.observed`; update `serves:`
   comments to gently-a0k (+ gently-2n5).
2. **RED**: add scenarios to `tests/scenarios/ge_dot_parser.rs` (existing
   file) from pinned bytes — must fail on the current tree.
3. **GREEN**: implement in `crates/gently-core/src/parse/dot/`:
   graph-scope `key=value` statements (attr list form, including quoted
   values) and bare-scope endpoints. Keep the lexer/statement structure —
   extend, don't fork. `rankdir` maps per the oracle rule.
4. **Spec re-derivation**: amend `specs/ge-dot_parser.md` rows; regenerate
   + stage `openspec/` mirror and `specodelic/` artifacts IN THE SAME
   COMMIT as the corpus change. NOTE (proven dance from gently-51d): if
   pre-commit spec-drift-gate complains, land test+feat first, then
   restore and commit spec+mirror+artifacts+docs together.
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files).
6. **Bonus close**: if gently-2n5 is covered and verified against its
   evidence, `bd close gently-2n5` too (oy2/7qz bonus-close pattern).
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-a0k` → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/parse/dot/*.rs`,
  `crates/gently-core/tests/scenarios/ge_dot_parser.rs`,
  `tests/scenarios.rs` (registration only if needed),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-dot_parser.md`, `openspec/` (via deploy), `specodelic/`
  regenerated artifacts, `docs/known-hard-cases.md` (move resolved rows),
  `.beads/issues.jsonl` (via bd close/export).
- Never edit: other `specs/*.md`, `.espectacular/` (unless a new flag is
  truly needed), `pretender.toml`, `.wai/resources/**`,
  `.wai/projects/graph-easy-port/runs/*`.
- Respect pretender ratchets (abc > 30, function_lines > 40 — split
  helpers, never raise limits).
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- **Commit hygiene** (standing): before ANY `git commit`, `git status
  --short` and stage only files YOU authored.
- **Spec mirror gotcha**: deployed `openspec/` copy staged in the SAME
  commit as the corpus change (see the 51d stash dance if needed).
- **Probe claims must make each example its OWN graph** (pod_probe_body
  already does this).
- **Non-interactive shells**: use `-f`/`-rf`/`-y` flags.
- **beads**: run `bd export` after `bd close`.

## Completion criteria (all must exit 0)

```
cargo test -p gently-core
cargo clippy --workspace --all-targets -- -D warnings
ah check --run-tests
just gates
just examples-verify
just oracle-verify
spk lint openspec
git status --short                        # clean (only .whisper notes allowed)
git log origin/main..HEAD                 # empty (pushed)
bd show gently-a0k                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% of the model window — if exceeded, stop, commit
  what passes, and report the remaining delta.
- Follow-up threshold: if the oracle rule is not green after 2 attempts,
  or pinned bytes disagree in a way that implies a NEW probe you cannot
  author, STOP, file a follow-up bead, and report. The OBSERVED file wins.

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed (incl. bonus closes), and any deviations.
