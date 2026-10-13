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
    layout,
    parse::text,
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
    let graph = text::parse(&source)
        .map_err(|e| CliError(format!("parse error, line {}: {}", e.line, e.message)))?;
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
