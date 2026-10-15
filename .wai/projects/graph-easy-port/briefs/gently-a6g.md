# Subagent brief: gently-a6g — port upstream's fan placement engine (_near_places slot rings over grown clusters) for the hash-stable fan-in star

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-a6g` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## The divergence

The fan-in star (`[ n1 ] -> [ hub ]` × 4 sources) is hash-STABLE under
the pinned oracle (12 seeds checked, tier-1 byte-compat applies) —
evidence: `tests/repro/claims/fan-layout.observed`, SECOND example:

```
+----+     +-----+     +----+
| n1 | --> |     | <-- | n4 |
+----+     | hub |     +----+
           |     |     +----+
           |     | <-- | n2 |
           +-----+     +----+
             ^
             |
             |
           +-----+
           | n3  |
           +-----+
```

Key oracle mechanics (from the bead + 4ga decode):
- hub GROWS two east-side attachment rows (multi-attachment cluster
  growth) — n4 attaches at east row 1, n2 at east row 2 (slot (4,2)),
  leaving the hub column band 1 empty between them.
- n1 west-attached at the top; n3 hangs below on its own band.
- gently's current placement puts n2 below-hub instead and lacks the
  slot-ring evaluation — documented residual in
  `docs/known-hard-cases.md` under gently-4ga.

gently-4ga ALREADY fixed the router (contiguous jogged stacked-column
walks) — the RENDER is fine; only PLACEMENT diverges. Upstream engine:
`Graph::Easy::Layout::_near_places` — multi-celled slot rings evaluated
over GROWN node clusters, plus repair passes. Oracle at
`/var/tmp/ge0.69` (pin v0.69 @ ededa3d7). Check `bd show gently-4ga`
and `bd show gently-a6g` for the banked decode notes first — much of
the engine was already traced during 4ga.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-a6g`, `bd show gently-4ga` (banked notes).
- Read `tests/repro/claims/fan-layout.observed` IN FULL — target bytes
  for both examples.
- Read `docs/known-hard-cases.md` 4ga residual section.
- Read `crates/gently-core/src/layout/placement.rs` IN FULL — the
  current placement pass (ranks, attach_next_to, parallel channels,
  ports, slots/`Route::Slot` growth machinery just landed via
  gently-byq: indexed ports grow node row spans to `max(cy, N+1)` and
  route by slot — REUSE this machinery where it fits).
- Read `crates/gently-core/src/layout/paths.rs` (walk realization) and
  `crates/gently-core/src/render/ascii/canvas.rs` (geometry) as needed.
- Decode the upstream rule (budget ~10 min, the 4ga notes may already
  carry most of it): `Graph::Easy::Layout` `_near_places` (ring
  enumeration order, which ring slot wins ties), cluster growth for
  multi-attachment bands (`_grow`? how the hub's east side gains
  attachment rows), and the repair passes that settle the star shape.
  Confirm against the pinned oracle with fresh probes where the notes
  are thin; add probe claims to `tests/repro/probes.pl` (each example
  its OWN graph), capture `.observed`.
- wai search "fan" / "near_places" / "4ga" for accumulated patterns.

## What to build (strict TDD)

1. **RED** — scenario test pinning the fan-star oracle bytes (from
   `fan-layout.observed` example 2) in
   `crates/gently-core/tests/scenarios/ge_layout*.rs`; watch it fail on
   placement (n2 below-hub vs east-slot (4,2), missing hub growth).
2. **GREEN** — port the engine:
   - Cluster growth for multi-attachment bands on a node's side
     (hub cy grows to the number of same-side attachments — kin to the
     byq `max(cy, N+1)` growth; generalize honestly).
   - `_near_places` ring enumeration for the attach_next_to decision:
     slot rings around the placed target, ring order + tie rule from
     the oracle, yielding n1 west, n4 east-row-1, n2 east-row-2,
     n3 below-band.
   - Keep the byte-identical recorded fixtures green — every existing
     corpus pin (mixed_isolated, chain, port-*, so-flowchart-size,
     so-start-port-index) constrains the port; placement changes must
     be surgical (fan/shared-target paths only, or whatever the decode
     shows is upstream-shared).
3. **Spec + artifacts (same-commit discipline).**
   - Re-derive the placement rows in `specs/ge-layout.md` (c4 family)
     with fan-star fixture evidence; `spk lint` clean.
   - Deploy openspec mirror + recompile specodelic + rustfmt them —
     ALL staged in the same commit as the corpus change.
   - Wire new contract rows' TOMLs to real cargo test paths.
4. **Commit atomically / incrementally** — RED commit when the failing
   pin is real; feat commit at GREEN; spec commit right after
   (single feat commit also acceptable per the spec-drift-gate gotcha).
5. **Gates, then close, then push**: all exit 0 → `bd close gently-a6g`
   → `bd export` → `git push`.

## GOTCHAS

- **spec-drift-gate** blocks commits while mirrors are worktree-modified
  — same-commit discipline avoids it.
- **Pre-push hook stalls** (bd TTY) — if stalled >2 min, run gates
  manually green, then `LEFTHOOK=0 git push` and note it.
- clippy denies `single_match`.
- Probe claims: each example its OWN graph.
- Fan-OUT stays scoped-off (hash-dependent, tier-2, pod-fan class) —
  do NOT chase it; only the fan-IN star is tier-1.
- Budget: 60 minutes wall clock; commit incrementally so partial
  progress survives a timeout.

## Hard scope guard

- Allowed files: `crates/gently-core/src/layout.rs`,
  `crates/gently-core/src/layout/*.rs`,
  `crates/gently-core/tests/scenarios/*.rs`,
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `specs/ge-layout.md`, `openspec/` (via deploy), `specodelic/`
  regenerated artifacts, `.espectacular/ge-layout/*.toml` (only if
  truly needed), `docs/known-hard-cases.md` (update the 4ga residual +
  a6g row), `.beads/issues.jsonl` (via bd).
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
bd show gently-a6g                        # shows closed
```

## Spawn model

- Context ceiling: ≤ 40% — stop, commit what passes, report the delta.
- Follow-up threshold: if the ring/tie rule is not decoded after 2
  attempts, land the reachable sub-shapes with own pins and file the
  remainder — do NOT stop at zero commits. The OBSERVED file wins.

## Reporting

End with a report listing: commits (sha + one-line), the decoded ring/
growth rule (concise), gate commands + results, tickets closed,
deviations.
