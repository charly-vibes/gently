//! Integration tests for the `gently` binary — the end-to-end tracer
//! pipeline (epic gently-2po, tb.cli / gently-2po.9), wired to the cli
//! capability contracts (`scenarios::cli::p1`–`p4`, gently-0h9).
//!
//! Binding constraints: `specs/cli.md` c1 (stdin / positional
//! `[inputfile [outputfile]]`, extras ignored), c2 (`--as` format +
//! `--output` file), c3 (stdout-vs-stderr streams, exit 0/nonzero), c4
//! (unknown format → diagnostic naming requested and valid formats, exit
//! 255 — bug-for-bug with the upstream script's uncaught `die`, re-derived
//! under the ge.oracle.c7 scope contract, gently-ghh). Flag roles follow
//! upstream bin/graph-easy v0.69. The ascii oracle is pinned from
//! Graph::Easy v0.69 @ ededa3d7 (`add_edge("a","b"); as_ascii`).
//!
//! Deepened by gently-ef5 (--json envelope, verbosity surface, init/doctor)
//! and gently-bzx (full parser capability).
mod common;

/// Contract scenarios for `specs/cli.md`, one module per property row —
/// the filter `scenarios::cli::pN` used by `.espectacular/cli/pN.toml`
/// matches these test paths.
mod scenarios {
    #[path = "cli.rs"]
    mod cli;
}
