---
id: cli
kind: intent
statement: "WHEN the gently command is invoked THE cli SHALL read graph text or DOT from files or stdin, render it in the requested output format, and print the result to stdout."
---

# cli

The command-line entry point — gently's port of the `graph-easy` script.
It wires [[ge.text_parser]], [[ge.dot_parser]], the renderers, and
[[ge.layout]] into one pipeline and defines the observable process
behavior of the port. Flag roles are bug-for-bug with the upstream script
(Graph::Easy v0.69 `bin/graph-easy`, re-derived under [[ge.oracle.c7]]):
`--as` selects the format, `--output` names the output file, and the
positionals are `[inputfile [outputfile]]`.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | With no file arguments, input is read from stdin; otherwise the first file argument is the input, a second file argument (or `--output`) names the output file, and further file arguments are silently ignored — bug-for-bug with the upstream script. Exception: when stdin is a terminal and no file argument is given, the source read fails fast with a diagnostic instead of blocking on interactive input (the oracle only ever exercises piped stdin, so agent/interactive invocations never hang; gently-in2). | [[cli]] | |
| c2 | invariant | The `--as` flag selects the output format (ascii, html, graphviz, txt; `dot` aliases graphviz), each routed to its library renderer, defaulting to ascii, or to the output filename's extension format when `--as` is absent; the `--output` flag names the output file that receives the rendered bytes — stdout when absent. | [[cli]] | |
| c3 | invariant | Rendered output goes to stdout, diagnostics go to stderr, and the process exits 0 on success and nonzero on any parse or render error. | [[cli]] | [[cli.c4]] |
| c4 | effect | An unknown output format produces a diagnostic on stderr naming the requested format and the valid formats, followed by exit code 255 — bug-for-bug with the upstream script, whose unknown `--as` dies calling the missing `as_<fmt>` method. The upstream-renderable but unwired names stay on this exit path as documented scope-offs (gently-4ig, tests/repro/claims/cli-formats.observed): svg dies under the pinned oracle's shebang perl (no in-dist `As_svg`), and boxart/vcg/gdl/graphml have no gently renderer yet. | [[cli]] | [[cli.c7]] |
| c5 | invariant | The CLI is built on the genesis-vibes foundation (crate >= 0.12): command dispatch via `Guide`, output via `Output::emit`, verbosity via `CliVerbosity` (`-v`/`-vv`/`-vvv`, `-q`), and format via `CliFormat`; `init` registers gently in `.genesis/tools.toml` via genesis discovery. | [[cli]] | |
| c6 | invariant | With `--json`, machine-readable results are emitted wrapped in the genesis Envelope (`ok`, `envelope_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta`); without it (human mode), the rendered graph is printed to stdout as raw bytes exactly as the oracle comparison requires. | [[cli]] | |
| c7 | invariant | An unknown subcommand or flag produces a genesis suggestion (`DidYouMean` / `Fix`) on stderr before the nonzero exit. | [[cli]] | [[cli.c8]] |
| c8 | effect | `gently doctor` runs the suite health checks through genesis `DoctorRunner` — oracle availability (perl + pinned Graph::Easy), output-format support, fixture pin freshness — and applies available auto-fixes. | [[cli]] | |
| c9 | invariant | The input parser is selected by the `--from` flag — txt default, `dot` aliases graphviz, the boolean aliases `--from_txt`/`--from_graphviz`/`--from_vcg`/`--from_gdl` override `--from` in upstream's loop order, and `--from_dot` is NOT a declared upstream option (it hits the unknown-flag path, exit 2) — and, when txt is selected, by upstream's content sniffing: a line-start `(strict)?(di)?graph [name]? {` header routes the source to the DOT parser [[ge.dot_parser]] even under an explicit txt selection (the Graphviz parser never sniffs back), and a `graph: {` header routes to the VCG parser upstream. Unknown `--from` values produce the upstream `Unknown input format '<fmt>'` diagnostic and exit 255 (get_options' die); the upstream-valid `vcg`/`gdl` names route to a VCG parser gently never ports — documented scope-offs exiting 255 — and a sniffed `graph: {` header is the same scope-off at parse time (gently-cij; probe evidence tests/repro/claims/cli-input-from.observed). | [[cli]] | [[cli.c3]] |

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
| t1 | reading | rendering | [[cli.c1]] [[cli.c9]] |
| t2 | rendering | cli_done | [[cli.c2]] |
| t3 | rendering | cli_failed | [[cli.c2]] |
| t4 | reading | diagnosing | [[cli.c5]] |
| t5 | diagnosing | cli_done | [[cli.c6]] |
| t6 | reading | cli_failed | [[cli.c1]] [[cli.c5]] [[cli.c9]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[cli.c1]] | invocations with no, one, and multiple file arguments | stdin and the first file argument reach the parser with the same bytes; the second positional names the output file and further positionals are ignored |
| p2 | unit | [[cli.c2]] | invocations over every output format with and without the flag | each requested format renders through its renderer; the default is ascii; `--output` receives the rendered bytes instead of stdout and extension inference applies when `--as` is absent |
| p3 | unit | [[cli.c3]] | successful and failing invocations | streams and exit codes match the constraint exactly |
| p4 | unit | [[cli.c4]] | invocations with unknown formats | the diagnostic names both the requested and the valid formats and exit code is 255 |
| p5 | unit | [[cli.c5]] | every subcommand and flag combination | dispatch, verbosity, and format routing behave per the genesis Guide contract, and init writes the `.genesis/tools.toml` entry |
| p6 | unit | [[cli.c6]] | render runs with and without `--json` | json mode wraps the result in a genesis Envelope; human mode emits raw bytes identical to non-wrapped output |
| p7 | unit | [[cli.c7]] | misspelled subcommands and flags | every misspelling yields a DidYouMean/Fix suggestion naming the closest known name |
| p8 | unit | [[cli.c8]] | environments with and without the oracle toolchain and fresh/stale fixture pins | the DoctorReport names each check's status and applies available auto-fixes |
| p9 | unit | [[cli.c1]] | no-file-argument invocations with terminal stdin and with piped stdin | a terminal stdin fails fast with a diagnostic naming stdin and the input file; piped stdin is still read to EOF |
| p10 | unit | [[cli.c9]] | invocations across `--from` values, boolean aliases, and sniffable/unsniffable sources | the selected parser parses the source (a digraph fixture renders the oracle's DOT bytes; graph-text input parses unchanged; a forced DOT parser rejects text content); unknown values and vcg/gdl exit 255 with diagnostics naming the value or the unwired VCG parser; `--from_dot` exits 2 through the unknown-flag path; a sniffed `graph: {` header is a scope-off diagnostic |
