//! Purpose: the Unicode boxart renderer — gently's port of upstream
//! Graph::Easy's `as_boxart`/`As_ascii.pm` with `_ascii_style = 1`
//! (gently-css, specs/ge-boxart_render.md).
//! Responsibilities: consume the same `ge.layout` grid contract as
//! `render::ascii` and draw the laid-out graph with Unicode box-drawing
//! glyphs selected by the probed tables (tests/repro/claims/
//! boxart-render-tables.observed).
//! Rationale: RED skeleton — the scenario contracts p1–p4 are written
//! first and must fail against this stub until the renderer lands.

use crate::graph::Graph;
use crate::layout::Layout;
use crate::render::ascii::RenderError;

/// Render `graph` over its `layout` as Unicode boxart.
pub fn render(_graph: &Graph, _layout: &Layout) -> Result<String, RenderError> {
    Err(RenderError {
        message: "boxart renderer not yet implemented (gently-css RED)".to_string(),
    })
}