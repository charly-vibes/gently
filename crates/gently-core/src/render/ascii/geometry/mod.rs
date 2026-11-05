//! Purpose: the ascii renderer's per-node/per-edge resolution —
//! anonymous-node detection, display labels, box widths, and
//! border/shape resolution (ge.ascii_render.c1/c5/c7/c8).
//! Responsibilities: resolve label/style/shape per node, style per edge,
//! and the shared width/balance primitives; column geometry lives in
//! `columns`.
//! Rationale: split from canvas.rs past the tidy file limit (gently-b0z);
//! the column-geometry passes were decomposed from a monolithic
//! `column_geometry` into one function per upstream pass.

pub(super) mod columns;

pub(super) use columns::{column_geometry, column_offsets};

use super::labels::{default_align, max_line_width, object_label_lines, LabelLine};
use super::styles::{shape, Shape};
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::Layout;

use super::canvas::BAND_HEIGHT;

/// Aligned label lines per node and per straight horizontal labelled
/// edge, in model order (gently-527/gently-y7i: upstream
/// `_aligned_label` splits at `\n`/`\r`/`\l`/`\c`). Only a straight
/// single-gap horizontal edge renders a label CELL (upstream's full-gap
/// `EDGE_HOR` label cell); bent and vertical edges keep their
/// single-line surface.
pub(super) fn label_lines(
    graph: &Graph,
    layout: &Layout,
    spans: &[(usize, usize)],
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
        .map(|ei| straight_label_lines(graph, layout, spans, ei))
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
    spans: &[(usize, usize)],
    edge_rows: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    let row_height = row_heights(graph, layout, node_lines, edge_lines, spans, edge_rows);
    let band_tops = prefix_sums(&row_height);
    (row_height, band_tops)
}

/// Row band heights: pass 1 (gently-awd, upstream `_prepare_layout`)
/// grows each row to its SINGLE-celled content — node boxes (lines + 2)
/// and edge cells (the 3-line gap band a vertical run rides, or a
/// labelled straight edge's taller cell). Rows a multi-celled node spans
/// beyond its band are continuation rows: empty ones collapse to height 0.
/// Pass 2 balances a multi-celled node's spanned rows smallest-first until
/// their sum reaches its own box height (upstream `_balance_sizes`).
fn row_heights(
    graph: &Graph,
    layout: &Layout,
    node_lines: &[Vec<LabelLine>],
    edge_lines: &[Option<Vec<LabelLine>>],
    spans: &[(usize, usize)],
    edge_rows: &[usize],
) -> Vec<usize> {
    let mut row_height = vec![0usize; layout.height];
    for (idx, &(_, r)) in layout.node_cells.iter().enumerate() {
        if spans[idx].0 + spans[idx].1 > 2 {
            continue;
        }
        let h = node_lines[idx].len() + 2;
        row_height[r] = row_height[r].max(h);
    }
    for &r in edge_rows {
        if r < row_height.len() {
            row_height[r] = row_height[r].max(BAND_HEIGHT);
        }
    }
    grow_label_bands(graph, layout, edge_lines, &mut row_height);
    balance_row_spans(layout, node_lines, spans, &mut row_height);
    row_height
}

/// A labelled straight horizontal edge's label rides ITS OWN grid row
/// (gently-awd): a stacked channel's single path cell sits on a
/// continuation row of the boxes' row span, south of the band row. Only a
/// horizontal label cell sizes its band (its label rows ride the band's
/// top rows); a vertical label cell rides the standard 3-line gap band
/// between the endpoint bands (gently-527).
fn grow_label_bands(
    graph: &Graph,
    layout: &Layout,
    edge_lines: &[Option<Vec<LabelLine>>],
    row_height: &mut [usize],
) {
    for (ei, lines) in edge_lines.iter().enumerate() {
        if let Some(lines) = lines {
            let edge = &graph.edges[ei];
            let horizontal = layout.node_cells[edge.from].1 == layout.node_cells[edge.to].1;
            if !horizontal {
                continue;
            }
            let r = layout.edge_paths[ei][0].1;
            row_height[r] = row_height[r].max(lines.len() + 2);
        }
    }
}

/// Pass 2 (gently-awd): a multi-celled node's box spans its whole row
/// span, so the spanned rows balance smallest-first until their sum
/// reaches the node's own box height (upstream `_balance_sizes`).
fn balance_row_spans(
    layout: &Layout,
    node_lines: &[Vec<LabelLine>],
    spans: &[(usize, usize)],
    row_height: &mut [usize],
) {
    for (idx, &(_, r)) in layout.node_cells.iter().enumerate() {
        let (span_c, cy) = spans[idx];
        if span_c + cy <= 2 {
            continue;
        }
        let need = node_lines[idx].len() + 2;
        let balanced: Vec<usize> = (r..r + cy)
            .map(|rr| row_height.get(rr).copied().unwrap_or(0))
            .collect();
        let balanced = balance_to(balanced, need);
        for (k, &h) in balanced.iter().enumerate() {
            if r + k < row_height.len() {
                row_height[r + k] = h;
            }
        }
    }
}

/// Unshifted prefix start lines: each entry is the sum of the preceding
/// sizes, with one extra terminal entry.
pub(super) fn prefix_sums(sizes: &[usize]) -> Vec<usize> {
    let mut band_tops = Vec::with_capacity(sizes.len() + 1);
    let mut acc = 0usize;
    for &h in sizes {
        band_tops.push(acc);
        acc += h;
    }
    band_tops.push(acc);
    band_tops
}

/// Grow the sizes smallest-first until their sum reaches `need`
/// (gently-awd, upstream `Grid::_balance_sizes`): zero sizes are
/// skipped by the smallest-search (they only ever grow when EVERY
/// size is zero, where the first one grows), the first smallest wins
/// ties, and a single-element list is simply raised to `need`.
/// [`balance_sizes`] on an owned vector, returning the balanced sizes.
pub(super) fn balance_to(mut sizes: Vec<usize>, need: usize) -> Vec<usize> {
    balance_sizes(&mut sizes, need);
    sizes
}

pub(super) fn balance_sizes(sizes: &mut [usize], need: usize) {
    if need == 0 {
        return;
    }
    if sizes.len() == 1 {
        sizes[0] = sizes[0].max(need);
        return;
    }
    loop {
        let sum: usize = sizes.iter().sum();
        if sum >= need {
            return;
        }
        let (mut sm_i, mut sm) = (0, usize::MAX);
        for (i, &s) in sizes.iter().enumerate() {
            if s == 0 {
                continue;
            }
            if s < sm {
                sm = s;
                sm_i = i;
            }
        }
        sizes[sm_i] += 1;
    }
}

/// Whether edge `ei` is a straight single-gap edge — horizontal (the
/// full-gap `EDGE_HOR` label cell) or vertical (the `EDGE_VER` label
/// cell, gently-cbk: the buy edge's `| buy` rides the gap cell's middle
/// line) — and carries a label. A horizontal label cell sizes its row
/// band (gently-527); a vertical one rides the standard 3-line gap band.
fn straight_label_lines(
    graph: &Graph,
    layout: &Layout,
    spans: &[(usize, usize)],
    ei: usize,
) -> Option<Vec<LabelLine>> {
    let edge = &graph.edges[ei];
    let (sc, sr) = layout.node_cells[edge.from];
    let (tc, tr) = layout.node_cells[edge.to];
    let horizontal =
        sr == tr && (tc == sc + spans[edge.from].0 + 1 || sc == tc + spans[edge.to].0 + 1);
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
            (
                style,
                shape(object_attr(graph, scope, ObjectKind::Node, "shape")),
            )
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
