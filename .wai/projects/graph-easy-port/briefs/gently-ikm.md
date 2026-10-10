# Subagent brief: gently-ikm — oracle determinism: pin Perl version, PERL_HASH_SEED, PERL5LIB contract in spec + tooling

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`.
An orchestrator has claimed ticket `gently-ikm` and is hands-off: you own
the re-derivation end-to-end. SPEC + TOOLING ticket — the deliverable is
amended spec rows + fixed tooling + recorded evidence.

## Orientation (max 8 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-ikm`
- Grounded findings (from the review report + probes):
  - `specs/ge-oracle.md` c1 (pin names version+commit only — missing:
    Perl version, `PERL_HASH_SEED`, `PERL5LIB`/source-checkout
    contract); c3 (byte-identical compare) — probe: 24 of 149 fixtures
    render differently run-to-run under default `PERL_HASH_SEED`
    (hash randomization); 4 fail under EVERY seed.
  - `tools/oracle.pl` lines 32-36: checks only `Graph::Easy->VERSION`,
    not the commit; recommends `cpanm Graph::Easy==$PIN_VERSION` —
    invalid cpanm syntax.
  - justfile ~line 55: same invalid syntax pattern; references
    `oracle-pin` which does not exist.
  - `tests/fixtures/graph-easy/README.md` lines 34-39: same items.
- The 4 under-every-seed failures are the hash-dependent fixture
  classes the ghh c7 scope contract already carves out
  (oracle-scoped-off tier) — cross-check `specs/ge-oracle.md` c7 and
  `.wai/projects/graph-easy-port/designs/2026-10-09-decision-byte-compat-scope.md`.

## What to do

1. **Verify the probe claims** by re-running a determinism probe:
   run the oracle recording under two different `PERL_HASH_SEED` values
   on a small fixture subset (use the existing oracle tooling — read
   `tools/oracle.pl` + justfile oracle recipes first) and confirm the
   24-hash-dependent / 4-every-seed split (or record corrected counts).
   Record the numbers in your report.
2. **Amend `specs/ge-oracle.md`** (rows c1, c3 — and c7 if the
   every-seed-failure tier needs wording): extend the pin contract to
   name (a) upstream version + commit, (b) the Perl version used for
   recording, (c) the `PERL_HASH_SEED` contract — recordings pin a
   seed (state the seed used for the current corpus, discoverable
   from the recorded evidence/tools), byte-compare requires the same
   seed, and (d) the `PERL5LIB` source-checkout contract (recordings
   run against the pinned source checkout at
   `/var/tmp/ge0.69/Graph-Easy-0.69/lib`, never a cpanm-installed
   copy). Deploy the openspec mirror in the SAME commit
   (`tools/deploy_specs.py`), recompile props via `just gates`.
3. **Fix tooling**:
   - `tools/oracle.pl`: check the pinned COMMIT (the source checkout
     path carries the revision — verify how; at minimum verify the
     version AND document the commit check), drop/replace the invalid
     cpanm recommendation.
   - justfile oracle recipes: valid syntax, reference the actual
     source-checkout path + `PERL_HASH_SEED`, remove the nonexistent
     `oracle-pin` reference.
   - `tests/fixtures/graph-easy/README.md`: same corrections.
4. **No corpus re-recording** — the corpus stays as recorded; the spec
   documents the contract that makes it reproducible. If your probe
   finds the recorded corpus contradicts the documented seed, STOP and
   record the discrepancy as a finding instead of re-recording.

## Hard scope guard

- Allowed: `specs/ge-oracle.md` + openspec mirror + regenerated props,
  `tools/oracle.pl`, `justfile`, `tests/fixtures/graph-easy/README.md`,
  `tests/repro/**` (probe script if needed).
- NEVER edit: other specs, `crates/**`, `.wai/**`, `.github/**`.
- Do NOT push, do NOT close the bead, do NOT run `wai close`.
- Commit subjects end with `(gently-ikm)`. File headers required for
  new/edited source files.

## Exact gates (all green before you report done)

```sh
spk lint specs/
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
ah check
ah check --run-tests
just gates
```

## Report format (end with this)

## Report

**Probe evidence** (seed-split numbers verified/corrected)

**Spec rows amended** (which rows)

**Tooling fixes** (files + what changed)

**Commits** (hash + one line)

**Gates run** — same six lines as impl briefs

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — brief, ge-oracle.md, tools/oracle.pl,
  justfile oracle section. Max 8 tool calls.
- **Minutes 5–12:** determinism probe re-run + numbers.
- **Minutes 12–22:** spec amendments + tooling fixes.
- **Minutes 22–27:** gates.
- **Minutes 27–30:** report.
Never re-read a file you have already read. A committed partial verdict
beats an uncommitted complete one.
