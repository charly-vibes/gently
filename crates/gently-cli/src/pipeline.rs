//! Purpose: the `gently` render pipeline — parse, lay out, render, and
//! emit through the genesis `Guide` (specs/cli.md c1–c6; p1–p4 in
//! gently-0h9, `--json` envelope in gently-ef5).
//! Responsibilities: read the graph source (stdin or first file
//! argument), run parse → layout → ascii render, resolve the output
//! format per the upstream flag roles, dispatch human vs `--json` emit,
//! and route rendered bytes to stdout or the `--output` file.
//! Rationale: human mode keeps `Output::emit`'s `Debug`-print path
//! (`RawAscii` writes the rendered bytes verbatim minus exactly one
//! trailing newline) so the byte-compat oracle stream of p1–p4 is
//! preserved; `--json` rides `Output::emit`'s envelope path so failures
//! come out typed (`envelope_kind: "error"`, c6); `--output` suppresses
//! stdout via an unreachable verbosity threshold, bug-for-bug with
//! upstream writing nothing to stdout when a file receives the bytes.

use std::fmt;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

use genesis::guide::{Guide, Output, OutputFormat};
use gently_core::{
    graph::Graph,
    layout,
    parse::{dot, text},
    render::{ascii, graphviz, html, txt},
};
use serde::Serialize;

use crate::{Cli, CliError};

/// Output formats wired in this slice (specs/cli.md c2): ascii, html,
/// graphviz (`dot` aliases it upstream) and txt — each routed to its
/// library renderer. `boxart`/`svg`/`vcg`/`gdl`/`graphml` stay exit-255
/// (c4): upstream v0.69 has no `as_boxart`/`as_svg` method, and the
/// vcg/gdl/graphml writers are documented scope-offs (gently-4ig).
pub const SUPPORTED_FORMATS: &[&str] = &["ascii", "html", "graphviz", "txt"];

/// Upstream's extension → format map (bin/graph-easy get_options): the
/// output filename infers the format when `--as` is absent. `dot` is an
/// alias for `graphviz`.
const EXTENSION_FORMATS: &[(&str, &str)] = &[
    ("html", "html"),
    ("svg", "svg"),
    ("txt", "txt"),
    ("dot", "graphviz"),
    ("vcg", "vcg"),
    ("gdl", "gdl"),
    ("graphml", "graphml"),
];

/// Which parser upstream's text-parser sniffing redirects to (Parser.pm
/// ~557): a line-start `(strict)?(di)?graph [name] {` header swaps in the
/// Graphviz parser, `graph: {` the VCG parser.
enum Sniffed {
    Text,
    Dot,
    Vcg,
}

/// The effective `--from` selection: the flag wins, then the boolean
/// aliases in upstream get_options' override order (txt, graphviz, vcg,
/// gdl — the `dot` VALUE aliases graphviz; `--from_dot` is not an upstream
/// option).
#[derive(Clone, Copy, PartialEq)]
enum From {
    Txt,
    Graphviz,
    Vcg,
    Gdl,
}

impl From {
    fn name(self) -> &'static str {
        match self {
            From::Txt => "txt",
            From::Graphviz => "graphviz",
            From::Vcg => "vcg",
            From::Gdl => "gdl",
        }
    }
}

fn input_from(cli: &Cli) -> Result<Option<From>, CliError> {
    // The boolean aliases override `--from` in get_options' loop order
    // (qw/txt graphviz dot vcg gdl/ — later wins), then the value is
    // validated: an unknown name dies upstream (bin/graph-easy line 333).
    let mut raw = cli.from_.as_deref();
    if cli.from_txt {
        raw = Some("txt");
    }
    if cli.from_graphviz {
        raw = Some("graphviz");
    }
    if cli.from_vcg {
        raw = Some("vcg");
    }
    if cli.from_gdl {
        raw = Some("gdl");
    }
    Ok(match raw {
        None => None,
        Some("txt") => Some(From::Txt),
        Some("dot" | "graphviz") => Some(From::Graphviz),
        Some("vcg") => Some(From::Vcg),
        Some("gdl") => Some(From::Gdl),
        Some(other) => return Err(CliError(format!("Unknown input format '{other}'"))),
    })
}

