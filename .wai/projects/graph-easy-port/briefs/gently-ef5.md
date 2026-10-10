# Subagent brief: gently-ef5 — implement cli (genesis surface, p5–p8)

You are a pi subagent in repo `/var/home/sasha/para/areas/dev/gh/charly/gently`
(Rust workspace `crates/gently-core` + `crates/gently-cli`, edition 2021).
An orchestrator has claimed ticket `gently-ef5` and is hands-off: you own
the implementation end-to-end. The orchestrator will verify with real
gates — never claim work the diff doesn't contain.

## Orientation (do this first — max 10 tool calls)

- `export WAI_PROJECT=graph-easy-port`
- `bd show gently-ef5`
- Read `specs/cli.md` IN FULL — c1–c8 + model + properties p1–p8 are the
  binding contract. p1–p4 (stdin/positionals, `--as`/`--output`, streams
  + exit codes, unknown-format 255) are DONE (gently-0h9, tests in
  `crates/gently-cli/tests/cli.rs` module `scenarios::cli`).
- **Your scope is p5–p8** (c5/c6/c7/c8), all unimplemented:
  - **c5/p5** — genesis-vibes foundation: command dispatch via `Guide`,
    output via `Output::emit`, verbosity via `CliVerbosity`
    (`-v`/`-vv`/`-vvv`, `-q`), format via `CliFormat`; `init`
    subcommand registers gently in `.genesis/tools.toml` via genesis
    discovery.
  - **c6/p6** — `--json`: machine-readable results wrapped in the
    genesis Envelope (`ok`, `envelope_version`, `envelope_kind`, `data`,
    `warnings`, `hints`, `meta`); human mode prints the rendered graph
    as raw bytes (byte-compat preserved).
  - **c7/p7** — unknown subcommand or flag → genesis suggestion
    (`DidYouMean` / `Fix`) on stderr before the nonzero exit.
  - **c8/p8** — `gently doctor` runs suite health checks through genesis
    `DoctorRunner` — oracle availability (perl + pinned Graph::Easy),
    output-format support, fixture pin freshness — and applies
    available auto-fixes.
- The genesis-vibes API is a real dependency (crate `genesis-vibes`
  0.12.1, `use genesis::...` in `crates/gently-cli/src/main.rs` today).
  Its source is in
  `~/.cargo/registry/src/index.crates.io-*/genesis-vibes-0.12.1/src/` —
  read `guide.rs`, `suggestions.rs` (`suggest_typo` /
  `suggest_order`), `envelope.rs`, `doctor.rs`, `cli.rs`
  (`CliVerbosity`, `CliFormat`), `discovery.rs`/`config.rs` (tools.toml)
  BEFORE designing. Do not guess API shapes; read them.
- Existing wiring: `crates/gently-cli/src/main.rs` — `main()` builds the
  `Guide`, `run_pipeline` dispatches. The doc comments name the deferred
  ef5 surfaces. `--as`/`--output`/positionals/exit-255 already work
  (p1–p4 green, do not regress them — the BrokenPipe-tolerant stdin
  writer is intentional, see the run_gently comment).
- `wai search "cli"` for accumulated patterns.

## What to build

All in `crates/gently-cli`:

1. **p5** — subcommand routing per genesis `Guide`: `gently` with a
   pipeline input (default run), `gently init` (registers in
   `.genesis/tools.toml` via genesis discovery), verbosity flags
   routed to `CliVerbosity` and actually honored (quiet suppresses,
   -vvv increases). Format routing via `CliFormat` where the genesis
   contract expects it.
2. **p6** — `--json` flag: successful render emits the genesis
   `Envelope` (data = rendered bytes as string, meta = tool/version
   info); parse/render failures also emit a typed Envelope (envelope
   kind per the genesis convention); human mode stays byte-identical
   to today's stdout (p1–p4 guard this).
3. **p7** — unknown subcommand/flag: use genesis `suggest_typo` (or
   `suggest_order`) against the known name registry to emit a
   `DidYouMean`/`Fix` suggestion on stderr before the nonzero exit.
