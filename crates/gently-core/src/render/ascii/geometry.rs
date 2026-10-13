//! Purpose: the ascii renderer's per-node/per-edge resolution and
//! column-geometry mapping — anonymous-node detection, display labels,
//! box widths, border/shape resolution, and the char-grid column widths
//! (ge.ascii_render.c1/c5/c7/c8).
//! Responsibilities: resolve label/style/shape per node, style per edge,
//! compute the column widths (node boxes' width, gap floors, labelled
//! straight-edge and group-title widening) and their prefix offsets.
//! Rationale: split from canvas.rs past the tidy file limit (gently-b0z).

use super::canvas::GAP_WIDTH;
use super::styles::{display_width, edge_gap_width, shape, Shape};
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::{Cell, Layout};

/// Minimum box width for a selfloop node: the loop elbow spans the box,
/// so the cell widens to fit both corners (oracle: `+------+` over `a`).
const SELFLOOP_MIN_WIDTH: usize = 8;
/// Anonymous-node predicate (ge.ascii_render.c7): the empty name (the
/// graph API's `add_anonymous_node`) or a `#<digits>` name — the text
/// parser names its anonymous nodes `#N` with the odd counter
/// (gently-6j0), and upstream's parser rejects a literal `#` in node
/// names, so a `#\d+` name can only be a parser-generated anon node.
fn is_anon(name: &str) -> bool {
    name.is_empty()
        || (name.len() > 1
            && name.starts_with('#')
            && name[1..].bytes().all(|b| b.is_ascii_digit()))
}

/// The node's display label: its `label` attribute (per-object, falling
/// back to the node class) when set, else its name (upstream Node::label)
/// — an anonymous node's stored label is `' '` (upstream Anon::_init).
pub(super) fn node_label(graph: &Graph, idx: usize) -> &str {
    if let Some(label) = object_attr(graph, Scope::Node(idx), ObjectKind::Node, "label") {
        return label;
    }
    let name = &graph.nodes[idx].name;
    if is_anon(name) {
        " "
    } else {
        name
    }
}

/// Box width of a node: display width of the label (double-width glyphs
/// count two columns, ge.ascii_render.c5) plus two border chars per side
/// — or one per side for a `none` border — widened for selfloop nodes so
/// the loop elbow fits above the box.
fn box_width(name: &str, style: &str, selfloop: bool) -> usize {
    let pad = if style == "none" { 1 } else { 2 };
    let w = display_width(name) + 2 * pad;
    if selfloop {
        w.max(SELFLOOP_MIN_WIDTH)
    } else {
        w
    }
}

/// The value of `key` on the object at `scope`, falling back to the base
/// class of `kind` (per-object attributes win over class attributes).
pub(super) fn object_attr<'a>(graph: &'a Graph, scope: Scope, kind: ObjectKind, key: &str) -> Option<&'a str> {
    graph
        .get_attr(scope, key)
        .or_else(|| graph.get_attr(Scope::Class(kind, String::new()), key))
}

/// Which nodes carry a selfloop edge (their boxes widen for the elbow).
pub(super) fn selfloops(graph: &Graph) -> Vec<bool> {
    (0..graph.nodes.len())
        .map(|i| graph.edges.iter().any(|e| e.from == i && e.to == i))
        .collect()
}

/// Per-node (border style, shape), per-object attributes over class ones.
pub(super) fn node_styles(graph: &Graph) -> Vec<(String, Shape)> {
    (0..graph.nodes.len())
        .map(|i| {
            let scope = Scope::Node(i);
            let style = match object_attr(graph, scope.clone(), ObjectKind::Node, "border_style") {
                Some(s) => s.to_string(),
                // upstream's node.anon class carries borderstyle none —
                // the anon node's own cell renders blank (c7).
                None if is_anon(&graph.nodes[i].name) => "none".to_string(),
                None => "solid".to_string(),
            };
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

/// Column widths per the char-grid mapping and their prefix offsets:
/// node columns take exactly their boxes' width (a `none`-border box is
/// narrower than the gap floor — oracle: ` x`), gap columns keep
/// GAP_WIDTH, and labelled straight edges widen their gap column (the
/// label hosts on the box-top row — oracle: `+---+  go   +---+` over
/// ` ----> `).
pub(super) fn column_geometry(
    graph: &Graph,
    layout: &Layout,
    node_style: &[(String, Shape)],
    selfloop_node: &[bool],
    edge_style: &[String],
) -> (Vec<usize>, Vec<(usize, usize)>) {
    let mut node_width = vec![0usize; layout.width];
    let mut node_col = vec![false; layout.width];
    for (idx, &(cx, _)) in layout.node_cells.iter().enumerate() {
        let (style, _) = &node_style[idx];
        let w = box_width(node_label(graph, idx), style, selfloop_node[idx]);
        node_width[cx] = node_width[cx].max(w);
        node_col[cx] = true;
    }
    let mut col_width = vec![GAP_WIDTH; layout.width];
    for c in 0..layout.width {
        if node_col[c] {
            col_width[c] = node_width[c];
        }
    }
    for (ei, path) in layout.edge_paths.iter().enumerate() {
        if path.len() == 1 {
            widen_edge_column(graph, layout, edge_style, &mut col_width, ei, path);
        }
    }
    (col_width, super::groups::title_widths(graph, layout))
}

/// Prefix offsets of the column widths.
pub(super) fn column_offsets(col_width: &[usize]) -> Vec<usize> {
    let mut offsets = vec![0usize; col_width.len() + 1];
    for (c, w) in col_width.iter().enumerate() {
        offsets[c + 1] = offsets[c] + w;
    }
    offsets
}

/// Widen the gap column of one straight single-gap edge (gently-7qz): the
/// label's width plus the 5-char label padding, the style's minimum gap
/// width (gently-oy2: upstream `_correct_size` widens the cell for
/// dot-dot-dash — the ` ..-> ` gap; the other styles keep the 5-column
/// floor), and the BD short-cell's +1 (upstream `_correct_size` w++ for a
/// bidirectional EDGE_HOR cell with arrows — the ` <--> ` gap), the last
/// only for the horizontal straight edge through the gap column.
fn widen_edge_column(
    graph: &Graph,
    layout: &Layout,
    edge_style: &[String],
    col_width: &mut [usize],
    ei: usize,
    path: &[Cell],
) {
    let c = path[0].0;
    let e = &graph.edges[ei];
    let bd_hor = e.arrows.start
        && e.arrows.end
        && layout.node_cells[e.from].1 == layout.node_cells[e.to].1;
    let extra = if bd_hor { 1 } else { 0 };
    if let Some(label) = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "label") {
        col_width[c] = col_width[c].max(display_width(label) + 5 + extra);
    }
    col_width[c] = col_width[c].max(edge_gap_width(&edge_style[ei]) + extra);
}

