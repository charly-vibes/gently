//! Purpose: the Unicode boxart renderer — gently's port of upstream
//! Graph::Easy's `as_boxart`/`As_ascii.pm` with `_ascii_style = 1`
//! (gently-css, specs/ge-boxart_render.md).
//! Responsibilities: consume the same `ge.layout` grid contract as
//! `render::ascii` and draw the laid-out graph with Unicode box-drawing
//! glyphs so borders and edge lines join seamlessly — glyph tables in
//! `styles`, char-grid canvas in `canvas`, bend polylines in `polyline`;
//! return a typed `RenderError` for unsupported geometry — never panic.
//! Rationale: a port of upstream's shared ascii/boxart render path shaped
//! byte-for-byte by the probed tables (tests/repro/claims/
//! boxart-render-tables.observed). Kept as a parallel module beside
//! `render::ascii` rather than unified: the ascii renderer is a closed,
//! corpus-pinned capability whose simplified cell model differs from the
//! boxart one (phase anchoring, attachment columns, column widths).

mod canvas;
mod columns;
mod polyline;
mod styles;

use crate::graph::Graph;
use crate::layout::Layout;
use crate::render::ascii::RenderError;
use canvas::Canvas;

/// Render `graph` over its `layout` as Unicode boxart.
///
/// Byte-identical to the pinned oracle (Graph::Easy v0.69 @ ededa3d7,
/// `as_boxart`) for the probed neighbourhoods, with one trailing newline.
/// The empty graph renders as the empty string.
pub fn render(graph: &Graph, layout: &Layout) -> Result<String, RenderError> {
    let mut canvas = Canvas::new(graph, layout);
    canvas.draw_nodes(graph, layout);
    for edge_index in 0..graph.edges.len() {
        canvas.draw_edge(graph, layout, edge_index)?;
    }
    Ok(canvas.into_string())
}