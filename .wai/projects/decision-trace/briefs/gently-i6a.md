# Subagent brief: gently-i6a — decision-trace spec corpus (spec-first)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace, edition 2021). An orchestrator has claimed ticket
`gently-i6a` and is hands-off: you own the work end-to-end (spec authoring,
mirror deploy, contract registration, commits). Do NOT push and do NOT
`bd close` — the orchestrator owns ship.

## TIME BUDGET (binding): 30 minutes. Spec-only slice, no implementation.

## Orientation (max 10 tool calls)

- `export WAI_PROJECT=decision-trace`
- `bd show gently-i6a`
- Read `.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md` IN FULL — D1, D2, D3, D4 are the decisions you encode; E1–E4 all PASSED, D7 go/no-go is GO (record: `2026-10-14-decision-d7-go-no-go-openspec-proposal.md`).
- Read the proposal `openspec/changes/add-decision-trace/proposal.md` and tasks.md task 1.
- Format reference: `specs/ge-oracle.md` (four-layer specodelic format that passes `spk lint` — copy its structure, not its content).
- Contract-toml template: `.espectacular/ge-oracle/p7.toml` (staged-contract pattern — flags reference cargo paths that land in a later slice).
- Deploy tool: `tools/deploy_specs.py` (check its usage line first).
- `spk explain` works offline if you need a rule refresher.

## Desired outcome

`specs/decision-trace.md` exists in the specodelic four-layer format, the
openspec mirror is deployed in the SAME commit, and
`.espectacular/decision-trace/p*.toml` registers the executing contracts.
This is the spec-first gate: nothing in sections 2/3/5 of tasks.md
implements until this spec exists.

## Spec content (from the design doc — hardened by E1–E4)

Cover these decision rows as Constraints (pick `invariant`/`effect`/`advisory`
kinds deliberately; cite the design doc + experiment evidence in prose):

1. **D1 — versioned decision-trace schema**: the decision trace is a
   versioned artifact with one sub-vocabulary per pipeline stage (parse,
   graph model, rank/col, bands, spans, route classification, bends,
   glyphs), each event carrying its governing spec id. Seed vocabulary:
   the Route enum in `crates/gently-core/src/layout.rs` (READ IT — do not
   invent stage names that contradict the code). A `version` field is
   mandatory from day one.
2. **D2 — oracle normalization is a committed, tested artifact (CORR-001)**:
   the oracle→Dec normalization is the explicit table in
   `tests/repro/trace/normalization-table.md` (committed, rows N1–N8), with
   the pairwise faithfulness property: fixtures whose oracle dumps differ
   at stage S normalize to traces differing in S's image; canonical
   reordering of a dump normalizes to an identical trace.
3. **D3 — trace-diff reports minimal cuts (CORR-002)**: divergence is a set
   over the decision dependency poset; the diff reports its minimal
   antichain, never a unique "first".
4. **D4 — committed seed-pinned stage-observable dumps**: oracle dumps are
   committed companions carrying the oracle pin header AND the hash seed as
   an explicit field, and mark per-stage observability; tier-2
   (hash-seed-dependent) fixture classes stay excluded from the committed
   dump corpus — exclusion is by admission probe naming (ge.oracle c7).
5. **Adjudication**: the registry (committed at 95f434c,
   `tests/repro/trace/` — find the adjudication files) governs (a)/(b)/(c)
   classification; E4's dry run (gently-btf) is the recorded precedent.

Keep the Model small but valid: every transition needs a guard, every
declared state must be used, failure terminal states must emit exactly one
file-owned effect constraint with a unique variant head. Prefer `unit`-kind
properties; if you declare a `law`, you must name its cases (identity +
associativity floor). Every constraint needs exactly ≥1 property with
`derives_from` pointing at it; no property may be an orphan. No self
references. Unique ids.

## Contract registration (staged-contract pattern)

Register `.espectacular/decision-trace/pN.toml` for each property (archetype
and shape like ge-oracle's p7.toml), with `cargo` flags pointing at
`decision_trace::pN` paths. Do NOT create `tests/scenarios/` files — those
land with gently-p7y; the no-tests-ran backlog is EXPECTED here (this is
exactly how ge-oracle p7 was staged).

## Red/green shape

- RED: the registered p*.toml contracts + spec rows exist while
  `ah check` reports them in the no-tests-ran backlog (staged, not failing).
- GREEN for THIS slice: `spk lint specs/decision-trace.md` → 0 findings;
  `ah check` → 0 findings other than the expected no-tests-ran backlog
  entries for the new contracts; `just gates` green; `cargo test` count
  unchanged (207).

## Commit protocol (one spec commit + no follow-ups)

1. Run `spk compile` and stage `specodelic/` artifacts WITH the spec change
   (gates regenerate them). THEN `rustfmt --edition 2021 specodelic/*.rs`
   (specodelic#15: generated files are not fmt-clean; pre-push will fail
   otherwise).
2. Stage `specs/decision-trace.md` + the deployed
   `openspec/specs/decision-trace.md` mirror + `.espectacular/decision-trace/`
   IN THE SAME COMMIT — spec-drift-gate compares worktree openspec/ against
   the INDEX; an unstaged mirror stalls every intermediate commit (protocol
   confirmed, gently-awd notes).
3. Commit message: `feat(spec): decision-trace spec corpus — D1-D4 hardened spec + contracts (gently-i6a)`.

## Out of scope

- Any change to `crates/gently-core/src/trace.rs`, `tests/repro/probes.pl`,
  `tests/repro/trace/`, `docs/known-hard-cases.md` (p7y/h1b/9nn/vhc own
  those).
- Implementing any scenario test.
- Push, `bd close`, openspec archive.

## Follow-up threshold

If `spk lint` keeps failing after 3 targeted fixes, STOP, commit nothing,
and report the exact rule_id + row in your summary — do not weaken the spec
to pass the linter.

## Report back (your final message, ≤200 tokens)

- commit sha(s), spec row inventory (constraints/properties/states),
  contracts registered, gates status, any deviation from this brief.
