# gently-8je — E2 oracle `_prepare_layout` dump pilot (decision-trace)

Fresh run: `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl dump`
(from repo root; each dump regenerates deterministically under the pinned
oracle with `PERL_HASH_SEED` swept over 0 and 1 per fixture).

## Sizes (`wc -c`)

| dump | bytes |
|---|---|
| dot-bare-scope-in-named.dump.txt | 996 |
| fan-layout.dump.txt | 1503 |
| pod-anon-group.dump.txt | 859 |
| so-flowchart-size.dump.txt | 876 |
| so-start-port-index.dump.txt | 896 |
| **TOTAL** | **5130** |

## Meter (pre-registered in the brief)

- **PASS** iff total < 51200 bytes AND elapsed < 1 day of work.
- **KILL** iff total > 512000 bytes, or hooking required >5 subs.

**Verdict: PASS** — 5130 bytes (10% of the PASS ceiling), ~1 s wall-clock for
the whole dump run (regenerating all five fixtures under both seeds), and
exactly **1 sub wrapped** (`_prepare_layout` only, per the design constraint).
Dumps are byte-deterministic across regens: per-band node lines are sorted,
because `%{$graph->{cells}}` iteration order is hash-seed dependent (the same
upstream hash-order artifact behind the `so-flowchart-size.observed` dirt).

Elapsed wall-clock, honest report: total implementation + run + measurement
came in under the 25-minute subagent budget; the dump run itself is ~1 s.

## Wrapper location (exact)

- **Package:** `Graph::Easy` — NOT `Graph::Easy::Layout::Grid`. `Grid.pm`
  re-opens `package Graph::Easy;` near the top of the file, so the sub
  compiled at its line ~67 lives in the `Graph::Easy` symbol table
  (verified: `exists $Graph::Easy::{_prepare_layout}`).
- **Defining file:** `/var/tmp/ge0.69/Graph-Easy-0.69/lib/Graph/Easy/Layout/Grid.pm`
  (pinned oracle checkout, Graph::Easy v0.69 @ ededa3d).
- **Wrapper source:** the `THE WRAPPER (gently-8je)` block in the
  `DUMP_CHILD_SRC` heredoc inside `tests/repro/probes.pl` — symbol-table
  aliasing (`*{$pkg.'::_prepare_layout'} = sub { ... }`) installed after the
  module is loaded; calls the original, then records the compact per-band
  summary from the mutated cell state. Only `_prepare_layout` is wrapped.

## Dump content contract

Per fixture: a pin header, then per seed (0, 1) a per-invocation block:
node placements (name, x, y, w, h, spans), band structure (rank → member
nodes, edge-cell/group-cell counts crossing it). No raw `Data::Dumper` of
graph objects, no full edge-cell matrices; a `DUMP_BUDGET` guard truncates
with a `...` marker line if a summary would exceed budget (never hit on
these five fixtures).

## E3 faithfulness verdict (gently-ugy, 2026-10-14)

D2's pairwise criterion is **computable** on these dumps — implemented in
`crates/gently-core/tests/e3_faithfulness.rs` (pairwise soundness across
fixtures AND across seeds — fan-layout's genuine seed-dependent case0
placement is preserved by normalization, not collapsed — plus per-fixture
ordering invariance under the hash-order line permutation). Kill criterion
not met: `_prepare_layout` alone suffices, no further upstream hooks.
Decision record:
`.wai/projects/decision-trace/designs/2026-10-14-decision-e3-faithfulness-computable.md`.

## Interpretation for D4/D7

Five hard-case fixtures produce ~1 KB dumps each — two orders of magnitude
below the PASS ceiling even with seed sweeping. Storage policy data point:
committed companions are cheap for this corpus class; the 512 KB kill
threshold is nowhere near reached. Note the known oracle quirk surfaces
honestly in the dumps: pod-anon-group warns "Layouter could only place 2
nodes/0 edges" (the Berlin drop documented for E4 adjudication).
