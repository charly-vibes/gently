//! Purpose: the ascii renderer's per-node/per-edge resolution and
//! column-geometry mapping — anonymous-node detection, display labels,
//! box widths, border/shape resolution, and the char-grid column widths
//! (ge.ascii_render.c1/c5/c7/c8).
//! Responsibilities: resolve label/style/shape per node, style per edge,
//! compute the column widths (node boxes' width, gap floors, labelled
//! straight-edge and group-title widening) and their prefix offsets.
//! Rationale: split from canvas.rs past the tidy file limit (gently-b0z).

use super::canvas::GAP_WIDTH;
use super::labels::{
    default_align, max_line_width, object_label_lines, Align, LabelLine,
};
use super::styles::{edge_gap_width, shape, Shape};
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::{Cell, Layout};

/// The minimum row-band height: a standard band is border, label, border.
pub(super) use super::canvas::BAND_HEIGHT;

/// Aligned label lines per node and per straight horizontal labelled
/// edge, in model order (gently-527/gently-y7i: upstream
/// `_aligned_label` splits at `\n`/`\r`/`\l`/`\c`). Only a straight
/// single-gap horizontal edge renders a label CELL (upstream's full-gap
/// `EDGE_HOR` label cell); bent and vertical edges keep their
/// single-line surface.
pub(super) fn label_lines(
    graph: &Graph,
    layout: &Layout,
) -> (Vec<Vec<LabelLine>>, Vec<Option<Vec<LabelLine>>>) {
    let node_lines: Vec<Vec<LabelLine>> = (0..graph.nodes.len())
        .map(|i| {
            let label = node_label(graph, i);
            object_label_lines(
                graph,
                Scope::Node(i),
                ObjectKind::Node,
                label,
                default_align(graph, Scope::Node(i), ObjectKind::Node),
            )
        })
        .collect();
    let edge_lines: Vec<Option<Vec<LabelLine>>> = (0..graph.edges.len())
        .map(|ei| straight_label_lines(graph, layout, ei))
        .collect();
    (node_lines, edge_lines)
}

/// Row band heights and their unshifted prefix start lines: every cell
/// is grown to the tallest cell of its row (upstream
/// `_prepare_layout` rewrites each cell's h to the row height) — node
/// boxes carry lines+2, a straight horizontal edge's label cell carries
/// lines+2 (gently-527).
pub(super) fn band_layout(
    graph: &Graph,
    layout: &Layout,
    node_lines: &[Vec<LabelLine>],
    edge_lines: &[Option<Vec<LabelLine>>],
) -> (Vec<usize>, Vec<usize>) {
    let mut row_height = vec![BAND_HEIGHT; layout.height];
    for (idx, &(_, r)) in layout.node_cells.iter().enumerate() {
        let h = node_lines[idx].len() + 2;
        row_height[r] = row_height[r].max(h);
    }
    for (ei, lines) in edge_lines.iter().enumerate() {
        if let Some(lines) = lines {
            let edge = &graph.edges[ei];
            // Only a horizontal label cell sizes its band (its label rows
            // ride the band's top rows); a vertical label cell rides the
            // standard 3-line gap band between the endpoint bands.
            let horizontal = layout.node_cells[edge.from].1 == layout.node_cells[edge.to].1;
            if !horizontal {
                continue;
            }
            let r = layout.node_cells[edge.from].1;
            row_height[r] = row_height[r].max(lines.len() + 2);
        }
    }
    let mut band_tops = Vec::with_capacity(layout.height + 1);
    let mut acc = 0usize;
    for &h in &row_height {
        band_tops.push(acc);
        acc += h;
    }
    band_tops.push(acc);
    (row_height, band_tops)
}

/// Whether edge `ei` is a straight single-gap edge — horizontal (the
/// full-gap `EDGE_HOR` label cell) or vertical (the `EDGE_VER` label
/// cell, gently-cbk: the buy edge's `| buy` rides the gap cell's middle
/// line) — and carries a label. A horizontal label cell sizes its row
/// band (gently-527); a vertical one rides the standard 3-line gap band.
fn straight_label_lines(graph: &Graph, layout: &Layout, ei: usize) -> Option<Vec<LabelLine>> {
    let edge = &graph.edges[ei];
    let (sc, sr) = layout.node_cells[edge.from];
    let (tc, tr) = layout.node_cells[edge.to];
    let horizontal = sr == tr && sc.abs_diff(tc) == 2;
    let vertical = sc == tc && sr.abs_diff(tr) == 2;
    if (!horizontal && !vertical) || layout.edge_paths[ei].len() != 1 {
        return None;
    }
    let label = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "label")?;
    Some(object_label_lines(
        graph,
        Scope::Edge(ei),
        ObjectKind::Edge,
        label,
        default_align(graph, Scope::Edge(ei), ObjectKind::Edge),
    ))
}

