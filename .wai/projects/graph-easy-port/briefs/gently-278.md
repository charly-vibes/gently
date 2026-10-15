# Subagent brief: gently-278 — pin + document the as_txt group-endpoint oracle defect (class b: gently is right(er))

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-278` and is hands-off: you own
the implementation end-to-end.

## The ticket

SMALL slice — documentation + pinning, no behavior change to chase.
Upstream defect (Graph::Easy v0.69 @ ededa3d7, DECIDED NOT TO FILE
upstream — user decision 2026-10-14; this beads ticket IS the item-3
evidence of the decision-trace E4 (b)-checklist, gently-btf):

- Oracle repro: `( [ Bonn ] ) -> [ Berlin ]` — `as_txt` prints only the
  `( [ Bonn ] )` group block; **Berlin is gone** (the group-end handler
  makes the closed group the node stack, so the edge links GROUP→Berlin
  and the as_txt declaration pass never reaches Berlin; as_ascii renders
  BOTH boxes via a different path — upstream is self-inconsistent).
  Deterministic (seeds 0/1 byte-identical).
- gently's behavior: group block + `[ Berlin ]` listed (txt); dashed
  group border in ascii — renderable and deterministic under ge.layout c1.
- Verdict: class (b) — oracle defect, gently is right(er).

## What to build (TDD where a pin lands)

1. **Probe pin (RED→GREEN is trivial here; the pin IS the test)**:
   - Check `tests/repro/claims/pod-anon-group.observed` — it already
     serves gently-58m with the oracle evidence. Add a gentle-side pin
     claim (e.g. `as-txt-group-endpoint-omission`) to
     `tests/repro/probes.pl` serving gently-278: pin gently's CURRENT
     bytes for `( [ Bonn ] ) -> [ Berlin ]` in BOTH as_txt and as_ascii
     (each example its OWN graph, pod_probe_body convention). Capture
     `.observed` from gently's own output (this is a raw-scaling /
     class-b claim — verified against gently's invariants, NOT the
     oracle; state that in the probe header comment).
   - Add the scenario test wired to those bytes in the appropriate
     `tests/scenarios` file (ge.txt_render family), running green.

2. **Known-hard-cases row**: add a `gently-278` row to
   `docs/known-hard-cases.md` in the correct table (text parser /
   txt render section), with the repro one-liner, the oracle-vs-gently
   delta, and the class (b) verdict (oracle defect, not filed upstream
   per user decision — evidence: this bead + the probe pin).

3. **Spec row (light)**: in `specs/ge-txt_render.md` (and/or
   `ge-text_parser.md` if the group-end linkage is the better home),
   add an advisory/effect row documenting the divergence: gently's
   as_txt lists the group-endpoint-connected node; upstream omits it
   (oracle defect, class b, reference the claim pin). Keep `spk lint`
   clean (guard required, observability satisfied by the pin's probe
   row — follow existing advisory-row patterns in the corpus).
   - Deploy openspec mirror + recompile specodelic + rustfmt them,
     staged in the SAME commit.

4. **Commit / gates / close / push**: gates all exit 0 →
   `bd close gently-278` → `bd export` → `git push` (LEFTHOOK=0 only
   after gates run green manually — pre-push stalls on bd TTY).

## GOTCHAS

- spec-drift-gate: stage mirrors+artifacts in the same commit as the
  spec change (or fold everything into one feat commit).
- `just gates` re-dirties `specodelic/*_props.rs` — restore, don't commit.
- Probe claims: each example its OWN graph.
- Budget: 30 minutes wall clock. Commit incrementally.

## Hard scope guard

- Allowed files: `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `crates/gently-core/tests/scenarios/*.rs` (pin wiring only),
  `docs/known-hard-cases.md`, `specs/ge-txt_render.md`
  (and/or `ge-text_parser.md`), `openspec/` (via deploy),
  `specodelic/` regenerated artifacts, `.espectacular/*.toml` (only if
  truly needed), `.beads/issues.jsonl` (via bd).
- Never edit: layout/parser source (NO behavior change),
  `pretender.toml`, `.wai/**`, `.whisper/**`, justfile.
- Do NOT run `wai close` — the orchestrator owns session close.

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
bd show gently-278                        # shows closed
```

## Reporting

End with a report listing: commits (sha + one-line), gate commands +
results, tickets closed, deviations.
