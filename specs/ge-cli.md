---
id: ge.cli
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
| c1 | invariant | With no file arguments, input is read from stdin; each file argument is parsed in order and concatenated as one graph source. | [[ge.cli]] | |
| c2 | invariant | The `--output` flag selects the renderer (ascii, boxart, html, graphviz, txt); with no flag the default output is ascii. | [[ge.cli]] | |
| c3 | invariant | Rendered output goes to stdout, diagnostics go to stderr, and the process exits 0 on success and nonzero on any parse or render error. | [[ge.cli]] | [[ge.cli.c4]] |
| c4 | effect | An unknown output format produces a diagnostic on stderr naming the requested format and the valid formats, followed by exit code 2. | [[ge.cli]] | |

## Model

### States

- `reading`
- `rendering`
- `cli_done`
- `cli_failed`

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | reading | rendering | [[ge.cli.c1]] |
| t2 | rendering | cli_done | [[ge.cli.c2]] |
| t3 | rendering | cli_failed | [[ge.cli.c2]] |
| t4 | cli_failed | reading | [[ge.cli.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.cli.c1]] | invocations with and without file arguments | stdin and file inputs reach the parser with the same bytes |
| p2 | unit | [[ge.cli.c2]] | invocations over every output format with and without the flag | each requested format renders through its renderer; the default is ascii |
| p3 | unit | [[ge.cli.c3]] | successful and failing invocations | streams and exit codes match the constraint exactly |
| p4 | unit | [[ge.cli.c4]] | invocations with unknown formats | the diagnostic names both the requested and the valid formats and exit code is 2 |
