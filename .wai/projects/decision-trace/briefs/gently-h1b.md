# Subagent brief: gently-h1b — oracle dump mode: tier-1 corpus + tier-2 guard

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-h1b` and is hands-off: you own
the work end-to-end (implementation, runs, commits). Do NOT push and do
NOT `bd close` — the orchestrator owns ship.

## TIME BUDGET (binding): 30 minutes.

## Orientation (max 10 tool calls)

- `export WAI_PROJECT=decision-trace`
- `bd show gently-h1b` — re-scoped: tasks 3.1 + 3.3 ONLY (3.2 is DONE —
  the five E2 dumps are committed under `tests/repro/dumps/` via
  gently-8je, c664876).
- `tests/repro/dumps/README.md` — the E2 pilot's meter + method.
- `tests/repro/probes.pl` — `dump_cases_for` (line ~2039), the
  `$DUMP_CHILD_SRC` child (wraps `_prepare_layout`, budget/truncation,
  per-band summaries), and the `dump` mode entry in the usage block
  (line ~46). The pilot regenerates per (claim, seed) with
  `PERL_HASH_SEED` swept over 0 and 1.
- `tests/repro/admission.tsv` — the admission manifest: 19 `stable`
  classes + 1 `hash-dependent` (`txt-diamond`). This is the tier-1
  corpus per ge.oracle c7.
- `specs/decision-trace.md` c4/c5 — the constraints you satisfy: dumps
  carry pin header + explicit hash-seed field + per-stage observability;
  tier-2 classes stay excluded BY ADMISSION-PROBE NAMING (not prose).
- Dump-mode invocation: `PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib perl tests/repro/probes.pl dump` (pinned oracle at /var/tmp/ge0.69 — check it exists before running).

## Desired outcome

1. **Corpus dump mode** (3.1): `dump` with no args now dumps ALL admission
   `stable` classes (the five E2 fixtures plus the other stable
   admission.tsv rows that have a claim probe with an extractable input;
   any stable class whose probe input is not extractable gets a named
   `skip:` line in the summary, not a silent omission). Per-fixture dumps
   land in `tests/repro/dumps/<claim>.dump.txt` (same format as the E2
   five: pin header + seed field + stage-observability marks — the E2
   dumps already define this format; keep it byte-compatible).
2. **Tier-2 guard** (3.3): a class listed `hash-dependent` in
   admission.tsv (today: `txt-diamond`) is EXCLUDED from the committed
   dump corpus — dump mode prints a named exclusion line citing the
   admission row and exits cleanly (exclusion is data-driven from
   admission.tsv, not a hardcoded list).
3. **Committed corpus**: commit the new dump files + updated
   `tests/repro/dumps/README.md` (sizes table updated; note the pilot →
   corpus extension).
4. The E2 five dumps must remain BYTE-IDENTICAL after the extension
   (regenerate them and diff — if the format changed at all, STOP and
   report; do not rewrite the committed pilots).

## Commit protocol

- Commit 1 (feat): probes.pl extension + new dumps + README update.
  `feat(repro): corpus-wide oracle dump mode with tier-2 admission guard (gently-h1b)`.
- Keep commits file-scoped; .whisper/.wai churn is the orchestrator's.

## Gates

- `perl tests/repro/probes.pl dump` (pinned oracle) exits 0, produces
  byte-identical E2 five + new dumps for the stable corpus classes.
- The dump run must stay fast (the E2 run was ~1 s; budget: whole corpus
  sweep < 60 s — record actual).
- `just gates` green; `cargo test` count unchanged (no Rust changes in
  this slice; if just gates regenerates specodelic, rustfmt and confirm
  byte-identical to HEAD).

## Out of scope

- p2 (normalization/faithfulness contracts — that's e3_faithfulness,
  already landed), p4/p6 contracts (vhc/9nn).
- Any Rust source change. Spec amendments. Push. bd close.

## Follow-up threshold

If any stable class's probe input cannot be extracted by dump_cases_for
without restructuring the probe heredocs, skip it with a named `skip:`
line and list it in your report — do not refactor probes.pl's probe
bodies.

## Report back (≤200 tokens)

- commit sha, dump inventory (files + bytes total), exclusions emitted,
  E2-five byte-identity check result, dump sweep wall-clock, gates.
