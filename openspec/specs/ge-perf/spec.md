---
id: spec
kind: intent
statement: "WHEN gently processes any supported input class THE performance SHALL stay within the suite's latency and memory budgets without sacrificing byte-compatibility with the oracle."
---

# perf

The performance capability: concrete, machine-checkable budgets for the
pipeline stages ([[ge.text_parser]], [[ge.dot_parser]], [[ge.layout]], and
the renderers) plus the CLI round-trip ([[ge.cli]]). Byte-compatibility
([[ge.oracle]]) always wins over speed; budgets are regression floors,
not aspirations. The `just perf-check` recipe runs the budget gates.

## Purpose

The performance capability: concrete, machine-checkable budgets for the
pipeline stages ([[ge.text_parser]], [[ge.dot_parser]], [[ge.layout]], and
the renderers) plus the CLI round-trip ([[ge.cli]]). Byte-compatibility
([[ge.oracle]]) always wins over speed; budgets are regression floors,
not aspirations. The `just perf-check` recipe runs the budget gates.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | Cold-start round-trip — parse, layout, and ascii-render a 20-node/30-edge graph — completes within 50 ms in release builds. | [[spec]] |  |
| c2 | invariant | A 1,000-node/2,000-edge graph lays out and renders in every supported format within 2 s wall time and 256 MB peak RSS in release builds. | [[spec]] | [[spec.c4]] |
| c3 | invariant | Peak memory stays O(nodes + edges + grid cells): the process holds no caches that grow across pipeline stages beyond one intermediate graph, one laid-out grid, and one rendered buffer. | [[spec]] |  |
| c4 | effect | A budget violation produces a typed diagnostic naming the stage, the budget, and the measured value, with a non-zero exit when `just perf-check` runs it. | [[spec]] |  |
| c5 | advisory | Beyond the budgeted input classes, runtime scales near-linearly in nodes plus edge-crossing work; regressions in scaling are reported, never silently accepted. | [[spec]] |  |

## Model

### States

- `idle`
- `measured`
- `budget_exceeded`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | idle | measured | [[spec.c1]] |
| t2 | idle | budget_exceeded | [[spec.c1]] |
| t3 | measured | idle | [[spec.c3]] |

### Failure terminal

`budget_exceeded` is a failure terminal: it emits [[spec.c4]].

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[spec.c1]] | repeated cold-start runs over budget-class inputs | median wall time stays under 50 ms with no run above 3x the median budget |
| p2 | unit | [[spec.c2]] | large graphs around the 1,000-node/2,000-edge class | every supported format renders within the wall-time and RSS budgets |
| p3 | unit | [[spec.c3]] | stage-by-stage memory profiling over growing inputs | peak allocation matches the O(nodes + edges + cells) profile; no cross-stage cache growth |
| p4 | unit | [[spec.c4]] | deliberately throttled stages exceeding each budget | the diagnostic names stage, budget, and measured value, and the exit code is non-zero |
| p5 | unit | [[spec.c5]] | input sizes sweeping two orders of magnitude beyond the budget classes | measured scaling stays near-linear; deviations beyond the bound are reported |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** repeated cold-start runs over budget-class inputs
- **THEN** median wall time stays under 50 ms with no run above 3x the median budget
- **VERIFIES** [[spec.p1]]

#### Scenario: p2

- **WHEN** large graphs around the 1,000-node/2,000-edge class
- **THEN** every supported format renders within the wall-time and RSS budgets
- **VERIFIES** [[spec.p2]]

#### Scenario: p3

- **WHEN** stage-by-stage memory profiling over growing inputs
- **THEN** peak allocation matches the O(nodes + edges + cells) profile; no cross-stage cache growth
- **VERIFIES** [[spec.p3]]

#### Scenario: p4

- **WHEN** deliberately throttled stages exceeding each budget
- **THEN** the diagnostic names stage, budget, and measured value, and the exit code is non-zero
- **VERIFIES** [[spec.p4]]

#### Scenario: p5

- **WHEN** input sizes sweeping two orders of magnitude beyond the budget classes
- **THEN** measured scaling stays near-linear; deviations beyond the bound are reported
- **VERIFIES** [[spec.p5]]

