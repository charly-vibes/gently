//! Purpose: the `gently` command-line entry point — the end-to-end tracer
//! pipeline (epic gently-2po, tb.cli / gently-2po.9).
//! Responsibilities: read graph text from the first file argument (stdin
//! when absent), parse it, lay it out, render the requested format, and
//! write the raw oracle bytes to stdout or the `--output` file; diagnostics
//! go to stderr; exit 0 on success, 1 on parse/render/IO errors, and 255 on
//! an unknown output format — bug-for-bug with the upstream script's
//! uncaught `die` (re-derived against bin/graph-easy v0.69 under the
//! ge.oracle.c7 scope contract; specs/cli.md c1–c4, gently-0h9).
//! Dispatch runs on the genesis-vibes `Guide` frame (cli.c5 foundation):
//! clap args carry `CliVerbosity`, the pipeline handler returns `Output<T>`
//! and is emitted through `Output::emit`.
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
const SUPPORTED_FORMATS: &[&str] = &["ascii"];

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

#[derive(Parser)]
#[command(
    name = "gently",
    version,
    about = "Graph text in, rendered output out"
)]
struct Cli {
    /// Positional arguments: `[inputfile [outputfile ...]]` — the first is
    /// the input, the second names the output file, further arguments are
    /// silently ignored, bug-for-bug with upstream (specs/cli.md c1,
    /// re-derived against bin/graph-easy v0.69).
    files: Vec<PathBuf>,

    /// Output format (upstream `--as <fmt>`; `dot` aliases graphviz).
    /// Defaults to ascii, or the output extension's format when `--output`
    /// names a mapped extension (specs/cli.md c2).
    #[arg(long = "as")]
    as_: Option<String>,

    /// Output filename (upstream `--output <file>`); stdout when absent
    /// (specs/cli.md c2).
    #[arg(long)]
    output: Option<PathBuf>,

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

    // cli.c2: resolve the output format — `--as` wins; otherwise the output
    // filename's extension (upstream's map); otherwise ascii.
    let output_path = cli.output.clone().or_else(|| cli.files.get(1).cloned());
    let format = resolve_format(&cli, output_path.as_deref());

    // cli.c4: validate the resolved format before dispatch — a diagnostic
    // naming the requested and valid formats, then exit 255 (bug-for-bug
    // with the upstream script's uncaught die, re-derived gently-0h9).
    if !SUPPORTED_FORMATS.contains(&format.as_str()) {
        report_error(
            &guide,
            &format!(
                "unknown output format '{format}' (valid formats: {})",
                SUPPORTED_FORMATS.join(", ")
            ),
        );
        exit(255);
    }
    exit(run_pipeline(&guide, &cli, output_path));
}

/// Dispatch the pipeline through the Guide (cli.c3/c5): `Output::emit`
/// (human mode in this slice) prints the raw ascii to stdout — or the
/// rendered bytes go to the `--output` file (cli.c2). Handler errors go to
/// stderr with exit 1. `--json` envelope mode lands with gently-ef5.
fn run_pipeline(guide: &Guide, cli: &Cli, output_path: Option<PathBuf>) -> i32 {
    guide.run_formatted(OutputFormat::Human, || {
        let source = read_source(cli)?;
        let graph = text::parse(&source)
            .map_err(|e| CliError(format!("parse error, line {}: {}", e.line, e.message)))?;
        let lay = layout::layout(&graph);
        let art = ascii::render(&graph, &lay)
            .map_err(|e| CliError(format!("render error: {}", e.message)))?;
        if let Some(path) = &output_path {
            std::fs::write(path, &art)
                .map_err(|e| CliError(format!("cannot write to {}: {e}", path.display())))?;
            // cli.c2: when the rendered bytes go to the `--output` file,
            // nothing is printed to stdout (upstream bug-for-bug) — suppress
            // the human emit path via an unreachable verbosity threshold.
            return Ok(Output::success(RawAscii::new(String::new())).with_verbosity(u8::MAX));
        }
        Ok(Output::success(RawAscii::new(art)))
    })
}

/// Resolve the output format per upstream bin/graph-easy (specs/cli.md c2):
/// `--as` wins (with `dot` aliasing `graphviz`); else the output filename's
/// extension via the upstream map; else the ascii default.
fn resolve_format(cli: &Cli, output_path: Option<&std::path::Path>) -> String {
    if let Some(fmt) = cli.as_.as_deref() {
        return if fmt == "dot" { "graphviz".to_string() } else { fmt.to_string() };
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
    let Some(file) = cli.files.first() else {
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