/// Validate the `--from` surface before the pipeline (dispatch-level, c9):
/// an unknown value dies upstream at get_options (bin/graph-easy line 333,
/// `Unknown input format '<fmt>'`) — exit 255; `vcg`/`gdl` (value or
/// boolean alias) name the VCG parser gently never ports — documented
/// scope-offs, exit 255 like the c4 output-side unwired formats.
pub fn check_input_format(cli: &Cli) -> Result<(), CliError> {
    match input_from(cli)? {
        None | Some(From::Txt) | Some(From::Graphviz) => Ok(()),
        Some(f @ (From::Vcg | From::Gdl)) => Err(CliError(format!(
            "{} input parsing is not wired: gently has no VCG parser (scope-off, gently-cij)",
            f.name()
        ))),
    }
}

/// Parse the source with the c9-selected input parser: `--from` picks the
/// parser, and a txt-selected source is sniffed for a line-start DOT or
/// VCG header (upstream Graph::Easy::Parser sniffing — the sniff applies
/// even under an explicit `--from txt`). A sniffed `graph: {` header
/// routes to the VCG parser upstream; with no gently VCG parser the sniff
/// is a scope-off diagnostic (upstream exits 0/2 via its own parser).
fn parse_input(cli: &Cli, source: &str) -> Result<Graph, CliError> {
    match input_from(cli)? {
        Some(From::Graphviz) => dot::parse(source).map_err(|e| {
            CliError(format!("parse error, line {}: {}", e.line, e.message))
        }),
        Some(f @ (From::Vcg | From::Gdl)) => Err(CliError(format!(
            "{} input parsing is not wired: gently has no VCG parser (scope-off, gently-cij)",
            f.name()
        ))),
        Some(From::Txt) | None => match sniff(source) {
            Sniffed::Dot => dot::parse(source).map_err(|e| {
                CliError(format!("parse error, line {}: {}", e.line, e.message))
            }),
            Sniffed::Vcg => Err(CliError(
                "graph: { input requires the upstream VCG parser — not wired (scope-off, gently-cij)"
                    .to_string(),
            )),
            Sniffed::Text => text::parse(source).map_err(|e| {
                CliError(format!("parse error, line {}: {}", e.line, e.message))
            }),
        },
    }
}

/// Upstream Graph::Easy::Parser content sniffing (Parser.pm ~557): the
/// source is scanned line-wise — the digraph-family header regexes run
/// first over the whole text, then the `graph: {` VCG sniff. A line head
/// matches the DOT family when, after leading whitespace and `/*...*/`
/// comments, it reads `strict? digraph|graph [name]? {` — where a nameless
/// `graph {` matches only under `strict` (the upstream regexes require a
/// name otherwise). Whitespace between the tokens is optional upstream.
fn sniff(source: &str) -> Sniffed {
    if source
        .lines()
        .any(|line| line_head_sniffs_dot(&line.to_lowercase()))
    {
        return Sniffed::Dot;
    }
    // The VCG sniff is case-sensitive upstream (no /i).
    if source.lines().any(line_head_sniffs_vcg) {
        return Sniffed::Vcg;
    }
    Sniffed::Text
}

/// Strip leading whitespace and /*...*/ comment groups from a line head
/// (upstream's $o_cmt prefix alternation).
fn strip_line_head(line: &str) -> &str {
    let mut rest = line;
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix("/*") {
            match after.find("*/") {
                Some(i) => rest = &after[i + 2..],
                None => return rest,
            }
        } else {
            return rest;
        }
    }
}

