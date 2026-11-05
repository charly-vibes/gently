//! Purpose: render and emit dispatch for the pipeline (specs/cli.md
//! c1–c6).
//! Responsibilities: read the source, parse it, render the requested
//! format, route the bytes to stdout or the `--output` file, and emit
//! through the genesis `Guide` (human mode vs `--json` envelope).
//! Rationale: human mode keeps `Output::emit`'s `Debug`-print path
//! (`RawAscii` writes the rendered bytes verbatim minus exactly one
//! trailing newline) so the byte-compat oracle stream of p1–p4 is
//! preserved; `--json` rides `Output::emit`'s envelope path so failures
//! come out typed (`envelope_kind: "error"`, c6).

use std::fmt;

use serde::Serialize;
use std::path::PathBuf;

use genesis::guide::{Guide, Output, OutputFormat};
use gently_core::{
    layout,
    render::{ascii, graphviz, html, txt},
};

use crate::pipeline::input::parse_input;
use crate::pipeline::source::read_source;
use crate::{Cli, CliError};

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
fn write_output(art: &str, output_path: Option<&std::path::Path>) -> Result<(), CliError> {
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
