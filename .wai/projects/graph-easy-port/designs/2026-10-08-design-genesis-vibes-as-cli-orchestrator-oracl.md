# Design: genesis-vibes as CLI orchestrator, oracle + perf gating

## Decision

1. **gently's CLI is built on genesis-vibes (crate >= 0.12)** — the
   charly-vibes shared foundation. Dispatch via `Guide`, output via
   `Output::emit`, flags via `CliVerbosity`/`CliFormat`, health checks via
   `DoctorRunner` (`gently doctor`), unknown-name suggestions via
   `suggestions`, scratch fixtures via `fixture`, init registration via
   `discovery` (writes `.genesis/tools.toml`).
   Rationale: suite consistency (envelopes, doctor, self-healing identical
   across tools); fixes land once. Spec'd as ge.cli c5-c8.
2. **Oracle gating**: the original Perl Graph::Easy is the compatibility
   oracle, pinned to v0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499.
   Recorded fixtures under tests/fixtures/graph-easy/ carry pin headers;
   `just oracle-verify` compares byte-for-byte; `just oracle-record`
   regenerates deliberately (pin change required for re-record).
   Spec'd as ge.oracle; complements ge.txt_render c4.
3. **Performance budgets** (ge.perf): 50 ms cold-start for 20n/30e;
   2 s + 256 MB RSS for 1000n/2000e all formats; O(n+e+cells) memory;
   typed budget-violation diagnostics via `just perf-check`.
   Compatibility always wins over speed; budgets are regression floors.

## Alternatives considered

- Free-floating CLI (no genesis): rejected — re-implements suite plumbing,
  drifts from dont/wai/specodelic conventions (the exact problem genesis
  exists to solve).
- Property-based compatibility only (no recorded oracle fixtures): rejected
  — upstream output is quirky; only recorded bytes prove compatibility.
- Relative perf targets only (no absolute budgets): rejected — absolute
  floors are machine-checkable in CI and catch accidental algorithmic
  regressions.

## Consequences

- Two new specs: specs/ge-oracle.md, specs/ge-perf.md; ge.cli extended
  (c5-c8 + diagnosing state). Corpus now 12 specs, 0 findings/warnings.
- justfile gained oracle-record / oracle-verify / perf-check recipes that
  fail honestly with spec pointers until the implementation phase starts.
- oracle-record requires perl + Graph::Easy==0.69 locally; CI needs the
  same or recorded fixtures alone (verify runs against committed bytes,
  so CI can stay perl-free for oracle-verify).

