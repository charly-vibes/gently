//! Purpose: the table-based HTML renderer — gently's port of upstream
//! Graph::Easy's `as_html` (gently-eyo, specs/ge-html_render.md).
//! Responsibilities: consume the same `ge.layout` grid contract as
//! `render::ascii`/`render::boxart` and emit an HTML table mirroring the
//! grid, one `td` per grid cell, with CSS classes and inline styles
//! derived from the graph, node, and edge attributes (remap tables in
//! `styles`, the rule set in `css`); `document` embeds the CSS rules for
//! every emitted class so the output is self-contained (ge.html_render.c4).
//! Rationale: shaped by the probed pinned-oracle bytes (tests/repro/
//! claims/html-*.observed, href-escaping.observed, td-colspan.observed,
//! shape-outline-collapse.observed). Gently's grid is one cell per node —
//! the oracle's 4×4 subcell machinery (colspan/rowspan=4, filler
//! `<tr></tr>` rows, `el` padding cells, separate arrow tds) has no
//! counterpart — so the observed table shape is followed where it maps,
//! with these documented divergences:
//! - one td per grid cell (the oracle spans nodes colspan=4 rowspan=4);
//! - an arrowhead cell keeps its line border (the oracle hosts arrows in
//!   their own borderless `eb` td), and label+arrow cells combine (the
//!   oracle separates them into a lh cell and an arrow cell);
//! - empty grid cells emit `<td></td>` (the oracle pads colspan=4
//!   rowspan=4 and pops trailing ones);
//! - `background` on a plain node is suppressed — only `fill` surfaces
//!   (observed);
//! - anonymous nodes render as plain `node` tds (the oracle carries a
//!   `node_anon` class — no probe pins it in gently's grid).

mod css;
mod styles;

use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::{Cell, Layout};
use crate::render::ascii::{display_width, RenderError};
use css::css;
use styles::{border_attribute_as_html, color_as_hex, escape_href, escape_text};

/// What occupies a grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Occupant {
    Node(usize),
    Edge { edge: usize, at: usize },
    Empty,
}

/// How a path cell connects to its neighbours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
    Corner,
}

/// Render `graph` over its `layout` as the HTML table (upstream `as_html`).
///
/// Byte-shaped by the probed pinned oracle (Graph::Easy v0.69 @ ededa3d7)
/// for the recorded skeleton and cell forms, adapted to gently's grid as
/// documented in the module header. The empty graph renders the bare
/// skeleton.
pub fn render(graph: &Graph, layout: &Layout) -> Result<String, RenderError> {
    let occupants = cell_map(layout);
    let mut out = String::from("\n\n<table class=\"graph\" cellpadding=0 cellspacing=0>\n");
    for row in 0..layout.height {
        out.push_str(&format!("<!-- row {row} line 0 -->\n<tr>\n"));
        for col in 0..layout.width {
            out.push_str(&cell_td(graph, layout, &occupants, (col, row)));
            out.push('\n');
        }
        out.push_str("</tr>\n\n");
    }
    out.push_str("</table>\n");
    Ok(out)
}

/// Render `graph` over its `layout` as a self-contained HTML document: the
/// CSS rules every emitted class uses (upstream `css()`) embedded in a
/// `<style>` block ahead of the table (ge.html_render.c4).
pub fn document(graph: &Graph, layout: &Layout) -> Result<String, RenderError> {
    let rules = css(graph.edges.iter().any(|_| true), !graph.groups.is_empty(), has_rounded(graph));
    let mut out = format!("<style type=\"text/css\">\n<!--\n{rules}-->\n</style>\n");
    out.push_str(&render(graph, layout)?);
    Ok(out)
}

/// Any node shaped rounded, circle, or ellipse (the rounded-shape CSS
/// block's condition, upstream `css()`).
fn has_rounded(graph: &Graph) -> bool {
    graph.nodes.iter().any(|n| {
        matches!(
            shape_of(graph, n),
            Some("rounded") | Some("circle") | Some("ellipse")
        )
    })
}

/// A node's `shape` attribute (per-object over the base class), `None`
/// when unset.
fn shape_of<'a>(graph: &'a Graph, node: &'a crate::graph::Node) -> Option<&'a str> {
    node.attributes
        .get("shape")
        .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Node, String::new()), "shape"))
}

