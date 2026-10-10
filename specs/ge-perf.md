---
id: ge.perf
kind: intent
statement: "WHEN gently processes any supported input class THE performance SHALL stay within the suite's latency and memory budgets without sacrificing byte-compatibility within the oracle-verifiable envelope (specs/ge-oracle.md c7)."
---

# perf

The performance capability: concrete, machine-checkable budgets for the
pipeline stages ([[ge.text_parser]], [[ge.dot_parser]], [[ge.layout]], and
the renderers) plus the CLI round-trip ([[cli]]). Byte-compatibility
([[ge.oracle]]) always wins over speed, but only within the oracle-verifiable
envelope (specs/ge-oracle.md c7): the pinned oracle grows superlinearly and is
alarm-killed at 200 nodes (tests/repro/claims/size-envelope.observed — 0.597 s
at 50 nodes/141 edges, 2.744 s at 100 nodes/291 edges, no output at 200), so
large-graph budgets are gently-only budgets in the raw-scaling tier, verified
structurally with no oracle byte comparison. Budgets are regression floors,
not aspirations. The `just perf-check` recipe runs the budget gates.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | Cold-start round-trip — parse, layout, and ascii-render a 20-node/30-edge graph — completes within 50 ms in release builds. | [[ge.perf]] | |
| c2 | invariant | A 1,000-node/2,000-edge graph lays out and renders in every supported format within 2 s wall time and 256 MB peak RSS in release builds. Re-derived (gently-k4u): this is a gently-only budget (Tier 3, specs/ge-oracle.md c7) — the pinned oracle cannot produce comparison fixtures at this class (tests/repro/claims/size-envelope.observed: 2.744 s at 100 nodes/291 edges with superlinear growth, alarm timeout with no output at 200 nodes), so the budget is verified structurally against gently alone with no oracle byte comparison; measured headroom: 13 ms wall and ~4.4 MB peak RSS for a 1,000-node/2,991-edge fixture in release builds. | [[ge.perf]] | [[ge.perf.c4]] |
| c3 | invariant | Peak memory stays O(nodes + edges + grid cells): the process holds no caches that grow across pipeline stages beyond one intermediate graph, one laid-out grid, and one rendered buffer. | [[ge.perf]] | |
| c4 | effect | A budget violation produces a typed diagnostic naming the stage, the budget, and the measured value, with a non-zero exit when `just perf-check` runs it. | [[ge.perf]] | |
| c5 | advisory | Beyond the budgeted input classes, gently's runtime scales near-linearly in nodes plus edge-crossing work; regressions in scaling are reported, never silently accepted. Re-derived (gently-k4u): the near-linear expectation holds for gently only — upstream grows superlinearly and dies at 200 nodes (tests/repro/claims/size-envelope.observed), so upstream behavior cannot anchor any scaling claim; scaling is measured on gently alone within the raw-scaling tier (specs/ge-oracle.md c7), where measured wall time stays near-linear (13 ms at 1,000 nodes, 55 ms at 4,000 nodes for chain-class inputs). | [[ge.perf]] | |

## Model

### States

- `idle`
- `measured`
- `budget_exceeded`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | idle | measured | [[ge.perf.c1]] |
| t2 | idle | budget_exceeded | [[ge.perf.c1]] |
| t3 | measured | idle | [[ge.perf.c3]] |

### Failure terminal

`budget_exceeded` is a failure terminal: it emits [[ge.perf.c4]].

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.perf.c1]] | repeated cold-start runs over budget-class inputs | median wall time stays under 50 ms with no run above 3x the median budget |
| p2 | unit | [[ge.perf.c2]] | large graphs around the 1,000-node/2,000-edge class, rendered by gently alone (Tier 3) | every supported format renders within the wall-time and RSS budgets, verified structurally against gently with no oracle comparison |
| p3 | unit | [[ge.perf.c3]] | stage-by-stage memory profiling over growing inputs | peak allocation matches the O(nodes + edges + cells) profile; no cross-stage cache growth |
| p4 | unit | [[ge.perf.c4]] | deliberately throttled stages exceeding each budget | the diagnostic names stage, budget, and measured value, and the exit code is non-zero |
| p5 | unit | [[ge.perf.c5]] | gently-only input sizes sweeping two orders of magnitude beyond the budget classes (raw-scaling tier) | measured gently scaling stays near-linear; deviations beyond the bound are reported |
