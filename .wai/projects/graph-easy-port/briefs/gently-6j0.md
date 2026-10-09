# Subagent brief: gently-6j0 — text_parser spec re-derivation (c1/c6/c7/c9 + operator rules)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-6j0` and is hands-off: you own
the work end-to-end. The orchestrator will verify with real gates — never
claim work the diff doesn't contain.

## Orientation (do this first)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-6j0` — the bead description lists findings a)–f); this
  brief operationalizes them.
- Read `specs/ge-text_parser.md` IN FULL — c1, c6, c7, c9 and their
  properties p1/p6/p7/p9 are the rows you will amend.
- Read `tests/repro/README.md` — the probe harness (closed gently-mwo) is
  your evidence source. The pin recipe and known hazards are there.
- `wai search "text_parser"` — accumulated patterns.
- Precedent: the gently-ghh amendment (commit 38a7803) is the workflow
  template — probe evidence → amend corpus row → redeploy → same-commit
  staging → gates.

## Desired outcome

`specs/ge-text_parser.md` matches the pinned oracle's actual behavior
(Graph::Easy v0.69 @ `ededa3d787ad89ac532c578c06390e8a7b270499`) on the
five finding classes below, with every amendment grounded in committed
probe evidence. After this bead, implementation ticket gently-bzx can be
briefed from the amended spec without re-litigating behavior.

## Finding classes to re-derive (from the bead + probes)

a) **c7 — in-string `#`:** `{ label: x # y }` is a parse error in the
   oracle; `#` must be escaped as `\#` (hex colours are a special case).
   Evidence: probe `sharp-escape` (already recorded:
   `tests/repro/claims/sharp-escape.observed`). Amend c7/p7 so the rule is
   the oracle's actual rule (escape requirement, comment-line scope).
b) **c9 — operator families:** the spec calls `..-..-..>` (truncated
   repeat) and `.-` (single-char arrow-less) parse errors, but the oracle
   accepts both. Re-derive the real repetition/arrow-less rule by probing
   operator patterns against the oracle.
c) **c6 — group colon:** in the oracle, `( G: ... )` makes the colon part
   of the group NAME. Also: a node can belong to only ONE group, and
   nested groups do not include inner nodes in the outer group. Amend
   c6/p6 accordingly.
d) **c1 — anonymous nodes** are named `#N` by the oracle (see
   `tests/repro/claims/node-unnamed.observed`, serving gently-r22) — check
   whether c1's "cannot be referenced again" wording survives probing
   (`node('#1')` finds them), amend wording if needed.
e) **omissions:** bidirectional operators and the no-left-only-edges rule
   are absent from the spec — probe them and add rows (or extend c2) as
   the evidence dictates.

## What to build

1. **New probes where missing.** `sharp-escape` exists; b/c/e findings have
   no probes yet. Add probe cases to `tests/repro/probes.pl` (or new claim
   probes) following the existing probe style, record
   `tests/repro/claims/<name>.observed` files, and extend the probe table
   in `tests/repro/README.md`. Every spec claim you change must cite
   committed observed bytes — prose is not evidence.
2. **Amend the corpus spec** `specs/ge-text_parser.md`: rows c1/c6/c7/c9,
   their properties (p1/p6/p7/p9 predicates stay consistent with the new
   constraints), and any added rows. Respect the specodelic lint rules
   (guards non-null, unique ids, EARS statement, coverage/derivation).
3. **Redeploy the mirror:** run `python3 tools/deploy_specs.py` so
   `openspec/specs/ge-text_parser/spec.md` matches, and STAGE both the
   corpus spec and the deployed mirror in the SAME commit — the
   spec-drift-gate compares worktree `openspec/` against the INDEX.
   Existing `.espectacular/ge-text_parser/p1–p9.toml` stubs stay as-is.
4. **Update `tests/repro/README.md`** probe table with the new probes and
   the beads they serve.

## Red → green → refactor shape

- **RED:** for each finding, run the probe and capture the delta between
  the current spec row's claim and the observed oracle bytes. Commit the
  probe scripts + observed evidence first (evidence before amendment).
- **GREEN:** amend the spec rows to match observed reality; `spk lint
  specs/` clean; redeployed mirror staged.
- **REFACTOR:** tighten prose, ensure p-predicates read as checkable
  statements, README table coherent.

## Hard scope guard

- Allowed files: `specs/ge-text_parser.md`,
  `openspec/specs/ge-text_parser/spec.md` (via deploy script only),
  `tests/repro/probes.pl`, `tests/repro/claims/*`,
  `tests/repro/README.md`.
- Never edit: `crates/**` (implementation is gently-bzx), other
  `specs/*.md`, `.espectacular/**`, `openspec/specs/**` other than the
  ge-text_parser mirror, `.wai/resources/**`, this template.
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
  with `(gently-6j0)`.
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
- If a probe reveals a finding outside text_parser scope, do NOT amend
  other specs — note it in your report for the orchestrator to file.

## Follow-up threshold

If any finding class cannot be grounded in a probe (oracle unreachable,
behavior nondeterministic across seeds), STOP that class, record the raw
observations in your report, and leave the row unamended with a note — do
not guess. Implementation gaps go to gently-bzx, not this bead.