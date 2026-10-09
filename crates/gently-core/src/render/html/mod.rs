//! Purpose: the table-based HTML renderer — gently's port of upstream
//! Graph::Easy's `as_html`/`Easy.pm` (gently-eyo, specs/ge-html_render.md).
//! Responsibilities: consume the same `ge.layout` grid contract as
//! `render::ascii`/`render::boxart` and emit an HTML table mirroring the
//! grid, one `td` per grid cell, with CSS classes and inline styles
//! derived from the graph, node, and edge attributes (attribute remaps and
//! the W3C color scheme in `styles`, the CSS rule set in `css`).
//! Rationale: shaped by the probed pinned-oracle bytes
//! (tests/repro/claims/html-edge-styles.observed, html-css-rules.observed,
//! html-colors-shapes.observed, href-escaping.observed,
//! td-colspan.observed, shape-outline-collapse.observed). Gently's layout
//! grid is one cell per node (the oracle's 4×4 subcell machinery —
//! colspan/rowspan=4, filler `<tr></tr>` rows, `el` padding cells,
//! separate arrow tds — has no counterpart), so the oracle table shape is
//! followed where it maps and the divergences are documented per rule.

use crate::graph::Graph;
use crate::layout::Layout;
use crate::render::ascii::RenderError;

/// Render `graph` over its `layout` as the HTML table (upstream `as_html`).
pub fn render(_graph: &Graph, _layout: &Layout) -> Result<String, RenderError> {
    Err(RenderError {
        message: "html renderer not yet implemented (gently-eyo RED)".to_string(),
    })
}

/// Render `graph` over its `layout` as a self-contained HTML document: the
/// CSS rules every emitted class uses (upstream `css()`) embedded in a
/// `<style>` block ahead of the table (ge.html_render.c4).
pub fn document(_graph: &Graph, _layout: &Layout) -> Result<String, RenderError> {
    Err(RenderError {
        message: "html renderer not yet implemented (gently-eyo RED)".to_string(),
    })
}
