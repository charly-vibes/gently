//! Purpose: the `gently` render pipeline — parse, lay out, render, and
//! emit through the genesis `Guide` (specs/cli.md c1–c6; p1–p4 in
//! gently-0h9, `--json` envelope in gently-ef5).
//! Responsibilities: module root re-exporting the pipeline surface —
//! input-format selection lives in `input`, content sniffing in `sniff`,
//! source reading in `source`, and render/emit dispatch in `render`.
//! Rationale: human mode keeps `Output::emit`'s `Debug`-print path
//! (`RawAscii` writes the rendered bytes verbatim minus exactly one
//! trailing newline) so the byte-compat oracle stream of p1–p4 is
//! preserved; `--json` rides `Output::emit`'s envelope path so failures
//! come out typed (`envelope_kind: "error"`, c6); `--output` suppresses
//! stdout via an unreachable verbosity threshold, bug-for-bug with
//! upstream writing nothing to stdout when a file receives the bytes.

mod input;
mod render;
mod sniff;
mod source;

pub use input::check_input_format;
pub use render::run;

use std::path::Path;

use crate::Cli;

/// Output formats wired in this slice (specs/cli.md c2): ascii, html,
/// graphviz (`dot` aliases it upstream) and txt — each routed to its
/// library renderer. `boxart`/`svg`/`vcg`/`gdl`/`graphml` stay exit-255
/// (c4): upstream v0.69 has no `as_boxart`/`as_svg` method, and the
/// vcg/gdl/graphml writers are documented scope-offs (gently-4ig).
pub const SUPPORTED_FORMATS: &[&str] = &["ascii", "html", "graphviz", "txt"];

/// Upstream's extension → format map (bin/graph-easy get_options): the
/// output filename infers the format when `--as` is absent. `dot` is an
/// alias for `graphviz`.
pub(crate) const EXTENSION_FORMATS: &[(&str, &str)] = &[
    ("html", "html"),
    ("svg", "svg"),
    ("txt", "txt"),
    ("dot", "graphviz"),
    ("vcg", "vcg"),
    ("gdl", "gdl"),
    ("graphml", "graphml"),
];

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