/// One lowercased line head against the upstream DOT-family header regex
/// `(strict)?$o_cmt(di)?graph$o_cmt("[^"]*"|[\w_]+)?$o_cmt\{` — with the
/// upstream nuance that a nameless `graph {` only matches under `strict`.
fn line_head_sniffs_dot(line: &str) -> bool {
    let rest = strip_line_head(line);
    let (strict, rest) = match rest.strip_prefix("strict") {
        Some(r) => (true, r.trim_start()),
        None => (false, rest),
    };
    let (has_di, rest) = if let Some(r) = rest.strip_prefix("digraph") {
        (true, r)
    } else if let Some(r) = rest.strip_prefix("graph") {
        (false, r)
    } else {
        return false;
    };
    let rest = rest.trim_start();
    // Optional graph name: a quoted string or a word ([\w_]+ upstream).
    let (named, rest) = if let Some(after) = rest.strip_prefix('"') {
        match after.find('"') {
            Some(i) => (true, &after[i + 1..]),
            None => return false,
        }
    } else if rest.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
        let n = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        (true, &rest[n..])
    } else {
        (false, rest)
    };
    let rest = rest.trim_start();
    if !rest.starts_with('{') {
        return false;
    }
    // A nameless, non-strict `graph {` does not match the upstream regexes.
    !(!has_di && !named && !strict)
}

/// One line against the upstream VCG sniff regex `^[\s\n\t]*graph\s*:\s*\{`
/// (case-sensitive — no /i on that match).
fn line_head_sniffs_vcg(line: &str) -> bool {
    let rest = line.trim_start();
    let Some(rest) = rest.strip_prefix("graph") else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix(':') else {
        return false;
    };
    rest.trim_start().starts_with('{')
}

/// The rendered output, printed by `Output::emit`'s human path and
/// serialized as `data` in `--json` envelopes (c6).
///
/// `Output::print` writes the payload through `Debug` plus `writeln!`'s
/// final newline, so `Debug` writes the bytes verbatim minus one trailing
/// newline to reproduce the oracle stream exactly (specs/cli.md c3).
#[derive(Serialize, Default)]
pub struct RawAscii {
    pub bytes: String,
}

impl RawAscii {
    pub fn new(bytes: String) -> RawAscii {
        RawAscii { bytes }
    }
}

impl fmt::Debug for RawAscii {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body = self.bytes.strip_suffix('\n').unwrap_or(&self.bytes);
        f.write_str(body)
    }
}

/// Dispatch the pipeline through the Guide (cli.c3/c5/c6): human mode
/// prints the raw ascii to stdout via `Output::emit`; `--json` wraps the
/// result (success or typed failure) in the genesis Envelope. Rendered
/// bytes go to the `--output` file when named (cli.c2). Handler errors go
/// to stderr with exit 1 in human mode; in json mode they emit a typed
/// error envelope (c6).
pub fn run(
    guide: &Guide,
    cli: &Cli,
    output_path: Option<PathBuf>,
    format: OutputFormat,
    format_name: &str,
) -> i32 {
    guide.run_formatted(format, || {
        let result = render(cli, format_name);
        match result {
            Ok(art) => match write_output(&art, output_path.as_deref()) {
                // When the rendered bytes go to the `--output` file, nothing
                // is printed to stdout (upstream bug-for-bug) — suppress the
                // emit path via an unreachable verbosity threshold.
                Ok(()) if output_path.is_some() => {
                    Ok(Output::success(RawAscii::new(String::new())).with_verbosity(u8::MAX))
                }
                Ok(()) => {
                    let note = format!(
                        "rendered {} bytes of {format_name} via the gently pipeline",
                        art.len()
                    );
                    Ok(Output::success(RawAscii::new(art)).with_warning(note))
                }
                Err(e) => error_output(e, format),
            },
            Err(e) => error_output(e, format),
        }
    })
}

