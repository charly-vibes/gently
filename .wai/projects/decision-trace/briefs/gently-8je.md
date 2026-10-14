# Subagent brief: gently-8je — decision-trace E2 oracle dump pilot

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core`; the experiment tooling is Perl). An
orchestrator has claimed ticket `gently-8je` and is hands-off: you own the
implementation end-to-end (code, run, measurement, commit). Do NOT close the
ticket and do NOT push — the orchestrator owns ship.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=decision-trace`
- `bd show gently-8je`
- Read `.wai/projects/decision-trace/designs/2026-10-14-design-decision-trace-layer-oracle-adjudicati.md` — sections D4 and D7 (E2 row + decision rule).
- Read `tests/repro/probes.pl` header comment + the mode dispatch at the bottom (`claims` mode) and ONE claim probe body end-to-end (e.g. `pod-anon-group`) to learn the probe shape.
- Oracle toolchain: perl + pinned checkout `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib`. It EXISTS on this machine; never let the script hang when absent (degrade with a printed reason + exit 2).

## The five E2 fixtures (claim probes already in probes.pl)

1. `fan-layout` (gently-4ga)
2. `pod-anon-group` (gently-58m)
3. `dot-bare-scope-in-named` (gently-fb1)
4. `so-flowchart-size` (gently-awd)
5. `so-start-port-index` (gently-byq)

GOTCHA (from beads notes): `pod_probe_body` merges all example lines into ONE
graph → hash-dependent parallel edges. Each dump run must parse the probe's
example as its OWN graph, exactly like the existing claim probes do.

## What to build

A `dump` mode in `tests/repro/probes.pl`:

- Usage: `perl tests/repro/probes.pl dump [claim...]` — no args = the five E2 claims above.
- For each claim: install a wrapper around the oracle's `_prepare_layout` (symbol-table aliasing after the module is loaded; find the package it actually lives in by grepping `/var/tmp/ge0.69/Graph-Easy-0.69/lib` — do NOT assume; only `_prepare_layout` may be wrapped, nothing else), then run the probe's as_ascii render.
- The wrapper appends a COMPACT summary per invocation to `tests/repro/dumps/<claim>.dump.txt`:
  - invocation count,
  - per-node placement: name, x, y, width, height (as decided by the layout),
  - per-band structure: rank → member nodes, and edge count crossing it.
  - NO raw Data::Dumper of graph objects; NO full edge-cell matrices. If a summary line would exceed the budget, truncate with a `...` marker line.
- Create `tests/repro/dumps/README.md` recording: per-dump `wc -c`, TOTAL bytes, the PASS/KILL verdict, the exact wrapper location (package + file), and the elapsed wall-clock for the whole run.

## Pre-registered meter (do not deviate)

- **PASS** iff total `wc -c` over the 5 dumps < 51200 bytes AND elapsed < 1 day of work (you report your own elapsed honestly).
- **KILL** iff total > 512000 bytes, or hooking required >5 subs. If you hit a kill threshold, STOP expanding the instrumentation, record the honest numbers in README.md — a clean kill is a VALID outcome (it settles design D4's storage policy: dumps go on-demand, not committed).
- Anti-hack guard: the dumps must be captured from real oracle execution. A hand-written or reconstructed dump invalidates the run. If `_prepare_layout` cannot be wrapped within the ≤1-sub budget, record that as the finding — do not fake it.

## Out of scope

- No changes to `crates/` (no Rust code, no specs, no openspec, no specodelic artifacts).
- No new claim probes; no changes to existing probe bodies or `.observed` files.
- Do not implement the trace layer, trace-diff, or E1/E3/E4 — this is the dump pilot only.
- Leave pre-existing worktree dirt alone (`tests/repro/claims/so-flowchart-size.observed`, `.whisper/`); commit ONLY your files.

## Red/green sequence (experiment protocol, not rust-TDD)

1. RED: `perl tests/repro/probes.pl dump` fails with unknown-mode usage message; `tests/repro/dumps/` absent.
2. GREEN: mode emits 5 dumps; `wc -c` totals recorded in README.md with the verdict.
3. Regression check: `perl tests/repro/probes.pl claims` still lists all probes unchanged; run one existing claim (`pod-anon-group`) and confirm its `.observed` bytes are unchanged (`git status` must show no modification to claims/).
4. Commit: `git add tests/repro/probes.pl tests/repro/dumps/` → single commit `feat(repro): oracle _prepare_layout dump pilot for the five decision-trace E2 fixtures (gently-8je)` with the wc -c total + verdict in the body. Note: pre-commit hooks run — if the spec-drift-gate complains, it is misfiring on unrelated dirt; commit with `--no-verify` ONLY if you verified your own files pass (this is the documented pattern).

## Follow-up threshold

If the wrapper needs restructuring probes.pl beyond a new mode + one helper, or exceeds 30 minutes of work: stop, commit what works, and leave a precise hand-off comment — do not expand scope.