4. **p8** — `gently doctor`: genesis `DoctorRunner` with checks for
   (a) oracle availability — `perl` present + pinned Graph::Easy
   loadable under the pin (see justfile oracle recipes for the exact
   pin), (b) output-format support, (c) fixture pin freshness (pin
   commit recorded in tooling vs specs headers). Auto-fixes where
   genesis provides them; the report names each check's status.

**Contract wiring (critical):**
- Extend `crates/gently-cli/tests/cli.rs` module `scenarios::cli` with
  one `#[test]` per property id p5–p8 (the existing p1–p4 layout).
  Tests spawn the real binary (env!("CARGO_BIN_EXE_gently")).
- Verify `.espectacular/cli/p1–p8.toml` filters match real test paths
  (`scenarios::cli::pN` — they look right; confirm p5–p8 actually
  execute after your change). After your change,
  `ah check --run-tests`'s no-tests-ran backlog must shrink by the cli
  contracts.
- p5/p8 tests that touch the filesystem (init, doctor) must use
  tempdirs — never the repo working tree; init must not write
  `.genesis/` into the repo.

## TDD sequence (red → green → refactor commits)

1. **RED:** commit the scenario tests p5–p8 first, honestly failing.
2. **GREEN:** implement until all tests pass. Commit GREEN as soon as
   tests pass.
3. **REFACTOR:** tidy main.rs (the pretender gate enforces file line
   limits — split handlers into modules when main.rs grows),
   contract-flag confirmation, README touch-ups — separate commit.

## Hard scope guard

- Allowed files: `crates/gently-cli/**`,
  `crates/gently-core/src/lib.rs` (exports only, if needed),
  `.espectacular/cli/p1.toml`…`p8.toml` (flags only),
  `tests/repro/**` (only if a genuine behavior question forces a new
  probe — record it in your report, never amend spec rows).
- Never edit: `specs/*.md`, `openspec/**`,
  `crates/gently-core/src/**` (except lib.rs exports), `.wai/**`,
  `deny.toml`, `.gitattributes`, `.github/**`.
- Do NOT push, do NOT close the bead, do NOT run `wai close` — the
  orchestrator owns ship.

## Exact gates (all green before you report done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
spk lint specs/
ah check
ah check --run-tests   # cli p1–p8 must now execute
just gates
```

## Repo facts (boilerplate)

- **Commit hygiene:** before ANY `git commit`, `git status --short`; stage
  only files YOU authored; end subjects with `(gently-ef5)`.
- **File headers:** every source file carries the
  Purpose/Responsibilities/Rationale header.
- CI is currently green on all 4 jobs (windows corpus CRLF fix landed
  via `.gitattributes`, cargo-deny via `deny.toml`) — keep it that way.

## Follow-up threshold

If the full genesis surface does not fit the session (p5–p8 is the
floor), ship the coherent subset that keeps every existing test green
and file follow-up beads listing exactly what remains — never leave
the tree red.

## Report format (end with this — the orchestrator verifies against it)

## Report

**Commits**
- `<hash>` <message> — <what it does, one line>

**Gates run**
- `cargo test --workspace` → <result>
- `cargo clippy ...` → <result>
- `spk lint specs/` → <result>
- `ah check` → <result>
- `ah check --run-tests` (cli p1–p8) → <result>
- `just gates` → <result>

**Deviations** (or "none")

**Next**
- <exact next action for the orchestrator, or "ticket complete">

## ⏱ TIME BUDGET (binding)

You are hard-capped at 30 minutes wall clock. Budget:
- **Minutes 0–5:** orientation — read this brief, specs/cli.md, and the
  genesis-vibes guide.rs/suggestions.rs/envelope.rs/doctor.rs API
  signatures. That is ALL. Max 10 tool calls.
- **Minutes 5–10:** write the RED scenario tests (p5–p8 in
  tests/cli.rs); commit RED.
- **Minutes 10–22:** GREEN implementation; commit GREEN as soon as
  tests pass.
- **Minutes 22–28:** flags wiring + gates.
- **Minutes 28–30:** report.
Never re-read a file you have already read. Never cat an entire
directory. If RED+GREEN cannot both land, ship RED-commit + partial
GREEN and say so in the report — a committed RED beats an uncommitted
almost.