/// Route a failure to the emit path: a typed error envelope in json mode
/// (c6), a bare human diagnostic otherwise.
fn error_output(
    e: CliError,
    format: OutputFormat,
) -> Result<Output<RawAscii>, Box<dyn std::error::Error>> {
    if format == OutputFormat::Json {
        // c6: failures emit a typed error envelope, not a bare human
        // diagnostic.
        Ok(Output::failure(e.to_string()))
    } else {
        Err(Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// Write the rendered bytes to the `--output` file when named (cli.c2);
/// `Ok(())` without a path means stdout carries the bytes.
fn write_output(art: &str, output_path: Option<&Path>) -> Result<(), CliError> {
    let Some(path) = output_path else {
        return Ok(());
    };
    std::fs::write(path, art)
        .map_err(|e| CliError(format!("cannot write to {}: {e}", path.display())))
}

/// Parse, lay out, and render the requested format, returning the rendered
/// bytes (c1/c2). ascii, html, graphviz and txt route to their library
/// renderers (c2); the c4 guard rejects other formats before dispatch.
fn render(cli: &Cli, format_name: &str) -> Result<String, CliError> {
    let source = read_source(cli)?;
    let graph = parse_input(cli, &source)?;
    match format_name {
        "graphviz" => Ok(graphviz::render(&graph)),
        "txt" => Ok(txt::render(&graph)),
        "html" => {
            let lay = layout::layout(&graph);
            // The upstream script's `as_html_file` alias embeds the table in
            // a full document (ge.html_render's scope-off); the CLI serves
            // the document form the library pins.
            html::document(&graph, &lay)
                .map_err(|e| CliError(format!("render error: {}", e.message)))
        }
        _ => {
            let lay = layout::layout(&graph);
            let art = ascii::render(&graph, &lay)
                .map_err(|e| CliError(format!("render error: {}", e.message)))?;
            Ok(art)
        }
    }
}

/// Resolve the output format per upstream bin/graph-easy (specs/cli.md c2):
/// `--as` wins (with `dot` aliasing `graphviz`); else the output filename's
/// extension via the upstream map; else the ascii default.
pub fn resolve_format(cli: &Cli, output_path: Option<&Path>) -> String {
    if let Some(fmt) = cli.as_.as_deref() {
        return if fmt == "dot" {
            "graphviz".to_string()
        } else {
            fmt.to_string()
        };
    }
    if let Some(path) = output_path {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some((_, fmt)) = EXTENSION_FORMATS.iter().find(|(e, _)| *e == ext) {
                return (*fmt).to_string();
            }
        }
    }
    "ascii".to_string()
}

/// Read the graph source: stdin when no file arguments are given, otherwise
/// the first file argument's contents (specs/cli.md c1 — a second positional
/// names the output file, further positionals are ignored, so only the
/// first is ever parsed).
fn read_source(cli: &Cli) -> Result<String, CliError> {
    read_source_impl(cli, std::io::stdin().is_terminal())
}

/// The source read with the stdin kind injected so the interactive guard is
/// unit-testable: a terminal stdin with no file argument fails fast (c1
/// guard for gently-in2 — the upstream oracle is always exercised with
/// piped stdin, so this deviates from upstream only where upstream would
/// block forever on an interactive prompt).
fn read_source_impl(cli: &Cli, stdin_is_terminal: bool) -> Result<String, CliError> {
    let Some(file) = cli.files.first() else {
        if stdin_is_terminal {
            return Err(CliError(
                "no input file given and stdin is a terminal; pipe graph text into gently or pass an input file"
                    .to_string(),
            ));
        }
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CliError(format!("cannot read stdin: {e}")))?;
        return Ok(buf);
    };
    let content = std::fs::read_to_string(file)
        .map_err(|e| CliError(format!("cannot read {}: {e}", file.display())))?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    fn cli_with(args: &[&str]) -> Cli {
        Cli::try_parse_from(std::iter::once("gently").chain(args.iter().cloned()))
            .expect("valid args")
    }

    /// gently-in2: with no file argument and a terminal stdin, the source
    /// read fails fast with a diagnostic instead of blocking forever —
    /// interactive invocations (and agent subprocesses inheriting an open
    /// TTY) get a usable error, piped stdin keeps the c1 oracle contract.
    #[test]
    fn terminal_stdin_without_file_fails_fast() {
        let cli = cli_with(&[]);
        let err = read_source_impl(&cli, true).expect_err("must not block on a TTY");
        assert!(err.to_string().contains("stdin is a terminal"));
        assert!(err.to_string().contains("input file"));
    }

    /// The piped path is unchanged: non-terminal stdin is read (cli.c1).
    #[test]
    fn piped_stdin_still_read_when_not_terminal() {
        let cli = cli_with(&[]);
        let source = read_source_impl(&cli, false).expect("piped stdin reads");
        assert_eq!(source, "");
    }
}
