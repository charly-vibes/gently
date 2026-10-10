---
date: 2026-10-09
project: graph-easy-port
phase: research
---

# Session Handoff

## What Was Done

<!-- Summary of completed work -->

## Key Decisions

<!-- Decisions made and rationale -->

## Gotchas & Surprises

<!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

## What Took Longer Than Expected

<!-- Steps that needed multiple attempts. Commands that failed before the right one. -->

## Open Questions

<!-- Unresolved questions -->

## Next Steps

<!-- Prioritized list of what to do next -->

## Context

### open_issues

```
○ gently-ef5 P1 spec: implement cli
○ gently-0kg P2 round-trip invariants fail in the oracle: as_graphviz always directed, named groups round-trip as cluster0..N, group-endpoint edges crash as_graphviz
○ gently-b4v P2 spec: implement ge.graphviz_render
○ gently-cbb P2 verify gently implementation claim: existing parser has no sharp/anonymous-node/group handling
○ gently-dcp P2 renderers (html/ascii/graphviz/txt): report findings 8a-c need verification against the implementation
○ gently-4ln P3 spec: implement ge.perf
○ gently-89d P3 layout spec: c3/p3 flow-direction claim unsatisfiable; c1 contradicted by oracle alarm timeout; c4 false at 50 nodes (silently drops edges)
○ gently-a83 P3 [bug] text_parser: store-layer %XX entity decode and control-char filtering missing from attribute values
○ gently-ikm P3 oracle determinism: PERL_HASH_SEED, Perl version, and PERL5LIB source-pin absent from c1/c3 and justfile/README; 24/149 fixtures hash-dependent; 4 fail under every seed
○ gently-k4u P3 perf spec: c2 1000-node/2s and c5 near-linear budgets contradict the oracle (9.6s at 200 nodes, superlinear growth)
○ gently-liz P3 smaller spec/tooling errors: ge.cli missing reading->cli_failed transition; ge.oracle c4 names oracle-record as the fix for a drifted oracle; justfile refs nonexistent oracle-pin and invalid cpanm syntax; pin checks version only not commit
○ gently-q1x P3 spec: implement ge.oracle

--------------------------------------------------------------------------------
Total: 12 issues (12 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```

