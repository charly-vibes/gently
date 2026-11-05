//! Purpose: the ascii renderer's char-grid column widths
//! (ge.ascii_render.c5/c8).
//! Responsibilities: compute each column's width from its anchoring node
//! box, edge-cell floors, labelled-edge and group-title widening, and
//! multi-celled span balancing; expose the prefix offsets.
//! Rationale: one function per upstream `_prepare_layout` pass, each
//! growing a shared `col_width` vector — the passes compose exactly as
//! upstream's did, without a monolithic `column_geometry`.

use super::super::canvas::GAP_WIDTH;
use super::super::groups;
use super::super::labels::{max_line_width, object_label_lines, Align, LabelLine};
use super::super::styles::{edge_gap_width, Shape};
use super::{box_width, is_anon, object_attr};
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::{Cell, Layout};

/// Column widths per the char-grid mapping: node columns take exactly
/// their boxes' width (a `none`-border box is narrower than the gap
/// floor — oracle: ` x`), gap columns keep GAP_WIDTH, labelled straight
/// edges widen their gap column (the label hosts on the box-top row —
/// oracle: `+---+  go   +---+` over ` ----> `), and group titles widen
/// their leftmost member's column (c8).
pub(crate) fn column_geometry(
    graph: &Graph,
    layout: &Layout,
    node_style: &[(String, Shape)],
    selfloop_node: &[bool],
    edge_style: &[String],
    node_lines: &[Vec<LabelLine>],
    spans: &[(usize, usize)],
) -> Vec<usize> {
    let mut col_width = vec![0usize; layout.width];
    let node_col = node_column_widths(
        graph,
        layout,
        node_style,
        selfloop_node,
        node_lines,
        spans,
        &mut col_width,
    );
    edge_column_floors(layout, &mut col_width);
    straight_single_edge_columns(graph, layout, edge_style, spans, &mut col_width);
    slot_label_widths(graph, layout, &mut col_width);
    structural_gap_floors(layout, spans, &node_col, &mut col_width);
    balance_spans(
        layout,
        node_style,
        selfloop_node,
        node_lines,
        spans,
        &mut col_width,
    );
    absorb_title_widths(graph, layout, &mut col_width);
    col_width
}

/// Group-title columns widen to their title width (title never wraps).
fn absorb_title_widths(graph: &Graph, layout: &Layout, col_width: &mut [usize]) {
    for (c, w) in groups::title_widths(graph, layout) {
        if c < col_width.len() {
            col_width[c] = col_width[c].max(w);
        }
    }
}

/// Prefix offsets of the column widths.
pub(crate) fn column_offsets(col_width: &[usize]) -> Vec<usize> {
    super::prefix_sums(col_width)
}

/// Straight single-gap edges widen their gap column for the label and
/// the style's gap floor (see [`widen_edge_column`]).
fn straight_single_edge_columns(
    graph: &Graph,
    layout: &Layout,
    edge_style: &[String],
    spans: &[(usize, usize)],
    col_width: &mut [usize],
) {
    for (ei, path) in layout.edge_paths.iter().enumerate() {
        if path.len() == 1 {
            widen_edge_column(graph, layout, edge_style, col_width, ei, path, spans);
        }
    }
}

/// Pass 1 (gently-awd, upstream `_prepare_layout`): only SINGLE-celled
/// nodes record their column's minimum width; every column starts at
/// width 0 — a gap column's width comes from its edge cells (see
/// [`edge_column_floors`]), and a multi-celled node's spanned columns
/// stay 0 until its own balance pass grows them. Returns which columns
/// anchor a single-celled node.
fn node_column_widths(
    graph: &Graph,
    layout: &Layout,
    node_style: &[(String, Shape)],
    selfloop_node: &[bool],
    node_lines: &[Vec<LabelLine>],
    spans: &[(usize, usize)],
    col_width: &mut [usize],
) -> Vec<bool> {
    let node_width =
        anchored_node_widths(graph, layout, node_style, selfloop_node, node_lines, spans);
    let node_col = single_cell_columns(layout, spans);
    for c in 0..layout.width {
        if node_col[c] {
            col_width[c] = node_width[c];
        }
    }
    node_col
}

