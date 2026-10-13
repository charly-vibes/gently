//! Purpose: gallery generator — emit the library's Graphviz DOT render for
//! one example input, piped through `dot -Tsvg` by `just gallery-verify` to
//! produce the committed docs/gallery/<name>.svg artifacts (gently-irk).
//! Responsibilities: read the graph source file given as the sole argument,
//! parse it with the text parser (every published example uses the text
//! syntax), render it via `render::graphviz::render`, and write the DOT to
//! stdout — nothing else; `dot` lays out its own graph, so the layout grid
//! is not involved.
//! Rationale: a standalone cargo example (not a CLI flag) because the CLI's
//! `--as graphviz` wiring is a later capability slice (gently-4ig); once
//! that lands this example MAY be retired in favor of `gently --as dot`,
//! with the committed SVGs and the byte gate invariant either way.

use std::fs;
use std::process::ExitCode;

use gently_core::{parse::text, render::graphviz};

fn main() -> ExitCode {
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: gallery <input.gv>  (writes DOT source to stdout)");
            return ExitCode::from(2);
        }
    };
    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("gallery: cannot read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    let graph = match text::parse(&source) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("gallery: parse error in {path}: {e}");
            return ExitCode::from(2);
        }
    };
    print!("{}", graphviz::render(&graph));
    ExitCode::SUCCESS
}
