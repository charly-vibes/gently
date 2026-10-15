# Contributing an example (maintainer workflow)

> **Maintainer reference.** Moved out of `examples/README.md` (which is
> synced into the user-facing book) so the published docs stay
> reader-focused. The admission rule it enforces is unchanged: an example
> is only published when gently is byte-identical to the pinned oracle.

Every example is byte-verified against the pinned oracle
(**Graph::Easy v0.69** @ `ededa3d7…`, `PERL_HASH_SEED=0`) before it is
published. The workflow (guided first, then explore):

1. **Mine the oracle bytes** — add your input as
   `tests/fixtures/graph-easy/<name>.txt`, then run `just oracle-record`
   (records with Graph::Easy v0.69, `PERL_HASH_SEED=0`; see
   `tests/fixtures/graph-easy/README.md` for the recording setup).
2. **Commit the example** — create `examples/<name>/` containing
   `input.gv` (your graph source) and `expected.txt` = the recorded
   `<name>.ascii.expected` companion with its leading `# oracle: ` pin
   header stripped.
3. **Verify** — run `just examples-verify`; it re-renders every
   `input.gv` with the gently CLI and fails on any byte mismatch. Publish
   the example only when this is green (byte-identical).
4. **If it is NOT byte-identical** — do not publish an example; instead
   add a divergence row to `docs/known-hard-cases.md` (the admission
   rule) and track it in the issue tracker (`bd`).

For further exploration: the fixture corpus lives in
`tests/fixtures/graph-easy/` (with its own README on recording and pin
headers), and `just repro-claims` lists the claim probes behind the
gates.
