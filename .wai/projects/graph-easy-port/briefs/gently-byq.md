# Subagent brief: gently-byq — indexed start/end port attrs `{start: front,N}` route edges to numbered side slots

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-byq` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The divergence

Claim probe `so-start-port-index` (gently-ni9.4, SO 47323345): the oracle
(v0.69 @ ededa3d7) routes edges to INDEXED ports on a node's side —
`{ start: front,0 }`, `{ start: front,3 }`, `{ end: front,2 }` —
producing the recorded 12-line interleave where each edge attaches at a
DIFFERENT vertical slot along the (tall) box side:

- `[ Msg Server ] -- 1. Number X of unread --> { start: front,0; } [ Client ]`
  leaves Msg Server's east side at its TOP slot (slot 0).
- `[ Msg Server ] -- 3. X Msg --> { start: front,3; } [ Client ]`
  leaves at slot 3 rows down and runs down the flanking channel.
- `[ Client ] -- 2. read X --> { end: front,2; } [ Msg Server ]`
  ENTERS Msg Server's east side at slot 2.

Evidence already pinned (do NOT re-record):
`tests/repro/claims/so-start-port-index.observed` (seeds 0 and 1 both
recorded, byte-identical). gently ignores the index and produces a
9-line loop layout.

gently-89r already landed SIDE-constrained ports (`start: right` → dip
below, `start: left` → bend above; `port_dip` in
`crates/gently-core/src/layout.rs`). Its doc-comment explicitly defers
indexed forms to THIS ticket: "Indexed forms (`front,N`) select a
numbered slot on the side — variable node heights, out of scope
(gently-byq)." gently-awd has since landed variable node heights
(`columns`/`rows` spans, `node_span`), which unblocks this work.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-byq`
- Read `tests/repro/claims/so-start-port-index.observed` — the target
  bytes. Also `tests/repro/claims/bg-start-right.observed` and the
  port-side fixtures (`port-*`) for the 89r side rules.
- Read `crates/gently-core/src/layout.rs` — `Route` enum, `port_dip`
  (the `front`/`forward` arm currently returns None/default).
- Read `crates/gently-core/src/layout/placement.rs` — `apply_ports`.
- Read `crates/gently-core/src/layout/paths.rs` — `mark_slot`,
  route→cell-path realization; where the path's endpoints touch the
  boxes.
- Read the render attachment code — where an edge path's first/last
  cell meets the node box border (`crates/gently-core/src/render/ascii/edges.rs`,
  `canvas.rs`, and the geometry pass for span drawing).
- Decode the upstream rule BEFORE coding (pin the oracle v0.69 at
  `/var/tmp/ge0.69`; methods live in reopened package
  `Graph::Easy::Edge::Cell` inside As_ascii.pm — patch THAT stash when
  tracing; `_printfb_line`/`_printfb_ver` are in package
  Graph::Easy::Node):
  - How does an indexed port choose the attachment row? (Expect: slot N
    counts attachment positions along the side, 0-based from the
    flow-start corner; verify with fresh probes.)
  - What happens when N exceeds the side's positions (clamp? wrap?
    ignored?). Probe it.
  - How does `front` compose with the flow direction for the INDEX
    numbering (front = flow direction side — for east flow, east/west
    sides; count from where)?
  - Write any new probe claims into `tests/repro/probes.pl`, each
    example its OWN graph (pod_probe_body convention — merged graphs go
    hash-dependent), capture `.observed` against the pinned oracle.
- wai search "port" and "89r" for accumulated patterns.

## What to build (strict TDD)

1. **RED** — scenario test in
   `crates/gently-core/tests/scenarios/` wired to the recorded
   `so-start-port-index` bytes + your new probe pins; watch it fail on
   the indexed cases (side-only cases must stay green).

2. **GREEN** — extend the port machinery:
   - Parse `start:`/`end:` values as `side[,index]` (the existing
     `split(',')` already isolates the side).
   - Thread the indexed slot through `Route` (likely a variant like
     `PortSide { below/above/…, slot }` or parameters on PortBelow/
     PortAbove — keep the 89r arms' recorded shapes byte-identical for
     index-less ports).
   - Path realization: the edge's attachment row on the box side moves
     to slot N; gap-row accounting (`mark_slot`) must reserve rows so
     the interleave fits — derive the rule from the oracle bytes, not
     guesswork.
   - Non-east flows stay out of scope (no tier-1 fixture pins indexed
     ports there — keep the existing transformed-layout behavior).

3. **Spec + artifacts (same-commit discipline).**
   - Amend `specs/ge-layout.md` (and/or `ge-ascii_render.md` if the
     attachment-row rule lands render-side): amend the 89r port row or
     add an indexed-ports row; extend/re-derive the predicate. `spk lint`
     clean.
   - Deploy mirror + recompile specodelic artifacts + rustfmt them
     (`rustfmt --edition 2021 specodelic/*.rs` — specodelic#15) and
     stage ALL in the same commit as the corpus change.
   - Wire any new contract row's `.espectacular` TOML to the real cargo
     test path.

4. **Commit atomically**; given the spec-drift-gate gotcha prefer ONE
   feat commit containing test+fix+spec+mirror+artifacts.

5. **Gates, then close, then push**: all criteria exit 0 →
   `bd close gently-byq` → `bd export` → `git push`.

## GOTCHAS

- **spec-drift-gate** blocks commits while mirrors are worktree-modified
  — same-commit discipline avoids it entirely.
- **Pre-push hook stalls** (bd TTY query) — if stalled >2 min, abort,
  run gates manually green, then `LEFTHOOK=0 git push` and note it.
- clippy denies `single_match` — use `if let`.
- Probe claims: each example its OWN graph; pod_probe_body already does
  this.
- Do NOT pin global PERL_RAND_SEED (drifts other recordings).

## Hard scope guard

- Allowed files: `crates/gently-core/src/layout.rs`,
  `crates/gently-core/src/layout/*.rs`,
  `crates/gently-core/src/render/ascii/*.rs` (attachment geometry only),
  `crates/gently-core/tests/scenarios/*.rs`,
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-layout.md`, `specs/ge-ascii_render.md`,
  `openspec/` (via deploy), `specodelic/` regenerated artifacts,
  `.espectacular/ge-layout/*.toml` + `.espectacular/ge-ascii_render/*.toml`
  (only if truly needed), `docs/known-hard-cases.md` (move the byq row),
  `.beads/issues.jsonl` (via bd).
- Never edit: other `specs/*.md`, `pretender.toml`, `.wai/**`,
  `.whisper/**`, justfile.
- Do NOT run `wai close` — the orchestrator owns session close.

## Repo facts (boilerplate)

- Commit hygiene: stage only files YOU authored.
- `bd export` after `bd close`.
- Non-interactive shells: `-f`/`-rf`/`-y`.

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
bd show gently-byq                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% — stop, commit what passes, report the delta.
- Follow-up threshold: if the oracle rule is not decoded after 2
  attempts, or the recorded bytes imply a rule you cannot pin, STOP,
  file a follow-up bead, report. The OBSERVED file wins.

## Reporting

End with a report listing: commits (sha + one-line), the decoded
upstream indexed-port rule (concise), gate commands + results, tickets
closed, deviations.