/// Map every grid cell to its occupant: nodes first (one cell each), then
/// edge path cells in edge-index order (a cell two edges share keeps the
/// first claim — the grid emits exactly one td per cell).
fn cell_map(layout: &Layout) -> Vec<Occupant> {
    let mut cells = vec![Occupant::Empty; layout.width * layout.height];
    let put = |cells: &mut Vec<Occupant>, c: Cell, occ: Occupant| {
        if c.0 < layout.width && c.1 < layout.height {
            let slot = &mut cells[c.1 * layout.width + c.0];
            if *slot == Occupant::Empty {
                *slot = occ;
            }
        }
    };
    for (i, &c) in layout.node_cells.iter().enumerate() {
        put(&mut cells, c, Occupant::Node(i));
    }
    for (edge, path) in layout.edge_paths.iter().enumerate() {
        for (at, &c) in path.iter().enumerate() {
            put(&mut cells, c, Occupant::Edge { edge, at });
        }
    }
    cells
}

/// The td of one grid cell.
fn cell_td(graph: &Graph, layout: &Layout, occupants: &[Occupant], c: Cell) -> String {
    match occupants[c.1 * layout.width + c.0] {
        Occupant::Node(i) => node_td(graph, i),
        Occupant::Edge { edge, at } => edge_td(graph, layout, edge, at),
        Occupant::Empty => " <td></td>".to_string(),
    }
}

/// The label LINES of `node`: the `label` attribute over the name.
fn node_label_lines<'a>(graph: &'a Graph, node: &'a crate::graph::Node) -> Vec<&'a str> {
    let label = node
        .attributes
        .get("label")
        .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Node, String::new()), "label"))
        .unwrap_or(&node.name);
    label.split('\n').collect()
}

/// The td of node `i` (ge.html_render.c2/c5): dispatch by shape —
/// invisible → classless empty cell, point → star glyph, rounded/
/// circle/ellipse → inner-div template, else the plain node td.
fn node_td(graph: &Graph, i: usize) -> String {
    let node = &graph.nodes[i];
    let shape = shape_of(graph, node);
    match (shape, rounded_div_class(shape)) {
        (Some("invisible"), _) => {
            " <td style=\"border: none; background: inherit;\"></td>".to_string()
        }
        (Some("point"), _) => {
            " <td class='node' style=\"background: inherit; border: none\">\u{2605}</td>".to_string()
        }
        (_, Some(div_class)) => rounded_node_td(graph, node, div_class),
        _ => plain_node_td(graph, node),
    }
}

/// The label of `node`: its `label` attribute over the node name (upstream
/// `label()` defaults to the name), each line escaped, joined with `<br>`
/// (upstream _label_as_html escapes line-wise).
fn node_label_text(graph: &Graph, node: &crate::graph::Node) -> String {
    let label = node
        .attributes
        .get("label")
        .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Node, String::new()), "label"))
        .unwrap_or(&node.name);
    label
        .split('\n')
        .map(escape_text)
        .collect::<Vec<_>>()
        .join("<br>")
}

/// The td of a rounded/circle/ellipse node: the border moves onto the
/// inner div (observed template).
fn rounded_node_td(graph: &Graph, node: &crate::graph::Node, div_class: &str) -> String {
    let lines = node_label_lines(graph, node);
    let (w, h) = (
        lines.iter().map(|l| display_width(l)).max().unwrap_or(0),
        lines.len(),
    );
    let label = node_label_text(graph, node);
    let fill = node
        .attributes
        .get("fill")
        .map(color_as_hex)
        .unwrap_or_else(|| "#ffffff".to_string());
    let (bs, bw, bc) = border_components(graph, node);
    let border = border_attribute_as_html(&bs, &bw, &bc);
    let top = h as f64 / 2.0 + 0.5;
    let span_top = if (top - 1.5).abs() < f64::EPSILON {
        String::new()
    } else {
        format!(" style=\"top: {top}em\"")
    };
    let inner_label = node
        .attributes
        .get("link")
        .map(escape_href)
        .map_or_else(|| label.clone(), |href| format!("<a href='{href}'>{label}</a>"));
    format!(
        " <td class='node' style=\"border: none;background: inherit\"><div class='{div_class}' style='background:{fill};border:{border};width: {}em; height: {}em'><span class='c'{span_top}>{inner_label}</span></div></td>",
        w + 2,
        h + 2
    )
}