/// Each single-celled node's box width, maximized into its column
/// (anonymous nodes' cells are upstream-FIXED at 3×3 —
/// `Node::Anon::_correct_size` — independent of their blank label).
fn anchored_node_widths(
    graph: &Graph,
    layout: &Layout,
    node_style: &[(String, Shape)],
    selfloop_node: &[bool],
    node_lines: &[Vec<LabelLine>],
    spans: &[(usize, usize)],
) -> Vec<usize> {
    layout.node_cells.iter().enumerate().fold(
        vec![0usize; layout.width],
        |mut widths, (idx, &(cx, _))| {
            if spans[idx].0 + spans[idx].1 > 2 {
                return widths;
            }
            let (style, _) = &node_style[idx];
            let w = box_width(&node_lines[idx], style, selfloop_node[idx]);
            // an anonymous node's cell is upstream-FIXED at 3×3
            // (`Node::Anon::_correct_size`), independent of its blank label
            let w = if is_anon(&graph.nodes[idx].name) {
                w.max(3)
            } else {
                w
            };
            widths[cx] = widths[cx].max(w);
            widths
        },
    )
}

/// Columns anchoring a single-celled node (span ≤ 2 cells total).
fn single_cell_columns(layout: &Layout, spans: &[(usize, usize)]) -> Vec<bool> {
    let mut flags = vec![false; layout.width];
    for (&(cx, _), &(sc, sr)) in layout.node_cells.iter().zip(spans) {
        if sc + sr <= 2 {
            flags[cx] = true;
        }
    }
    flags
}

/// Edge cells are single-celled and record their columns' widths
/// (upstream pass 1): a cell on a horizontal run raises its column to
/// the 5-char gap floor; a VERTICAL-run cell only needs the 3-char VER
/// floor (gently-byq: the dodge channel column renders 3 wide —
/// `53..55`, pipe at offset 2 — where the oracle's EDGE_VER cells sit).
/// Orientation per cell: vertical when a path neighbor shares its column.
fn edge_column_floors(layout: &Layout, col_width: &mut [usize]) {
    for path in &layout.edge_paths {
        for (i, &(c, _)) in path.iter().enumerate() {
            if c < col_width.len() {
                col_width[c] = col_width[c].max(cell_floor(path, i, c));
            }
        }
    }
}

/// A path cell's width floor: 3 for a VERTICAL-run cell (gently-byq:
/// the dodge channel column renders 3 wide — `53..55`, pipe at offset 2
/// — where the oracle's EDGE_VER cells sit), the 5-char gap floor
/// otherwise. Orientation per cell: vertical when a path neighbor
/// shares its column.
fn cell_floor(path: &[Cell], i: usize, c: usize) -> usize {
    let vertical = path
        .get(i.wrapping_sub(1))
        .is_some_and(|&p| p.0 == c && i > 0)
        || path.get(i + 1).is_some_and(|&n| n.0 == c);
    if vertical {
        3
    } else {
        GAP_WIDTH
    }
}

/// A slot-routed edge's label rides the ARROW-END cell (gently-byq):
/// its column grows to the label demand — the label+5 cell width
/// (the SO fixture's Client column grows 10→13 for `3. X Msg`).
fn slot_label_widths(graph: &Graph, layout: &Layout, col_width: &mut [usize]) {
    for (ei, cell) in layout.slot_labels.iter().enumerate() {
        let Some(&(c, _)) = cell.as_ref() else {
            continue;
        };
        if let Some(label) = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "label") {
            if c < col_width.len() {
                col_width[c] = col_width[c].max(label_cell_width(graph, ei, label));
            }
        }
    }
}

