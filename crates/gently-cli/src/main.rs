//! Purpose: the `gently` command-line entry point — the end-to-end tracer
//! pipeline (epic gently-2po, tb.cli / gently-2po.9).
//! Responsibilities: read graph text from file arguments (concatenated in
//! order) or stdin, parse it, lay it out, render ascii, and write the raw
//! oracle bytes to stdout; diagnostics go to stderr; exit 0 on success, 1 on
//! parse/render/IO errors, and 2 on an unknown output format
//! (`specs/cli.md` c1, c3, c4). Dispatch runs on the genesis-vibes `Guide`
//! frame (cli.c5 foundation): clap args carry `CliVerbosity`, the pipeline
//! handler returns `Output<T>` and is emitted through `Output::emit`.
//! Rationale: `--json` envelopes, `init`/`doctor`, and suggestion surfaces
//! are deferred to gently-ef5. `Output::emit`'s human path prints the
//! payload through `Debug` formatting (plus `writeln!`'s final newline), so
//! [`RawAscii`]'s `Debug` writes the rendered bytes verbatim minus exactly
//! one trailing newline — the byte-identical oracle stream cli.c3 pins.

use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use genesis::guide::{ErrorSink, Guide, Output, OutputFormat, Verbosity};
use gently_core::{layout, parse::text, render::ascii};
use serde::Serialize;

/// Output formats valid in this slice (specs/cli.md c2 — only ascii is
/// wired; boxart/html/graphviz/txt land with later capability slices).
const VALID_FORMATS: &str = "ascii";

#[derive(Parser)]
#[command(
    name = "gently",
    version,
    about = "Graph text in, rendered output out"
)]
struct Cli {
    /// Input files (parsed in order and concatenated); stdin when absent.
    files: Vec<PathBuf>,

    /// Output format (valid: ascii)
    #[arg(long)]
    format: Option<String>,

    /// Verbosity plumbing from genesis (cli.c5); deeper use lands with
    /// gently-ef5.
    #[command(flatten)]
    verbose: genesis::guide::CliVerbosity,
}

/// The rendered output, printed by `Output::emit`'s human path.
///
/// `Output::print` writes the payload through `Debug` plus `writeln!`'s
/// final newline, so `Debug` writes the bytes verbatim minus one trailing
/// newline to reproduce the oracle stream exactly (specs/cli.md c3).
#[derive(Serialize)]
struct RawAscii {
    bytes: String,
}

impl RawAscii {
    fn new(bytes: String) -> RawAscii {
        RawAscii { bytes }
    }
}

impl fmt::Debug for RawAscii {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body = self.bytes.strip_suffix('\n').unwrap_or(&self.bytes);
        f.write_str(body)
    }
}

/// A user-facing CLI failure (parse, render, or IO), reported through the
/// genesis `ErrorSink`.
#[derive(Debug)]
struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}

fn main() {
    let cli = Cli::parse();
    let guide = Guide::builder("gently", env!("CARGO_PKG_VERSION"))
        .with_verbosity(cli.verbose.verbosity())
        .build();

    // cli.c4: validate the output format before dispatch — a diagnostic
    // naming the requested and valid formats, then exit code 2.
    if let Some(fmt) = cli.format.as_deref() {
        if fmt != "ascii" {
            report_error(
                &guide,
                &format!("unknown output format '{fmt}' (valid formats: {VALID_FORMATS})"),
            );
            exit(2);
        }
    }

    // Dispatch the pipeline through the Guide; `Output::emit` (human mode in
    // this slice) prints the raw ascii to stdout, and handler errors go to
    // stderr with exit 1 (cli.c3). `--json` envelope mode lands with
    // gently-ef5.
    let code = guide.run_formatted(OutputFormat::Human, || {
        let source = read_source(&cli)?;
        let graph = text::parse(&source)
            .map_err(|e| CliError(format!("parse error, line {}: {}", e.line, e.message)))?;
        let lay = layout::layout(&graph);
        let art = ascii::render(&graph, &lay)
            .map_err(|e| CliError(format!("render error: {}", e.message)))?;
        Ok(Output::success(RawAscii::new(art)))
    });
    exit(code);
}

/// Read the graph source: stdin when no file arguments are given, otherwise
/// the files' contents concatenated in order (specs/cli.md c1).
fn read_source(cli: &Cli) -> Result<String, CliError> {
    if cli.files.is_empty() {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CliError(format!("cannot read stdin: {e}")))?;
        return Ok(buf);
    }
    let mut buf = String::new();
    for file in &cli.files {
        let content = std::fs::read_to_string(file)
            .map_err(|e| CliError(format!("cannot read {}: {e}", file.display())))?;
        buf.push_str(&content);
        if !content.ends_with('\n') {
            buf.push('\n');
        }
    }
    Ok(buf)
}

/// Print a diagnostic to stderr — and nowhere else (specs/cli.md c3).
///
/// The genesis `ErrorSink` prints `<tool>: <message>`; its scratch
/// persistence and suggestion/feedback footers are disabled in this slice
/// (`doctor`/`feedback` subcommands land with gently-ef5).
fn report_error(guide: &Guide, message: &str) {
    let sink = ErrorSink {
        tool_name: guide.name().to_string(),
        scratch: false,
        suggest: false,
        context: false,
        feedback_subcommand: None,
        verbosity: Verbosity::Normal,
    };
    sink.handle(&CliError(message.to_string()), &mut std::io::stderr());
}