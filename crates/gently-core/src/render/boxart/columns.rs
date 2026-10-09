//! Purpose: the char-grid column mapping of the boxart renderer — per-node
//! shape/border resolution and per-column widths with the upstream
//! edge-cell width rules.
//! Responsibilities: resolve per-node (border style, shape) and per-edge
//! style strings (per-object attributes over class ones), compute node
//! box widths, and derive column widths: node columns take the max of
//! their boxes' natural width and the edge cells routed through them, gap
//! columns the 5-char floor widened by straight-edge cells (style bonus +
//! label width).
//! Rationale: split from canvas.rs past the tidy file limit; the width
//! rules come from upstream `Edge::Cell::_correct_size` and the probe
//! round (tests/repro/claims/boxart-render-tables.observed — gently-css).

use super::styles;
use crate::render::ascii::display_width;
use super::canvas::GAP_WIDTH;
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::Layout;

/// How a node shape renders (ge.boxart_render.c4 — the probed oracle
/// collapses every non-special shape to the plain box; `rounded` overlays
/// or blanks corners per border style, `point` swaps the label for a
/// centered ★, `invisible` draws nothing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shape {
    Box,
    Rounded,
    Point,
    Invisible,
}

/// Resolve a node's `shape` attribute to its render class.
fn shape(attr: Option<&str>) -> Shape {
    match attr {
        Some("rounded") => Shape::Rounded,
        Some("point") => Shape::Point,
        Some("invisible") => Shape::Invisible,
        _ => Shape::Box,
    }
}

/// The value of `key` on the object at `scope`, falling back to the base
/// class of `kind` (per-object attributes win over class attributes).
pub(super) fn object_attr<'a>(
    graph: &'a Graph,
    scope: Scope,
    kind: ObjectKind,
    key: &str,
) -> Option<&'a str> {
    graph
        .get_attr(scope, key)
        .or_else(|| graph.get_attr(Scope::Class(kind, String::new()), key))
}

/// Per-node (border style, shape), per-object attributes over class ones.
pub(super) fn node_styles(graph: &Graph) -> Vec<(String, Shape)> {
    (0..graph.nodes.len())
        .map(|i| {
            let scope = Scope::Node(i);
            let style = object_attr(graph, scope.clone(), ObjectKind::Node, "border_style")
                .unwrap_or("solid")
                .to_string();
            (style, shape(object_attr(graph, scope, ObjectKind::Node, "shape")))
        })
        .collect()
}

/// Per-edge style string, per-object attributes over class ones.
pub(super) fn edge_styles(graph: &Graph) -> Vec<String> {
    (0..graph.edges.len())
        .map(|i| {
            object_attr(graph, Scope::Edge(i), ObjectKind::Edge, "style")
                .unwrap_or("solid")
                .to_string()
        })
        .collect()
}

/// Box width of a node: display width of the label (double-width glyphs
/// count two columns) plus two border chars per side — or one per side
/// for a `none` border — widened for selfloop nodes so the loop elbow
/// fits above the box (the upstream loop cell is 7 wide plus its arrow).
fn box_width(name: &str, style: &str, selfloop: bool) -> usize {
    let pad = if style == "none" { 1 } else { 2 };
    let w = display_width(name) + 2 * pad;
    if selfloop {
        w.max(8)
    } else {
        w
    }
}

/// The natural node-column width per the char-grid mapping: the widest
/// box placed in the column.
fn node_widths(graph: &Graph, layout: &Layout, node_style: &[(String, Shape)]) -> Vec<Option<usize>> {
    let mut node_width = vec![0usize; layout.width];
    let mut node_col = vec![false; layout.width];
    for (idx, &(cx, _)) in layout.node_cells.iter().enumerate() {
        let (style, _) = &node_style[idx];
        let selfloop = graph.edges.iter().any(|e| e.from == idx && e.to == idx);
        let w = box_width(&graph.nodes[idx].name, style, selfloop);
        node_width[cx] = node_width[cx].max(w);
        node_col[cx] = true;
    }
    (0..layout.width)
        .map(|c| if node_col[c] { Some(node_width[c]) } else { None })
        .collect()
}

/// Widen `widths` by the edge-cell width each edge contributes: a selfloop
/// cell to its node's column (7 wide plus its arrowhead), a straight cell
/// to its gap column (5 plus style bonus and label width), and a bend's
/// routed cells to every column they pass through (each hosts an edge cell
/// of at least `5 + bonus` — probed: the dot-dot-dash cycle renders
/// 6-wide boxes and gap).
fn edge_cell_widths(graph: &Graph, layout: &Layout, widths: &mut [Option<usize>]) {
    for (ei, path) in layout.edge_paths.iter().enumerate() {
        let style = edge_styles(graph)[ei].clone();
        let bidi = graph.edges[ei].arrows.start && graph.edges[ei].arrows.end;
        let bonus = styles::cell_bonus(&style, bidi);
        let edge = &graph.edges[ei];
        let (sc, sr) = layout.node_cells[edge.from];
        let (tc, tr) = layout.node_cells[edge.to];
        if sc == tc && sr == tr {
            widen(widths, sc, 8);
        } else if sr == tr && path.len() == 1 && sc.abs_diff(tc) == 2 {
            let base = GAP_WIDTH + bonus;
            let w = match object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "label") {
                Some(label) => display_width(label) + base,
                None => base,
            };
            widen(widths, path[0].0, w);
        } else {
            for &(pc, _) in path {
                widen(widths, pc, GAP_WIDTH + bonus);
            }
        }
    }
}

/// Raise column `c`'s width to at least `w`.
fn widen(widths: &mut [Option<usize>], c: usize, w: usize) {
    widths[c] = Some(widths[c].map_or(w, |old| old.max(w)));
}

/// Column widths per the char-grid mapping and their prefix offsets.
pub(super) fn column_geometry(
    graph: &Graph,
    layout: &Layout,
    node_style: &[(String, Shape)],
) -> (Vec<usize>, Vec<usize>) {
    let mut widths = node_widths(graph, layout, node_style);
    edge_cell_widths(graph, layout, &mut widths);
    let col_width: Vec<usize> = widths
        .into_iter()
        .map(|w| w.unwrap_or(GAP_WIDTH))
        .collect();
    let mut offsets = vec![0usize; layout.width + 1];
    for c in 0..layout.width {
        offsets[c + 1] = offsets[c] + col_width[c];
    }
    (col_width, offsets)
}