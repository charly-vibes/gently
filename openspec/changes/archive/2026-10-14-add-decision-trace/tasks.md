# Tasks: add-decision-trace

## 1. Spec corpus first (spec-first convention)

- [x] 1.1 Write `specs/decision-trace.md` (specodelic four-layer format)
      covering requirements R1–R5 below; run `spk lint specs` to zero
      findings.
- [x] 1.2 Deploy the openspec mirror (`tools/deploy_specs.py`) and
      regenerate specodelic artifacts (TOMLs, `*_props.rs`, `*.check.json`);
      run `rustfmt --edition 2021 specodelic/*.rs` after every compile
      (specodelic#15). Stage spec + mirror + artifacts in the SAME commit
      as the corpus change (spec-drift-gate gotcha).
- [x] 1.3 Register the espectacular contract scenarios (`.espectacular/`
      TOMLs) for the new capability; `ah check --run-tests` clean.

## 2. Trace emission (D1 — harden the E1 prototype)

- [x] 2.1 Move `crates/gently-core/src/trace.rs` from env-gated prototype
      to the versioned schema: `schema_version` field, per-stage
      sub-vocabularies, `spec_id` on every event, hash seed recorded.
- [x] 2.2 Property: with `GENTLY_TRACE` unset, the full fixture corpus
      renders byte-identical (regression-pin the E1 result as a test).
- [x] 2.3 Red→green→refactor cycle per stage vocabulary as it lands
      (start: `layout.place`, `render.route` — the E1-validated stages).

## 3. Oracle dump mode (D4 — extend the E2 pilot)

- [x] 3.1 Extend the `dump` mode in `tests/repro/probes.pl` from the
      `_prepare_layout` pilot: pin header + explicit `PERL_HASH_SEED`
      field + per-stage observability marks on every dump.
- [x] 3.2 Commit the five E2 fixture dumps under `tests/repro/dumps/`
      (already committed by gently-8je — re-verify byte-stability after
      the schema lands; regenerate if the schema version bumps).
- [x] 3.3 Guard: tier-2 (hash-seed-dependent) fixture classes are
      excluded from the committed-dump corpus — probe-enforced, not
      prose.

## 4. Normalization table (D2 — gated on E3)

- [x] 4.1 Land E3 (gently-ugy) first: prove the pairwise faithfulness
      criterion is computable on real dumps without hooking every
      upstream sub. Do NOT write the table before E3 reports. DONE
      2026-10-14: computable, kill not met —
      crates/gently-core/tests/e3_faithfulness.rs + decision record.
- [x] 4.2 Commit the normalization table as a tested artifact in
      `tests/repro/` with the pairwise faithfulness test. DONE
      2026-10-14: tests/repro/trace/normalization-table.md (rows N1-N8,
      each enforced by a row_nN_* subtest).

## 5. trace-diff (D3 + D6)

- [x] 5.1 Promote `tests/repro/trace/e1.sh` worktree-diff driver into the
      trace-diff tool: report the minimal antichain over the decision
      dependency poset (never a linear "first").
- [x] 5.2 Byte-gate: trace-diff refuses to run on byte-identical fixtures
      (decision-only diffs downgrade to informational).
- [x] 5.3 Corpus-level property test (EXCL-001): every byte-divergent
      fixture with complete traces yields a nonempty minimal cut that
      trace-diff computes.

## 6. Adjudication registry (D5 — gated on E4)

- [x] 6.1 Land E4 (gently-btf, interactive session): run the
      pre-registered (b)-checklist on gently-58m. Do NOT build the
      registry before E4 reports. DONE 2026-10-14: class (b) reachable,
      all 5 items hold (item 3 = in-repo beads record gently-278, no
      upstream post per user); record in
      `.wai/projects/decision-trace/designs/2026-10-14-decision-e4-adjudication-dry-run-gently-58m.md`.
- [x] 6.2 If E4 reaches class (b): commit the registry (checklist +
      adjudication-record template). If E4 kills: record the kill in the
      design doc and ship the registry in a later change. DONE
      2026-10-14: tests/repro/adjudication-registry.md (checklist
      verbatim + record template + first entry: gently-58m (b)).

## 7. Gates

- [x] 7.1 `just gates` green (spk lint/compile/model-check, ah check,
      pretender, spec-drift-gate — spec+mirror+artifacts same-commit).
- [x] 7.2 `cargo test` full suite + `clippy --all-targets` clean.
- [x] 7.3 `just examples-verify` byte-identical.
- [x] 7.4 Update `docs/known-hard-cases.md` workflow row to reference
      trace-diff as the debugging entry point.
