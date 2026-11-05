//! Purpose: the blank-grid build pass — column geometry resolution,
//! band heights, group-frame margins, and the `Canvas::new` assembly.
//! Responsibilities: resolve spans, label lines, column widths, band
//! layout, frame margins, and allocate the blank byte grid.
//! Rationale: split from canvas.rs past the tidy file limit (gently-b0z).

use super::super::geometry::{
    band_layout, column_geometry, column_offsets, edge_styles, is_anon, label_lines, node_styles,
    selfloops,
};
use super::super::groups::{self, Frame};
use super::super::labels::LabelLine;
use super::super::styles::Shape;
use super::Canvas;
use crate::graph::Graph;
use crate::layout::Layout;

/// The layout's own spans (gently-byq): the grown row spans of
/// indexed-port nodes live on the Layout — re-deriving them from the
/// attrs would shrink the boxes back.
fn resolved_spans(graph: &Graph, layout: &Layout) -> Vec<(usize, usize)> {
    if layout.spans.len() == graph.nodes.len() {
        layout.spans.clone()
    } else {
        super::node_spans(graph)
    }
}

/// The column/label resolution half of the build: spans, wrapped label
/// lines (gently-emd: resolved FIRST — the wrapped lines drive the box
/// widths), the column geometry consuming them, and the byte offsets.
struct Columns {
    node_lines: Vec<Vec<LabelLine>>,
    edge_lines: Vec<Option<Vec<LabelLine>>>,
    col_width: Vec<usize>,
    offsets: Vec<usize>,
    spans: Vec<(usize, usize)>,
    node_style: Vec<(String, Shape)>,
    edge_style: Vec<String>,
}

/// The band-height half: every cell is grown to the tallest cell of its
/// row (upstream `_prepare_layout` rewrites each cell's h to the row
/// height).
struct Bands {
    row_height: Vec<usize>,
    band_tops: Vec<usize>,
}

/// The frame half: the blank grid, the margin-shifted offsets, the group
/// frames, the margins, and the last drawable line.
struct Framed {
    grid: Vec<Vec<u8>>,
    offsets: Vec<usize>,
    frames: Vec<Frame>,
    ml: usize,
    mt: usize,
    floor: usize,
}

/// Resolve the column/label half: spans, label lines, column geometry,
/// offsets, node styles.
fn resolve_columns(graph: &Graph, layout: &Layout) -> Columns {
    let selfloop_node = selfloops(graph);
    let node_style = node_styles(graph);
    let spans = resolved_spans(graph, layout);
    let (node_lines, edge_lines) = label_lines(graph, layout, &spans);
    let edge_style = edge_styles(graph);
    let col_width = column_geometry(
        graph,
        layout,
        &node_style,
        &selfloop_node,
        &edge_style,
        &node_lines,
        &spans,
    );
    let offsets = column_offsets(&col_width);
    Columns {
        node_lines,
        edge_lines,
        col_width,
        offsets,
        spans,
        node_style,
        edge_style,
    }
}

/// Resolve the band half from the columns: the rows every edge path
/// touches drive the growth (gently-527).
fn band_rows(graph: &Graph, layout: &Layout, columns: &Columns) -> Bands {
    let edge_rows: Vec<usize> = layout
        .edge_paths
        .iter()
        .flat_map(|p| p.iter().map(|&(_, r)| r))
        .collect();
    let (row_height, band_tops) = band_layout(
        graph,
        layout,
        &columns.node_lines,
        &columns.edge_lines,
        &columns.spans,
        &edge_rows,
    );
    Bands {
        row_height,
        band_tops,
    }
}

/// Resolve the frame half from both: margins (c8), the blank grid, and
/// the shifted offsets.
fn frame_half(graph: &Graph, layout: &Layout, columns: &Columns, bands: &Bands) -> Framed {
    let (frames, (ml, mt, mr, mb)) = groups::frame_margins(
        graph,
        layout,
        &columns.col_width,
        &columns.offsets,
        &bands.band_tops,
        &bands.row_height,
    );
    let total_lines = bands.band_tops[layout.height];
    let (grid, offsets) = shifted_grid(
        columns.offsets.clone(),
        layout.width,
        ml,
        mr,
        mt,
        mb,
        total_lines,
    );
    let floor = last_floor(graph, layout, columns, bands, grid.len(), mt);
    Framed {
        grid,
        offsets,
        frames,
        ml,
        mt,
        floor,
    }
}

/// The last drawable line: anonymous nodes draw only blanks (borderstyle
/// none over a blank label), so they never hold the floor open.
fn last_floor(
    graph: &Graph,
    layout: &Layout,
    columns: &Columns,
    bands: &Bands,
    grid_len: usize,
    mt: usize,
) -> usize {
    let draws: Vec<bool> = (0..graph.nodes.len())
        .map(|i| !is_anon(&graph.nodes[i].name))
        .collect();
    last_band_floor(
        layout,
        &draws,
        &bands.row_height,
        &bands.band_tops,
        grid_len,
        mt,
        &columns.spans,
    )
}

