//! Purpose: `--from` input-format selection and the parse dispatch it
//! drives (specs/cli.md c9).
//! Responsibilities: resolve the effective parser choice from the flag
//! surface, validate it before the pipeline runs, and parse the source
//! with the chosen parser (sniffing txt-selected sources for DOT/VCG
//! headers via [`crate::pipeline::sniff`]).
//! Rationale: the boolean aliases override `--from` in upstream
//! get_options' loop order; `vcg`/`gdl` name the upstream VCG parser
//! gently never ports, so both selection and parse surface the scope-off
//! diagnostic instead of a parse attempt (gently-cij).

use gently_core::{
    graph::Graph,
    parse::{dot, text},
};

use crate::pipeline::sniff::{sniff, Sniffed};
use crate::{Cli, CliError};

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
    let raw = if cli.from_gdl {
        Some("gdl")
    } else if cli.from_vcg {
        Some("vcg")
    } else if cli.from_graphviz {
        Some("graphviz")
    } else if cli.from_txt {
        Some("txt")
    } else {
        cli.from_.as_deref()
    };
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
pub(super) fn parse_input(cli: &Cli, source: &str) -> Result<Graph, CliError> {
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
