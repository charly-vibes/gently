# gently-8je — E2 oracle `_prepare_layout` dump pilot (decision-trace)

Fresh run: `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl dump`
(from repo root; each dump regenerates deterministically under the pinned
oracle with `PERL_HASH_SEED` swept over 0 and 1 per fixture).

## Sizes (`wc -c`)

## Sizes (gently-h1b corpus sweep, `wc -c`)

| dump | bytes | source |
|---|---|---|
| bg-chain-continue.dump.txt | 672 | admission |
| bg-dotted-style.dump.txt | 592 | admission |
| chain.dump.txt | 596 | admission |
| dot-bare-scope-in-named.dump.txt | 996 | E2 pilot |
| fan-layout.dump.txt | 1503 | E2 pilot |
| mixed_isolated.dump.txt | 725 | admission |
| parallel.dump.txt | 657 | admission |
| pod-anon-group.dump.txt | 859 | E2 pilot |
| pod-bold-label.dump.txt | 579 | admission |
| pod-bold-style.dump.txt | 579 | admission |
| pod-inline-styles.dump.txt | 640 | admission |
| pod-one-node.dump.txt | 553 | admission |
| port-chain-right.dump.txt | 695 | admission |
| port-end-right.dump.txt | 661 | admission |
| port-skip-right.dump.txt | 694 | admission |
| port-start-left.dump.txt | 664 | admission |
| port-start-right.dump.txt | 663 | admission |
| selfloop.dump.txt | 625 | admission |
| size-attr.dump.txt | 1306 | admission |
| so-flowchart-size.dump.txt | 876 | E2 pilot |
| so-start-port-index.dump.txt | 896 | E2 pilot |
| tracer.dump.txt | 565 | admission |
| txt-isolated.dump.txt | 691 | admission |
| txt-shared-target.dump.txt | 608 | admission |
| **TOTAL (24 dumps)** | **17895** | |

Excluded (tier-2 guard, admission.tsv verdict `hash-dependent`):
`exclude: txt-diamond (admission.tsv: hash-dependent — tier-2
hash-dependent class not admitted to the dump corpus; ge.oracle c7)`. No
`skip:` lines were needed — every stable admission class has an extractable
fixture input.

## Meter (pre-registered in the brief)

- **PASS** iff total < 51200 bytes AND elapsed < 1 day of work.
- **KILL** iff total > 512000 bytes, or hooking required >5 subs.

**Verdict: PASS** — 5130 bytes at the pilot (10% of the PASS ceiling), ~1 s
wall-clock for the five-fixture pilot, and exactly **1 sub wrapped**
(`_prepare_layout` only, per the design constraint).

## gently-h1b corpus extension (2026-10-14)

The no-arg dump sweep over the full tier-1 corpus (24 dumps = five E2 pilot
fixtures + 19 stable admission classes; `txt-diamond` excluded by its
admission row) produced 17895 bytes and ran in **3.9 s** wall-clock. The
five E2 pilot dumps remained byte-identical after the extension (verified
against the committed pilot bytes via `git show` + `cmp`); admission-class
dumps reuse the identical dump format, child, and seed sweep, reading the
tier-1 fixture file as the single case. All 19 stable classes dumped —
zero skips; the only named non-admission is the `txt-diamond` exclude line.
Explicit `dump txt-diamond` still names the exclusion on stderr and exits 2
(the admission-probe meter).
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