/// Shift the column offsets by the left margin and allocate the blank
/// byte grid padded by the right/bottom margins.
fn shifted_grid(
    offsets: Vec<usize>,
    width: usize,
    ml: usize,
    mr: usize,
    mt: usize,
    mb: usize,
    total_lines: usize,
) -> (Vec<Vec<u8>>, Vec<usize>) {
    let offsets: Vec<usize> = offsets.into_iter().map(|o| o + ml).collect();
    let total_width = offsets[width] + mr;
    let grid = vec![vec![b' '; total_width]; total_lines + mt + mb];
    (grid, offsets)
}

/// The last band's box-bottom line: trailing blank lines beyond it never
/// render — the oracle's as_ascii ends at the last drawn line (the port-*
/// dip-below fixtures end at their gap-row run), while a trailing blank
/// WITHIN the last band still renders (the border:none node pins
/// `\n x\n\n`). An anonymous node's cell renders only blanks (borderstyle
/// none over a blank label), so it does not hold the floor open: the
/// oracle's output ends at the last drawn line even when a blank anon
/// band trails the art (gently-cbk, bg-edge-node.observed ends at the buy
/// edge's run).
fn last_band_floor(
    layout: &Layout,
    draws: &[bool],
    row_height: &[usize],
    band_tops: &[usize],
    grid_len: usize,
    mt: usize,
    spans: &[(usize, usize)],
) -> usize {
    layout
        .node_cells
        .iter()
        .zip(draws)
        .zip(spans)
        .filter(|&((_, &draws_something), _)| draws_something)
        .map(|((&(_, r), _), &(_, cy))| {
            let h: usize = (r..r + cy)
                .map(|rr| row_height.get(rr).copied().unwrap_or(0))
                .sum();
            band_tops[r] + h - 1 + mt
        })
        .max()
        .map(|line| line.min(grid_len.saturating_sub(1)))
        .unwrap_or(0)
}

/// Target-side arrowhead presence, source-side arrowhead presence, and
/// bidirectionality per edge, in model order.
fn edge_arrow_flags(graph: &Graph) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
    let end = graph.edges.iter().map(|e| e.arrows.end).collect();
    let start = graph.edges.iter().map(|e| e.arrows.start).collect();
    let bidir = graph
        .edges
        .iter()
        .map(|e| e.arrows.start && e.arrows.end)
        .collect();
    (end, start, bidir)
}

impl Canvas {
    /// Build the blank grid: column widths per the byte-grid mapping
    /// (widened for labelled straight edges, whose gap hosts the label
    /// rows — oracle: `+---+  put msg    +---+` over ` ----------> `),
    /// the row-band heights from the label lines, the group-frame
    /// margins, then one blank byte row per canvas line. Label lines
    /// resolve FIRST (gently-emd); row band heights grow to the tallest
    /// cell of their row (upstream `_prepare_layout`).
    pub(in crate::render::ascii) fn new(graph: &Graph, layout: &Layout) -> Canvas {
        let columns = resolve_columns(graph, layout);
        let bands = band_rows(graph, layout, &columns);
        let frame = frame_half(graph, layout, &columns, &bands);
        assemble(parts(graph, columns, bands, frame))
    }
}

/// Bundle every resolved canvas field for [`assemble`].
#[allow(clippy::type_complexity)]
fn parts(graph: &Graph, columns: Columns, bands: Bands, frame: Framed) -> Parts {
    let (edge_arrow_end, edge_start_arrow, edge_bidir) = edge_arrow_flags(graph);
    Parts {
        grid: frame.grid,
        row_height: bands.row_height,
        band_tops: bands.band_tops,
        node_lines: columns.node_lines,
        edge_lines: columns.edge_lines,
        offsets: frame.offsets,
        col_width: columns.col_width,
        node_style: columns.node_style,
        edge_style: columns.edge_style,
        spans: columns.spans,
        arrows: (edge_arrow_end, edge_start_arrow, edge_bidir),
        frames: frame.frames,
        ml: frame.ml,
        mt: frame.mt,
        floor: frame.floor,
    }
}

/// Every canvas field, resolved, ready for [`assemble`].
#[allow(clippy::type_complexity)]
struct Parts {
    grid: Vec<Vec<u8>>,
    row_height: Vec<usize>,
    band_tops: Vec<usize>,
    node_lines: Vec<Vec<LabelLine>>,
    edge_lines: Vec<Option<Vec<LabelLine>>>,
    offsets: Vec<usize>,
    col_width: Vec<usize>,
    node_style: Vec<(String, Shape)>,
    edge_style: Vec<String>,
    spans: Vec<(usize, usize)>,
    arrows: (Vec<bool>, Vec<bool>, Vec<bool>),
    frames: Vec<Frame>,
    ml: usize,
    mt: usize,
    floor: usize,
}

/// Materialize the canvas from its resolved parts.
fn assemble(p: Parts) -> Canvas {
    let (edge_arrow_end, edge_start_arrow, edge_bidir) = p.arrows;
    Canvas {
        grid: p.grid,
        row_height: p.row_height,
        band_tops: p.band_tops,
        node_lines: p.node_lines,
        edge_lines: p.edge_lines,
        offsets: p.offsets,
        col_width: p.col_width,
        node_style: p.node_style,
        edge_style: p.edge_style,
        spans: p.spans,
        edge_arrow_end,
        edge_start_arrow,
        edge_bidir,
        frames: p.frames,
        margin_left: p.ml,
        margin_top: p.mt,
        last_band_floor: p.floor,
    }
}
