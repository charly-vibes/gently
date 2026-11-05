//! Purpose: the edge-cell emitters for ge.html_render (gently-eyo) —
//! per-path-position `<td>` construction: connector orientation classes,
//! border side styles, arrow spans, and in-cell edge labels.
//! Rationale: mirrors the upstream HTML driver's cell decomposition so
//! the pinned html fixture corpus renders byte-identically.

use super::*;

/// The td of edge `edge`'s path cell `at` (ge.html_render.c3).
pub(super) fn edge_td(graph: &Graph, layout: &Layout, edge: usize, at: usize) -> String {
    let path = &layout.edge_paths[edge];
    let c = path[at];
    let (class, sides) = match orientation_of(path, at, graph, layout, edge) {
        Orientation::Horizontal => ("lh", "border-bottom"),
        Orientation::Vertical => ("lv", "border-left"),
        Orientation::Corner => ("eb", "border-bottom; border-left"),
    };
    // the border value: the edge style through the observed remap, with
    // the edge color as its color (default #000000)
    let color = graph
        .get_attr(Scope::Edge(edge), "color")
        .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Edge, String::new()), "color"))
        .filter(|v| !v.is_empty());
    let hex = color
        .map(color_as_hex)
        .unwrap_or_else(|| "#000000".to_string());
    let border = border_attribute_as_html(&edge_style(graph, edge), "2", &hex);
    let spans = arrow_spans(graph, layout, edge, at, c);
    let labelled = layout.label_cells[edge] == Some(c);
    let style = edge_style_attr(sides, &border, &hex, color.is_some() || !spans.is_empty());
    let content = edge_content(graph, edge, labelled, &spans);
    format!(" <td class=\"edge {class}\" style=\"{style}\">{content}</td>")
}

/// The style attribute of an edge cell: the border declarations per side,
/// then the color declaration when the edge carries an explicit color or
/// hosts an arrowhead (the observed arrow td always carries
/// `color: #000000;`).
fn edge_style_attr(sides: &str, border: &str, hex: &str, colorize: bool) -> String {
    let mut style = sides
        .split("; ")
        .map(|side| format!("{side}: {border};"))
        .collect::<Vec<_>>()
        .join(" ");
    if colorize {
        style.push_str(&format!("color: {hex};"));
    }
    style
}

/// The content of an edge cell: the arrowhead spans (after the edge
/// label when this is the label cell), else the edge label on its label
/// cell, else the `&nbsp;` placeholder (upstream keeps it for the cell
/// borders).
fn edge_content(graph: &Graph, edge: usize, labelled: bool, spans: &str) -> String {
    if !spans.is_empty() {
        format!(
            "{}{spans}",
            if labelled {
                cell_label(graph, edge)
            } else {
                String::new()
            }
        )
    } else if labelled {
        cell_label(graph, edge)
    } else {
        "&nbsp;".to_string()
    }
}

/// The `style` attribute of edge `edge`: the per-object attribute over
/// the base edge class, defaulting to solid.
pub(super) fn edge_style(graph: &Graph, edge: usize) -> String {
    graph
        .get_attr(Scope::Edge(edge), "style")
        .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Edge, String::new()), "style"))
        .filter(|v| !v.is_empty())
        .unwrap_or("solid")
        .to_string()
}

/// The arrowhead spans landing on path cell `at`, pointing at the node
/// the arrowhead belongs to.
pub(super) fn arrow_spans(
    graph: &Graph,
    layout: &Layout,
    edge: usize,
    at: usize,
    c: Cell,
) -> String {
    let edge_obj = &graph.edges[edge];
    let mut spans = String::new();
    if edge_obj.arrows.start && at == 0 {
        spans.push_str(&arrow_span(layout, c, edge_obj.from));
    }
    if edge_obj.arrows.end && at + 1 == layout.edge_paths[edge].len() {
        spans.push_str(&arrow_span(layout, c, edge_obj.to));
    }
    spans
}

/// The escaped text label of edge `edge`, on its label cell.
pub(super) fn cell_label(graph: &Graph, edge: usize) -> String {
    graph
        .get_attr(Scope::Edge(edge), "label")
        .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Edge, String::new()), "label"))
        .map(|label| escape_text(label).replace('\n', "<br>"))
        .unwrap_or_default()
}

/// The arrowhead span pointing from cell `c` at `node`.
fn arrow_span(layout: &Layout, c: Cell, node: usize) -> String {
    let node_cell = layout.node_cells[node];
    let (class, glyph) = match (node_cell.0.cmp(&c.0), node_cell.1.cmp(&c.1)) {
        (std::cmp::Ordering::Greater, _) => ("sh", ">"),
        (std::cmp::Ordering::Less, _) => ("shl", "<"),
        (_, std::cmp::Ordering::Greater) => ("sv", "\u{2228}"),
        _ => ("su", "\u{2227}"),
    };
    format!("<span class=\"{class}\">{glyph}</span>")
}

/// How path cell `at` connects: horizontal, vertical, or corner (both) —
/// from its path neighbours and, at the ends, the adjacent node.
fn orientation_of(
    path: &[Cell],
    at: usize,
    graph: &Graph,
    layout: &Layout,
    edge: usize,
) -> Orientation {
    let c = path[at];
    let (mut horizontal, mut vertical) = (false, false);
    let neighbors = neighbor_cells(path, at, graph, layout, edge);
    neighbors
        .iter()
        .for_each(|&cell| note_axis(cell, c, &mut horizontal, &mut vertical));
    axis_orientation(horizontal, vertical)
}

/// The cells whose axis toward `path[at]` decides the connector's
/// orientation: the path neighbors, or the edge's endpoint cell at the
/// path ends.
fn neighbor_cells(
    path: &[Cell],
    at: usize,
    graph: &Graph,
    layout: &Layout,
    edge: usize,
) -> Vec<Cell> {
    let mut cells = Vec::new();
    if at > 0 {
        cells.push(path[at - 1]);
    }
    if at + 1 < path.len() {
        cells.push(path[at + 1]);
    }
    if at == 0 {
        cells.push(layout.node_cells[graph.edges[edge].from]);
    }
    if at + 1 == path.len() {
        cells.push(layout.node_cells[graph.edges[edge].to]);
    }
    cells
}

fn axis_orientation(horizontal: bool, vertical: bool) -> Orientation {
    match (horizontal, vertical) {
        (true, true) => Orientation::Corner,
        (false, true) => Orientation::Vertical,
        _ => Orientation::Horizontal,
    }
}

/// Record the axis `from`→`to` lies on.
fn note_axis(from: Cell, to: Cell, horizontal: &mut bool, vertical: &mut bool) {
    if from.0 == to.0 {
        *vertical = true;
    }
    if from.1 == to.1 {
        *horizontal = true;
    }
}