/// The td of a plain node: fill → background, color → color, border
/// components → border (suppressed when default; `background` suppressed
/// — observed); the link wraps the label unless the label is empty.
fn plain_node_td(graph: &Graph, node: &crate::graph::Node) -> String {
    let label = node_label_text(graph, node);
    let mut parts: Vec<String> = Vec::new();
    if let Some(fill) = node.attributes.get("fill") {
        parts.push(format!("background: {}", color_as_hex(fill)));
    }
    let (bs, bw, bc) = border_components(graph, node);
    let border = border_attribute_as_html(&bs, &bw, &bc);
    if !border.is_empty() && border != "solid 1px #000000" {
        parts.push(format!("border: {border}"));
    }
    if let Some(color) = node.attributes.get("color") {
        parts.push(format!("color: {}", color_as_hex(color)));
    }
    parts.sort();
    let style = parts.join("; ");
    let style = if style.is_empty() { String::new() } else { format!(" style=\"{style}\"") };
    let content = match node.attributes.get("link") {
        Some(href) if !label.is_empty() => format!("<a href='{}'>{label}</a>", escape_href(href)),
        _ => label,
    };
    format!(" <td class='node'{style}>{content}</td>")
}

/// The border components of a node: the derived border_style/width/color
/// attributes, empty components over the upstream defaults (solid, 1,
/// #000000 — a node's default border width is 1px).
fn border_components(graph: &Graph, node: &crate::graph::Node) -> (String, String, String) {
    let val = |key: &str, default: &str| -> String {
        node.attributes
            .get(key)
            .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Node, String::new()), key))
            .filter(|v| !v.is_empty())
            .unwrap_or(default)
            .to_string()
    };
    (val("border_style", "solid"), val("border_width", "1"), val("border_color", "#000000"))
}

/// The inner div class of a rounded shape: `r` for rounded, `c` for
/// circle/ellipse (upstream: the shape's first letter, e→c), `None` for
/// all other shapes (they leave the plain node td unchanged — observed).
fn rounded_div_class(shape: Option<&str>) -> Option<&'static str> {
    match shape {
        Some("rounded") => Some("r"),
        Some("circle") | Some("ellipse") => Some("c"),
        _ => None,
    }
}

/// The td of edge `edge`'s path cell `at` (ge.html_render.c3).
fn edge_td(graph: &Graph, layout: &Layout, edge: usize, at: usize) -> String {
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
    let hex = color.map(color_as_hex).unwrap_or_else(|| "#000000".to_string());
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
        format!("{}{spans}", if labelled { cell_label(graph, edge) } else { String::new() })
    } else if labelled {
        cell_label(graph, edge)
    } else {
        "&nbsp;".to_string()
    }
}

/// The `style` attribute of edge `edge`: the per-object attribute over
/// the base edge class, defaulting to solid.
fn edge_style(graph: &Graph, edge: usize) -> String {
    graph
        .get_attr(Scope::Edge(edge), "style")
        .or_else(|| graph.get_attr(Scope::Class(ObjectKind::Edge, String::new()), "style"))
        .filter(|v| !v.is_empty())
        .unwrap_or("solid")
        .to_string()
}

/// The arrowhead spans landing on path cell `at`, pointing at the node
/// the arrowhead belongs to.
fn arrow_spans(graph: &Graph, layout: &Layout, edge: usize, at: usize, c: Cell) -> String {
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
fn cell_label(graph: &Graph, edge: usize) -> String {
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
fn orientation_of(path: &[Cell], at: usize, graph: &Graph, layout: &Layout, edge: usize) -> Orientation {
    let c = path[at];
    let mut horizontal = false;
    let mut vertical = false;
    if at > 0 {
        note_axis(path[at - 1], c, &mut horizontal, &mut vertical);
    }
    if at + 1 < path.len() {
        note_axis(path[at + 1], c, &mut horizontal, &mut vertical);
    }
    if at == 0 {
        note_axis(layout.node_cells[graph.edges[edge].from], c, &mut horizontal, &mut vertical);
    }
    if at + 1 == path.len() {
        note_axis(layout.node_cells[graph.edges[edge].to], c, &mut horizontal, &mut vertical);
    }
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
