# Subagent brief: gently-m4i — text-parser statement surface: comma chains, anon groups, trailing node attrs

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-m4i` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The cluster (one statement surface, three pinned divergences)

All in gently's text parser (`crates/gently-core/src/parse/text/`):

1. **gently-m4i** (primary): comma-chained node lists parse-error — both
   `[ A ], [ B ], [ C ] --> [ D ]` and `[ A ] -> [ B ], [ C ], [ D ]`
   (gently says `", not recognized"`). Source: bloodgate.com
   manual/syntax.html (gently-ni9.2 mining).
2. **gently-58m** (bonus): anonymous-group syntax
   `( [ Bonn ] ) -> [ Berlin ]` parse error. Evidence:
   `tests/repro/claims/pod-anon-group.observed` (Parser.pm POD example).
3. **gently-8xd** (bonus): trailing node attribute block after node name
   silently dropped — `[ Big ] { label: "Huge"; }` keeps the default
   label. Source: bloodgate manual/syntax.html quoting section.

Evidence already pinned (do NOT re-record what exists):
`tests/repro/claims/bg-comma-lists.observed`,
`tests/repro/claims/pod-anon-group.observed`,
`tests/repro/claims/bg-trailing-node-attr.observed`.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-m4i` + `bd show gently-58m` + `bd show gently-8xd`
- Read `specs/ge-text_parser.md` IN FULL — the binding contract (this
  spec has been re-derived several times: 6j0, bzx, 8ol, oy2). Your
  change amends/extends rows for comma chains, anon groups, and trailing
  attr blocks.
- Read `crates/gently-core/src/parse/text/{mod,node,edge,attrs,clean}.rs`
  — the statement machinery (node.rs for attr blocks, edge.rs for chain
  lists, attrs.rs for the unquote rules from gently-8ol).
- Read the pinned-oracle source for the true rules (Graph::Easy v0.69 at
  /var/tmp/ge0.69): `Graph::Easy::Parser` — how a comma separates
  same-chain node lists (`_parse_list`? comma splitting around edge
  operators), how `( ... )` anon groups parse (group without name), and
  whether a node's attr block may follow the name (`_match_single_attribute`
  interplay with the 8ol unquote rules).
- wai search "text_parser" for accumulated patterns.

## What to build

1. **Probe check**: the three `.observed` files above cover the cluster.
   Add a claim probe ONLY if the pinned bytes leave a rule ambiguous
   (e.g. comma chains mixing groups/scopes, anon group with attrs
   `( [ A ] { color: red; } )`). Update `serves:` comments to include
   gently-m4i (+ 58m/8xd).
2. **RED**: add scenarios to `tests/scenarios/ge_text_parser.rs` (existing
   file — check its name) from pinned bytes — must fail on the current
   tree.
3. **GREEN**: implement in `crates/gently-core/src/parse/text/`:
   comma-separated node lists inside edge chains (both sides), anonymous
   group syntax, trailing attr block after node name. Keep the grammar
   structure — extend, don't fork. Respect the 8ol unquote rules
   (attrs.rs) — don't regress the pinned quote behavior.
4. **Spec re-derivation**: amend `specs/ge-text_parser.md`; regenerate +
   stage `openspec/` mirror and `specodelic/` artifacts IN THE SAME
   COMMIT as the corpus change. Proven dance (gently-51d/a0k): if
   pre-commit spec-drift-gate complains, land test+feat first, then
   restore and commit spec+mirror+artifacts+docs together.
5. **Commit atomically** (test pins → feat → spec/docs; stage only your
   files; pretender ratchets — if a function breaches abc>30 or 40 lines,
   split helpers, never raise limits).
6. **Bonus close**: `bd close gently-58m` and/or `bd close gently-8xd`
   when covered and verified against their evidence (a0k/7qz pattern).
7. **Gates, then close, then push**: completion criteria all exit 0 →
   `bd close gently-m4i` (+ bonus closes) → `bd export` → `git push`.

## Hard scope guard

- Allowed files: `crates/gently-core/src/parse/text/*.rs`,
  `crates/gently-core/tests/scenarios/ge_text_parser.rs`,
  `tests/scenarios.rs` (registration only if needed),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-text_parser.md`, `openspec/` (via deploy), `specodelic/`
  regenerated artifacts, `docs/known-hard-cases.md` (move resolved rows),
  `.beads/issues.jsonl` (via bd close/export).
- Never edit: other `specs/*.md`, `.espectacular/` (unless a new flag is
  truly needed), `pretender.toml`, `.wai/resources/**`,
  `.wai/projects/graph-easy-port/runs/*`.
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- **Commit hygiene** (standing): before ANY `git commit`, `git status
  --short` and stage only files YOU authored.
- **Spec mirror gotcha**: deployed `openspec/` copy staged in the SAME
  commit as the corpus change.
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
git status --short                        # clean (only .whisper/.wai session files)
git log origin/main..HEAD                 # empty (pushed)
bd show gently-m4i                        # shows closed
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