/// Minimum box width for a selfloop node: the loop elbow spans the box,
/// so the cell widens to fit both corners (oracle: `+------+` over `a`).
const SELFLOOP_MIN_WIDTH: usize = 8;
/// Anonymous-node predicate (ge.ascii_render.c7): the empty name (the
/// graph API's `add_anonymous_node`) or a `#<digits>` name — the text
/// parser names its anonymous nodes `#N` with the odd counter
/// (gently-6j0), and upstream's parser rejects a literal `#` in node
/// names, so a `#\d+` name can only be a parser-generated anon node.
pub(super) fn is_anon(name: &str) -> bool {
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

/// Box width of a node: the widest resolved label line's UTF-8 byte
/// width (ge.ascii_render.c5 re-derived, gently-bvb: upstream
/// `dimensions()` byte-lengths the label — `[ Köln ]` is a 7-dash box;
/// gently-emd: the lines are the WRAPPED ones when textwrap is set)
/// plus two border chars per side — or one per side for a `none`
/// border — widened for selfloop nodes so the loop elbow fits above
/// the box.
fn box_width(lines: &[LabelLine], style: &str, selfloop: bool) -> usize {
    let pad = if style == "none" { 1 } else { 2 };
    let w = max_line_width(lines) + 2 * pad;
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
/// GAP_WIDTH, labelled straight edges widen their gap column (the label
/// hosts on the box-top row — oracle: `+---+  go   +---+` over
/// ` ----> `), and group titles widen their leftmost member's column
/// (c8).
pub(super) fn column_geometry(
    graph: &Graph,
    layout: &Layout,
    node_style: &[(String, Shape)],
    selfloop_node: &[bool],
    edge_style: &[String],
    node_lines: &[Vec<LabelLine>],
) -> Vec<usize> {
    let mut node_width = vec![0usize; layout.width];
    let mut node_col = vec![false; layout.width];
    for (idx, &(cx, _)) in layout.node_cells.iter().enumerate() {
        let (style, _) = &node_style[idx];
        let mut w = box_width(&node_lines[idx], style, selfloop_node[idx]);
        // an anonymous node's cell is upstream-FIXED at 3×3
        // (`Node::Anon::_correct_size`), independent of its blank label
        if is_anon(&graph.nodes[idx].name) {
            w = w.max(3);
        }
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
    for (c, w) in super::groups::title_widths(graph, layout) {
        if c < col_width.len() {
            col_width[c] = col_width[c].max(w);
        }
    }
    col_width
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
    // A `minlen >= 2` straight edge stretches one extra empty column
    // wide (gently-yhg, probed oracle: minlen 2..5 all widen the straight
    // gap by exactly one GAP_WIDTH, minlen 0/1 stay adjacent) — the
    // stretch is a gap-column width, not a placement change.
    let (fu, fv) = (layout.node_cells[e.from], layout.node_cells[e.to]);
    if fu.1 == fv.1 && fu.0.abs_diff(fv.0) == 2 {
        if let Some(v) = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "minlen") {
            if v.parse::<u32>().is_ok_and(|m| m >= 2) {
                col_width[c] += GAP_WIDTH;
            }
        }
    }
    if let Some(label) = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "label") {
        col_width[c] = col_width[c].max(label_cell_width(graph, ei, label) + extra);
    }
    col_width[c] = col_width[c].max(edge_gap_width(&edge_style[ei]) + extra);
}

/// The width a straight edge's label demands of its gap column: the
/// widest resolved LINE (gently-527: upstream `dimensions()` measures
/// the aligned label's lines, gently-emd: the WRAPPED ones when
/// textwrap is set) plus the 5-char label padding.
fn label_cell_width(graph: &Graph, ei: usize, label: &str) -> usize {
    let lines = object_label_lines(
        graph,
        Scope::Edge(ei),
        ObjectKind::Edge,
        label,
        Align::Left,
    );
    max_line_width(&lines) + 5
}

