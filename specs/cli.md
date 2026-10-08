---
id: cli
kind: intent
statement: "WHEN the gently command is invoked THE cli SHALL read graph text or DOT from files or stdin, render it in the requested output format, and print the result to stdout."
---

# cli

The command-line entry point — gently's port of the `graph-easy` script.
It wires [[ge.text_parser]], [[ge.dot_parser]], the renderers, and
[[ge.layout]] into one pipeline and defines the observable process
behavior of the port.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | With no file arguments, input is read from stdin; each file argument is parsed in order and concatenated as one graph source. | [[cli]] | |
| c2 | invariant | The `--output` flag selects the renderer (ascii, boxart, html, graphviz, txt); with no flag the default output is ascii. | [[cli]] | |
| c3 | invariant | Rendered output goes to stdout, diagnostics go to stderr, and the process exits 0 on success and nonzero on any parse or render error. | [[cli]] | [[cli.c4]] |
| c4 | effect | An unknown output format produces a diagnostic on stderr naming the requested format and the valid formats, followed by exit code 2. | [[cli]] | [[cli.c7]] |
| c5 | invariant | The CLI is built on the genesis-vibes foundation (crate >= 0.12): command dispatch via `Guide`, output via `Output::emit`, verbosity via `CliVerbosity` (`-v`/`-vv`/`-vvv`, `-q`), and format via `CliFormat`; `init` registers gently in `.genesis/tools.toml` via genesis discovery. | [[cli]] | |
| c6 | invariant | With `--json`, machine-readable results are emitted wrapped in the genesis Envelope (`ok`, `envelope_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta`); without it (human mode), the rendered graph is printed to stdout as raw bytes exactly as the oracle comparison requires. | [[cli]] | |
| c7 | invariant | An unknown subcommand or flag produces a genesis suggestion (`DidYouMean` / `Fix`) on stderr before the nonzero exit. | [[cli]] | [[cli.c8]] |
| c8 | effect | `gently doctor` runs the suite health checks through genesis `DoctorRunner` — oracle availability (perl + pinned Graph::Easy), output-format support, fixture pin freshness — and applies available auto-fixes. | [[cli]] | |

## Model

### States

- `reading`
- `rendering`
- `cli_done`
- `diagnosing`
- `cli_failed` emits: [[cli.c4]]

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | reading | rendering | [[cli.c1]] |
| t2 | rendering | cli_done | [[cli.c2]] |
| t3 | rendering | cli_failed | [[cli.c2]] |
| t4 | reading | diagnosing | [[cli.c5]] |
| t5 | diagnosing | cli_done | [[cli.c6]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[cli.c1]] | invocations with and without file arguments | stdin and file inputs reach the parser with the same bytes |
| p2 | unit | [[cli.c2]] | invocations over every output format with and without the flag | each requested format renders through its renderer; the default is ascii |
| p3 | unit | [[cli.c3]] | successful and failing invocations | streams and exit codes match the constraint exactly |
| p4 | unit | [[cli.c4]] | invocations with unknown formats | the diagnostic names both the requested and the valid formats and exit code is 2 |
| p5 | unit | [[cli.c5]] | every subcommand and flag combination | dispatch, verbosity, and format routing behave per the genesis Guide contract, and init writes the `.genesis/tools.toml` entry |
| p6 | unit | [[cli.c6]] | render runs with and without `--json` | json mode wraps the result in a genesis Envelope; human mode emits raw bytes identical to non-wrapped output |
| p7 | unit | [[cli.c7]] | misspelled subcommands and flags | every misspelling yields a DidYouMean/Fix suggestion naming the closest known name |
| p8 | unit | [[cli.c8]] | environments with and without the oracle toolchain and fresh/stale fixture pins | the DoctorReport names each check's status and applies available auto-fixes |
