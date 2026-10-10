//! Purpose: the `gently` command-line entry point on the genesis-vibes
//! foundation — command dispatch via `Guide`, output via `Output::emit`,
//! verbosity via `CliVerbosity`, format via `CliFormat` (specs/cli.md
//! c5–c8; p1–p4 pipeline in gently-0h9, genesis surface in gently-ef5).
//! Responsibilities: parse the clap surface, precheck unknown
//! subcommands/flags for genesis suggestions (c7, [`suggest`]), route
//! `init`/`doctor` (c5/c8, [`doctor`]) or run the render pipeline
//! (c1–c4/c6, [`pipeline`]), and enforce the exit-code contract
//! (0 success, 1 user-facing errors, 2 unknown names, 255 unknown format).
//! Rationale: `main` stays a thin dispatcher — the surfaces live in
//! modules so the pretender file/function limits hold and each concern is
//! testable; human mode keeps the byte-compat oracle stream, `--json` the
//! genesis Envelope.

mod doctor;
mod pipeline;
mod suggest;

use std::fmt;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use genesis::guide::{ErrorSink, Guide, OutputFormat, Verbosity};

use pipeline::SUPPORTED_FORMATS;

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

    /// Verbosity plumbing from genesis (cli.c5): `--quiet` suppresses
    /// non-error output, `-v` adds verbose diagnostics on stderr.
    #[command(flatten)]
    verbose: genesis::guide::CliVerbosity,

    /// Output-format plumbing from genesis (cli.c5/c6): `--json` emits the
    /// genesis Envelope, `--human` forces the raw-bytes stream.
    #[command(flatten)]
    format: genesis::guide::CliFormat,

    /// Apply available auto-fixes (`gently doctor --fix`, cli.c8).
    #[arg(long)]
    fix: bool,
}

/// A user-facing CLI failure (parse, render, or IO), reported through the
/// genesis `ErrorSink`.
#[derive(Debug)]
pub struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}

fn main() {
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    let subcommand = suggest::extract_subcommand(&mut argv).map(|s| s.to_string());

    // cli.c7: unknown subcommand/flag → genesis suggestion on stderr
    // before the nonzero exit (2 = clap's invalid-usage code).
    if let Some(exit_code) = suggest::precheck_known_names(&argv) {
        exit(exit_code);
    }

    let cli = match Cli::try_parse_from(
        std::iter::once("gently".to_string()).chain(argv.iter().cloned()),
    ) {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            exit(2);
        }
    };
    let guide = Guide::builder("gently", env!("CARGO_PKG_VERSION"))
        .with_verbosity(cli.verbose.verbosity())
        .build();
    exit(dispatch(&cli, &guide, subcommand.as_deref()));
}

/// Route the parsed invocation: `init`/`doctor` subcommands (c5/c8) or the
/// render pipeline (c1–c4/c6), enforcing the format-resolution (c2) and
/// unknown-format-255 (c4) contracts on the pipeline path.
fn dispatch(cli: &Cli, guide: &Guide, subcommand: Option<&str>) -> i32 {
    let format = resolve_emit_format(cli);
    match subcommand {
        Some("init") => return doctor::run_init(guide, format),
        Some("doctor") => return doctor::run_doctor(guide, cli.fix, format),
        _ => {}
    }

    // cli.c2: resolve the output format — `--as` wins; otherwise the output
    // filename's extension (upstream's map); otherwise ascii.
    let output_path = cli.output.clone().or_else(|| cli.files.get(1).cloned());
    let format_name = pipeline::resolve_format(cli, output_path.as_deref());

    // cli.c4: validate the resolved format before dispatch — a diagnostic
    // naming the requested and valid formats, then exit 255 (bug-for-bug
    // with the upstream script's uncaught die, re-derived gently-0h9).
    if !SUPPORTED_FORMATS.contains(&format_name.as_str()) {
        report_error(
            guide,
            &format!(
                "unknown output format '{format_name}' (valid formats: {})",
                SUPPORTED_FORMATS.join(", ")
            ),
        );
        return 255;
    }
    pipeline::run(guide, cli, output_path, format)
}

/// The emit format (c5/c6): `--json` wraps results in the genesis
/// Envelope, everything else — including piped stdout — stays the raw
/// human byte stream that the oracle comparison requires.
fn resolve_emit_format(cli: &Cli) -> OutputFormat {
    if cli.format.json {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    }
}

/// Print a diagnostic to stderr — and nowhere else (specs/cli.md c3).
///
/// The genesis `ErrorSink` prints `<tool>: <message>`; its scratch
/// persistence and suggestion/feedback footers are disabled here so the
/// stream stays exactly the oracle-era diagnostic.
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
