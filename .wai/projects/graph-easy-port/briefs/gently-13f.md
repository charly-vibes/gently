# Subagent brief: gently-13f — dot_parser spec re-derivation (direction, subgraph→group, anonymous subgraph error, records/ports parsed)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-13f` and is hands-off: you own
the work end-to-end. The orchestrator will verify with real gates — never
claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-13f` — the bead description lists the findings; this
  brief operationalizes them.
- Read `specs/ge-dot_parser.md` IN FULL — c1–c5 and p1–p5 are the rows
  in play. The bead names "c2 (line 21)" and "c3 (line 22)" but the
  current file has c1=header-direction, c2=edge operators, c4=subgraph
  clusters, c5=unsupported-constructs — let the PROBES decide which rows
  each finding falsifies; the bead's line numbers are stale.
- Read `tests/repro/README.md` — the probe harness is your evidence
  source. Pin recipe and hazards are there.
- `wai search "dot_parser"` — accumulated patterns.
- Precedent: the gently-6j0 and gently-r22 amendments are the workflow
  template — probe evidence → amend corpus row → redeploy mirror →
  same-commit staging → gates. (r22 brief:
  `.wai/projects/graph-easy-port/briefs/gently-r22.md`; a fresh example
  of a completed re-derivation run.)

## Desired outcome

`specs/ge-dot_parser.md` matches the pinned oracle's actual behavior
(Graph::Easy v0.69 @ `ededa3d787ad89ac532c578c06390e8a7b270499`, module
`Graph::Easy::Parser::Graphviz`) on the finding classes below, with every
amendment grounded in committed probe evidence. After this bead,
implementation ticket gently-89p can be briefed from the amended spec
without re-litigating behavior.

## Finding classes to re-derive (from the bead)

a) **direction is set by the operator, not the header:** the spec says a
   `digraph` header → directed graph and `graph` header → undirected
   (c1). In the oracle the EDGE OPERATOR decides: `->` yields directed
   edges even inside a plain `graph {}` (and per upstream
   Graphviz-Parser code, `--` in a digraph yields undirected). Probe
   header×operator combinations, record what direction the resulting
   edges/model carry, and amend c1/c2 + p1/p2 to the oracle's actual
   rule.
b) **any named subgraph becomes a group, not only `cluster_*`:** amend
   c4/p4 — probe named subgraphs with arbitrary names (`subgraph foo {}`)
   and confirm group creation; also probe how the group name survives
   (verbatim? prefixed?).
c) **anonymous subgraph `{}` is an ERROR:** the spec says anonymous
   subgraphs "keep their nodes ungrouped" (p4) — the oracle rejects them.
   Probe `subgraph {}` / bare `{ }` and pin the exact error behavior
   (message shape, position).
d) **records and ports are PARSED, not rejected:** c5/p5 currently claim
   records, HTML labels, and ports are "out-of-scope constructs" that
   produce typed errors. In the oracle they are parsed (ports stripped/
   recorded, records handled per upstream `Graph::Easy::Parser::Graphviz`).
   Probe each construct, record the actual behavior, and amend c5/p5 —
   the "unsupported construct" rule may shrink to what ACTUALLY errors
   (probe to find what remains, if anything).

## What to build

1. **New probes where missing.** Add claim probes to
   `tests/repro/probes.pl` following the existing probe style (see the
   r22-run probes `anon-numbering`, `group-edge` for shape), record
   `tests/repro/claims/<name>.observed` files, and extend the probe
   table in `tests/repro/README.md` with the beads each probe serves.
   Every spec claim you change must cite committed observed bytes —
   prose is not evidence.
2. **Amend the corpus spec** `specs/ge-dot_parser.md`: whichever of
   c1/c2/c4/c5 the probes falsify, their properties p1/p2/p4/p5
   (predicates must stay consistent with the amended constraints), and
   any added rows. Respect the specodelic lint rules (guards non-null,
   unique ids, EARS statement, no conjoined/universal ids,
   coverage/derivation; `linter.terminal_states_emit` — if c5's error
   rule changes, `dot_rejected` still must emit exactly one file-owned
   effect constraint; `linter.guard_negation_total` — t2 is a failure
   transition, its guard must cite what its success sibling t1 cites).
3. **Redeploy the mirror:** run `python3 tools/deploy_specs.py` so
   `openspec/specs/ge-dot_parser/spec.md` matches, and STAGE both the
   corpus spec and the deployed mirror in the SAME commit — the
   spec-drift-gate compares worktree `openspec/` against the INDEX.
   Existing `.espectacular/ge-dot_parser/p1–p5.toml` stubs stay as-is.
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

- Allowed files: `specs/ge-dot_parser.md`,
  `openspec/specs/ge-dot_parser/spec.md` (via deploy script only),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `tests/repro/README.md`.
- Never edit: `crates/**` (implementation is gently-89p), other
  `specs/*.md`, `.espectacular/**`, `openspec/specs/**` other than the
  ge-dot_parser mirror, `.wai/**`, this brief.
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
  with `(gently-13f)`.
- **Spec staging gotcha:** the spec-drift-gate compares worktree
  `openspec/` against the INDEX — the deployed mirror must be staged in
  the same commit as the corpus change. An aborted `&&` chain leaves
  earlier adds staged; check `git status` before committing.
- **Oracle pin:** all oracle observations run under
  `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib` (the runner enforces it).
- **beads:** issues live in `.beads/issues.jsonl`; leave close to the
  orchestrator.
- If a probe reveals a finding outside dot_parser scope, do NOT amend
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