/// A structural gap column — no node anchors there and no multi-celled
/// span covers it — keeps the 5-column gap floor even with no edge cell
/// on it (the edgeless row's between-box gap; upstream places edgeless
/// neighbours one gap column apart). Span-reserved columns stay 0 for
/// their node's balance pass to grow.
fn structural_gap_floors(
    layout: &Layout,
    spans: &[(usize, usize)],
    node_col: &[bool],
    col_width: &mut [usize],
) {
    let spanned: Vec<bool> = (0..layout.width)
        .map(|c| {
            layout
                .node_cells
                .iter()
                .zip(spans)
                .any(|(&(cx, _), &(span_c, _))| span_c > 1 && cx <= c && c < cx + span_c)
        })
        .collect();
    for c in 0..layout.width {
        if !node_col[c] && !spanned[c] {
            col_width[c] = col_width[c].max(GAP_WIDTH);
        }
    }
}

/// Pass 2 (gently-awd): a multi-celled node balances its spanned
/// columns smallest-first until their sum reaches its own box width
/// (upstream `_balance_sizes`) — the box renders over the whole span,
/// so its width is the span sum.
fn balance_spans(
    layout: &Layout,
    node_style: &[(String, Shape)],
    selfloop_node: &[bool],
    node_lines: &[Vec<LabelLine>],
    spans: &[(usize, usize)],
    col_width: &mut [usize],
) {
    for (idx, &(cx, _)) in layout.node_cells.iter().enumerate() {
        let (span_c, span_r) = spans[idx];
        if span_c + span_r <= 2 {
            continue;
        }
        let (style, _) = &node_style[idx];
        let own = box_width(&node_lines[idx], style, selfloop_node[idx]);
        let sizes: Vec<usize> = (cx..cx + span_c)
            .map(|c| col_width.get(c).copied().unwrap_or(0))
            .collect();
        for (k, &w) in super::balance_to(sizes, own).iter().enumerate() {
            if cx + k < col_width.len() {
                col_width[cx + k] = w;
            }
        }
    }
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
    spans: &[(usize, usize)],
) {
    let c = path[0].0;
    let e = &graph.edges[ei];
    let bd_hor =
        e.arrows.start && e.arrows.end && layout.node_cells[e.from].1 == layout.node_cells[e.to].1;
    let extra = if bd_hor { 1 } else { 0 };
    widen_for_minlen(graph, layout, spans, ei, c, col_width);
    if let Some(label) = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "label") {
        col_width[c] = col_width[c].max(label_cell_width(graph, ei, label) + extra);
    }
    col_width[c] = col_width[c].max(edge_gap_width(&edge_style[ei]) + extra);
}

/// A `minlen >= 2` straight edge stretches one extra empty column wide
/// (gently-yhg, probed oracle: minlen 2..5 all widen the straight gap by
/// exactly one GAP_WIDTH, minlen 0/1 stay adjacent) — the stretch is a
/// gap-column width, not a placement change.
fn widen_for_minlen(
    graph: &Graph,
    layout: &Layout,
    spans: &[(usize, usize)],
    ei: usize,
    c: usize,
    col_width: &mut [usize],
) {
    let e = &graph.edges[ei];
    let (fu, fv) = (layout.node_cells[e.from], layout.node_cells[e.to]);
    let straight = if fu.1 == fv.1 && fu.0 < fv.0 {
        fv.0 == fu.0 + spans[e.from].0 + 1
    } else if fu.1 == fv.1 {
        fu.0 == fv.0 + spans[e.to].0 + 1
    } else {
        false
    };
    if !straight {
        return;
    }
    if let Some(v) = object_attr(graph, Scope::Edge(ei), ObjectKind::Edge, "minlen") {
        if v.parse::<u32>().is_ok_and(|m| m >= 2) {
            col_width[c] += GAP_WIDTH;
        }
    }
}

/// The width a straight edge's label demands of its gap column: the
/// widest resolved LINE (gently-527: upstream `dimensions()` measures
/// the aligned label's lines, gently-emd: the WRAPPED ones when
/// textwrap is set) plus the 5-char label padding.
fn label_cell_width(graph: &Graph, ei: usize, label: &str) -> usize {
    let lines = object_label_lines(graph, Scope::Edge(ei), ObjectKind::Edge, label, Align::Left);
    max_line_width(&lines) + 5
}
