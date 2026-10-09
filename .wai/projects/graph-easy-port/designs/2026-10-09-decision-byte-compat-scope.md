# Decision: byte-compat scope contract (gently-ghh)

**Date:** 2026-10-09 · **Ticket:** gently-ghh (P0) · **Status:** DECIDED — encoded in `specs/ge-oracle.md` c7/p7
**Gates:** gently-k4u (perf), gently-89d (layout), gently-r22 (graph_model), gently-bzx/6j0 (text_parser), gently-13f (dot_parser), gently-0kg (round-trips), gently-dcp (renderers)

## Context

The upstream-review report's findings 1–5 contradict byte-compatibility with
the Perl oracle as the *sole* comparison target, and specific probes show the
oracle cannot serve as the comparison target for whole claim classes:

- **Hash-dependence (gently-ikm):** 24/149 corpus fixtures produce
  oracle output that varies with Perl's hash randomization even under a
  pinned `PERL_HASH_SEED`; 4 fail under *every* seed. The pin chain
  (seed, Perl version, PERL5LIB source) is not enforced today.
- **Oracle size envelope (gently-k4u):** the oracle needs 9.6 s at 200
  nodes with superlinear growth — a 1000-node corpus fixture is
  unrecordable in practice, so perf c2's "1000-node within 2 s" budget
  cannot be gated on oracle byte-equality.
- **Round-trip gaps (gently-0kg):** upstream's own round-trip is broken
  for classes its renderers emit: `as_graphviz` always directs edges,
  named groups round-trip as `cluster0..N`, group-endpoint edges crash
  `as_graphviz`. Byte-compat on round-trip fidelity for these classes is
  unsatisfiable by construction.

## Decision

Three tiers replace "byte-compat with the oracle" as the universal target.

### Tier 1 — byte-compat (oracle-gated)

A claim may be verified byte-wise against recorded oracle outputs **only**
for fixture classes that pass both admission probes:

1. **Hash-stability:** the pinned oracle renders the input identically
   across repeated runs under the pinned seed at the pinned revision
   (`Graph::Easy v0.69 @ ededa3d7…`).
2. **Envelope:** the pinned oracle records the input within 10 s wall
   time (admits the 200-node class; excludes the 1000-node class given
   measured superlinear growth).

The admissible set is defined by the probes, not by a hardcoded count —
the report's "137/149" is an instance, not the contract. `just
oracle-record` records companions for every corpus input regardless of
tier (recording is cheap); tier membership only gates which *claims* may
be byte-gated.

### Tier 2 — oracle-scoped-off (no byte-compat claims)

Hash-randomized inputs, beyond-envelope sizes, and round-trip fidelity
the oracle itself lacks (as_graphviz direction, group↔cluster renames,
group-endpoint crashes). No spec may state byte-equality with the oracle
for these classes. Existing such claims are re-derived against upstream
and reworded (downstream tickets).

### Tier 3 — raw-scaling (structural verification)

Claims outside Tier 1 scope are verified against gently's *own*
invariants, never the oracle: determinism (pure-function properties),
structural round-trip within gently's model (parse(render(x)) ≈ x where
gently itself is lossless), and scaling budgets measured on gently alone.

## Claim-wording recipe (so a false claim fails a check)

1. Every byte-compat claim names its **fixture class + admission probe**
   ("hash-stable, in-envelope corpus inputs") — no unqualified
   "byte-identical with the oracle" anywhere in `specs/`.
2. Perf budgets: large-graph budgets (k4u) apply to gently alone at the
   1000-node class in Tier 3; byte-compat verification happens only
   within the corpus envelope. Budget numbers are re-derived from
   measured gently scaling, not aspirational.
3. Layout claims (89d): flow-direction claims re-derived against actual
   upstream behavior; graphs that trip the oracle alarm are excluded
   from the comparison set rather than asserted renderable.
4. Round-trip claims (0kg, r22): worded per-format and per-graph-class
   exactly as upstream behaves (directed-only, cluster renames), each
   with a false-claim test.
5. `gently-mwo` owns the falsifiable probe harness that machine-checks
   tier-1 admission (stability + envelope) — it is the enforcement arm
   of this contract.

## Encoding landed now

`specs/ge-oracle.md`: **c7** (invariant, scope contract) + **p7**
(deriving property, machine-checkable classification). Downstream
amendments trace their scoping language to `[[ge.oracle.c7]]`.

## Consequences

- Closing ghh unblocks the amendment tickets above; implementation
  capability beads were never gated (whisper 2026-10-09 13:33).
- `ge-oracle.c1` pin-chain hardening (seed, Perl version, PERL5LIB
  source) stays with gently-ikm; c7 references the pinned chain without
  redefining it.
- If the oracle pin is later moved past 0.69, tier-1 membership is
  recomputed by the probes — the contract does not change.