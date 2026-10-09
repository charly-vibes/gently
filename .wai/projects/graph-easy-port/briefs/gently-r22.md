# Subagent brief: gently-r22 — graph_model spec re-derivation (c1/c2/c3 + delete-node add_edge)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-r22` and is hands-off: you own
the work end-to-end. The orchestrator will verify with real gates — never
claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-r22` — the bead description lists the findings; this
  brief operationalizes them.
- Read `specs/ge-graph_model.md` IN FULL — c1, c2, c3 and properties
  p1, p2, p3 are the rows you will amend.
- Read `tests/repro/README.md` — the probe harness is your evidence
  source. Pin recipe and hazards are there.
- `wai search "graph_model"` and `wai search "spec re-derivation"` —
  accumulated patterns.
- Precedent: the gently-6j0 amendment is the workflow template — probe
  evidence → amend corpus row → redeploy mirror → same-commit staging →
  gates. (Brief: `.wai/projects/graph-easy-port/briefs/gently-6j0.md`.)

## Desired outcome

`specs/ge-graph_model.md` matches the pinned oracle's actual behavior
(Graph::Easy v0.69 @ `ededa3d787ad89ac532c578c06390e8a7b270499`) on the
finding classes below, with every amendment grounded in committed probe
evidence. After this bead, implementation follow-ups can be briefed from
the amended spec without re-litigating behavior.

## Finding classes to re-derive (from the bead)

a) **c1 — anonymous nodes ARE named and reachable:** the spec says
   anonymous nodes are "unnamed and cannot be referenced again after
   creation", but the oracle names them `#0`, `#1`, … and `node('#1')`
   FINDS them. Evidence: probe `node-unnamed` (already recorded:
   `tests/repro/claims/node-unnamed.observed` — anonymous nodes are NAMED
   `#1`,`#3`; `node('#1')` finds them). Amend c1/p1 so the rule is the
   oracle's actual rule: anonymous nodes get `#N` names (check the exact
   numbering scheme — probe more cases if the observed file doesn't pin
   it) and are referenceable like any named node.

b) **c2 — attributes are stored through unquoting/decomposition, NOT
   verbatim:** the spec says values are "stored verbatim (no loss)" and
   border components are "computed at assignment time". Report finding 7b:
   a RED read back comes back as red, and the border is
   decomposed/recomposed — i.e. the oracle runs assignment values through
   the store-layer unquote (see gently-8ol's two-layer unquote findings
   and `tests/repro/claims/attr-quote-value.observed` for the store-layer
   behavior) and normalizes/decomposes complex attributes. Add a probe
   pinning: assign `color: red` → read back; assign a border string →
   read back the decomposed form (style/width/color per component, or
   whatever the oracle actually returns). Amend c2/p2 to state the real
   rule.

c) **c3 — edges can target groups:** the spec says every edge references
   exactly two NODES. The oracle accepts edges with a group as an
   endpoint. Probe this (e.g. `( A ) --> ( B )` group-to-group edges, and
   node-to-group), record what the model stores (endpoint rewritten to
   all group members? a group reference?), and amend c3/p3 accordingly.

d) **delete-node then add_edge succeeds:** the spec says no dangling edge
   ever escapes a mutation, but `add_edge` on a DELETED node succeeds in
   the oracle (implicitly re-creating it?). Probe: create nodes, delete
   one, add_edge referencing it; observe whether the node reappears
   (and with which attributes — probably a fresh bare node) and whether
   old incident edges dangle. Amend c3/p3 to the oracle's actual rule.

## What to build

1. **New probes where missing.** `node-unnamed` exists; b/c/d findings
   have no probes yet. Add claim probes to `tests/repro/probes.pl`
   following the existing probe style (see `attr-quote-value` and
   `sharp-label` for shape), record
   `tests/repro/claims/<name>.observed` files, and extend the probe
   table in `tests/repro/README.md` with the beads each probe serves.
   Every spec claim you change must cite committed observed bytes —
   prose is not evidence.
2. **Amend the corpus spec** `specs/ge-graph_model.md`: rows c1/c2/c3,
   their properties p1/p2/p3 (predicates must stay consistent with the
   amended constraints), and any added rows. Respect the specodelic lint
   rules (guards non-null, unique ids, EARS statement, no conjoined/universal
   ids, coverage/derivation, every-state-used — changing c1's wording may
   ripple into the model/transitions table; lint will tell you).
3. **Redeploy the mirror:** run `python3 tools/deploy_specs.py` so
   `openspec/specs/ge-graph_model/spec.md` matches, and STAGE both the
   corpus spec and the deployed mirror in the SAME commit — the
   spec-drift-gate compares worktree `openspec/` against the INDEX.
   Existing `.espectacular/ge-graph_model/p1–p5.toml` stubs stay as-is.
4. **Update `tests/repro/README.md`** probe table with new probes and the
   beads they serve.

## Red → green → refactor shape

- **RED:** for each finding, run the probe and capture the delta between
  the current spec row's claim and the observed oracle bytes. Commit the
  probe scripts + observed evidence first (evidence before amendment).
- **GREEN:** amend the spec rows to match observed reality; `spk lint
  specs/` clean; redeployed mirror staged.
- **REFACTOR:** tighten prose, ensure p-predicates read as checkable
  statements, README table coherent.

## Hard scope guard

- Allowed files: `specs/ge-graph_model.md`,
  `openspec/specs/ge-graph_model/spec.md` (via deploy script only),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `tests/repro/README.md`.
- Never edit: `crates/**` (model implementation is separate follow-up
  work), other `specs/*.md`, `.espectacular/**`, `openspec/specs/**`
  other than the ge-graph_model mirror, `.wai/resources/**`, this brief.
- Do NOT push, do NOT close the bead, do NOT run `wai close` — the
  orchestrator owns ship.

## Exact gates (all must be green before you report done)

```sh
spk lint specs/
just gates
cargo test --workspace
ah check
PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl claim <new-probe>   # each new probe reproduces cleanly
```

## Repo facts (boilerplate)

- **Commit hygiene:** before ANY `git commit`, `git status --short` and
  stage only files YOU authored; unstage foreign files. Attribute the
  message only to what the diff contains. Tag beads: end commit subjects
  with `(gently-r22)`.
- **Spec staging gotcha:** the spec-drift-gate compares worktree
  `openspec/` against the INDEX — the deployed mirror must be staged in
  the same commit as the corpus change. An aborted `&&` chain leaves
  earlier adds staged; check `git status` before committing.
- **Oracle pin:** all oracle observations run under
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib` (the runner enforces it).
  Pin-contract gaps (commit unverifiable from a dist) are known —
  gently-ikm/liz territory, not yours.
- **beads:** issues live in `.beads/issues.jsonl`; leave close to the
  orchestrator.
- If a probe reveals a finding outside graph_model scope, do NOT amend
  other specs — note it in your report for the orchestrator to file.

## Follow-up threshold

If any finding class cannot be grounded in a probe (oracle unreachable,
behavior nondeterministic across seeds), STOP that class, record the raw
observations in your report, and leave the row unamended with a note — do
not guess. Implementation gaps go to the orchestrator to file, not this bead.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `spk lint specs/` → <result>
- `just gates` → <result>
- `cargo test --workspace` → <result>
- `ah check` → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">
