//! Purpose: straight-edge run drawing for the ascii renderer
//! (gently-7qz) — the glyph runs a single-gap straight edge stamps.
//! Responsibilities: render a straight edge's gap column — the BD run
//! (both arrowheads, upstream _draw_hor END_W/END_E) or the plain style
//! run (arrowed or not, gently-bpq: no head on an undirected labeled
//! run) — mirroring westward runs, and hosting labels on the box-top row
//! per the upstream _insert_label field rule.
//! Rationale: extracted from canvas.rs past the tidy file limit; the run
//! rules live in styles.rs, pinned by the recorded claims
//! (pod-bidir*.observed, pod-label-attr.observed).

use super::canvas::{centered, Canvas, NodeGeom};
use super::labels::{Align, LabelLine};
use super::polyline;
use super::runs::{draw_straight, draw_straight_ver};
use super::styles::Shape;
use super::styles::{self, byte_width, edge_fill, edge_run_at};
use super::RenderError;
use crate::graph::Graph;
use crate::layout::Layout;

/// Which endpoint of an edge is a `shape: edge` node (gently-cbk) — the
/// Scout.pm endpoint-flag stripping signal passed to the edge drawers.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EdgeNodeEnds {
    /// The source node renders as an edge cell.
    pub(super) src: bool,
    /// The target node renders as an edge cell.
    pub(super) dst: bool,
}

impl Canvas {
    /// Whether either endpoint of the edge is a `shape: edge` node — the
    /// Scout.pm endpoint-flag stripping signal (gently-cbk): a shape:edge
    /// source loses its start flags, a shape:edge target loses its end
    /// flags on non-short (multi-cell) paths and its `START_E/W` on short
    /// ones.
    pub(super) fn edge_node_ends(&self, from: usize, to: usize) -> EdgeNodeEnds {
        EdgeNodeEnds {
            src: self.node_style[from].1 == Shape::Edge,
            dst: self.node_style[to].1 == Shape::Edge,
        }
    }

    /// Stamp a `shape: edge` node as an edge cell (gently-cbk, upstream
    /// `Node::as_ascii` builds an `EDGE_HOR | EDGE_LABEL_CELL` cell): a
    /// solid dash line across the cell's full width on the band's middle
    /// row, the label lines riding above it — bottom-aligned in the
    /// cell's `h-2` field, left-aligned at byte offset 1 in a `w-2`
    /// field. The label rides the FRESH edge the cell is rendered
    /// through, whose `align` attribute defaults left — the node's own
    /// align (center) is not consulted (probed oracle: `car` and `train`
    /// sit at the cell's offset 1, upstream `_insert_label`'s `xs = 1`).
    /// The cell draws no borders.
    pub(super) fn draw_edge_cell(&mut self, geom: &NodeGeom, lines: &[LabelLine]) {
        let w = geom.w;
        self.stamp_at(geom.top + (geom.h - 1) / 2, geom.x0, &"-".repeat(w));
        let start = geom.top + geom.h.saturating_sub(2 + lines.len());
        for (i, line) in lines.iter().enumerate() {
            self.stamp_at(start + i, geom.x0 + 1, &line.text);
        }
    }

    /// Stamp the straight run of one edge (see `edges::draw_straight`).
    /// Draw one edge: selfloop elbow, straight gap arrow, or polyline.
    pub(super) fn draw_edge(
        &mut self,
        graph: &Graph,
        layout: &Layout,
        edge_index: usize,
    ) -> Result<(), RenderError> {
        let edge = &graph.edges[edge_index];
        let (sc, sr) = layout.node_cells[edge.from];
        let (tc, tr) = layout.node_cells[edge.to];
        let path = &layout.edge_paths[edge_index];
        if sc == tc && sr == tr {
            self.draw_selfloop((sc, sr));
            return Ok(());
        }
        if path.len() == 1 {
            let path = &layout.edge_paths[edge_index];
            let (from, to) = (edge.from, edge.to);
            if self.draw_single_cell(path, edge_index, from, to, (sc, sr), (tc, tr))? {
                return Ok(());
            }
        }
        self.draw_polyline(layout, edge_index, edge)
    }

    /// The multi-cell path form: the polyline router (gently-y7i).
    fn draw_polyline(
        &mut self,
        layout: &Layout,
        edge_index: usize,
        edge: &crate::graph::Edge,
    ) -> Result<(), RenderError> {
        let ends = self.edge_node_ends(edge.from, edge.to);
        polyline::draw(
            self,
            layout,
            edge_index,
            edge.from,
            edge.to,
            edge.arrows.start,
            ends,
        )
    }

    /// A one-cell path is a straight gap arrow: east/west in the gap
    /// column when the endpoints ride the same row, north/south in the
    /// gap row when they share a column two rows apart. Returns whether
    /// one of those forms matched (otherwise the polyline drawer runs).
    fn draw_single_cell(
        &mut self,
        path: &[(usize, usize)],
        edge_index: usize,
        from: usize,
        to: usize,
        (sc, sr): (usize, usize),
        (tc, tr): (usize, usize),
    ) -> Result<bool, RenderError> {
        let (span_f, span_t) = (self.spans[from], self.spans[to]);
        if sr == tr && (tc == sc + span_f.0 + 1 || sc == tc + span_t.0 + 1) {
            // The run rides the PATH's row (gently-awd): a stacked
            // parallel channel's single cell sits on a continuation
            // row of the boxes' row span, south of the band row.
            self.draw_gap_column(path[0].1, path[0].0, tc < sc, edge_index, from, to);
            return Ok(true);
        }
        if sc == tc && sr.abs_diff(tr) == 2 {
            self.draw_gap_row(path[0].1, path[0].0, tr < sr, edge_index);
            return Ok(true);
        }
        Ok(false)
    }

    /// Straight through the single gap column: centered run —
    /// arrow-less or style-headed (c2), widened to a pattern fill plus
    /// head when the edge carries a label (oracle bytes: ` ----> `
    /// under `  go  `); the label rides as its aligned lines
    /// (gently-527/y7i).
    fn draw_gap_column(
        &mut self,
        sr: usize,
        c: usize,
        westward: bool,
        edge_index: usize,
        from: usize,
        to: usize,
    ) {
        let lines = self.edge_lines[edge_index].clone().unwrap_or_default();
        let ends = self.edge_node_ends(from, to);
        draw_straight(self, sr, c, westward, edge_index, &lines, ends);
    }

    /// Straight through the single gap row (gently-cbk, upstream's
    /// EDGE_VER short cell): pipes at the fixed offset-2 column, the
    /// arrowhead at the target-adjacent end, the label left of the
    /// pipe on the band's middle line (`| buy`).
    fn draw_gap_row(&mut self, gap_row: usize, c: usize, upward: bool, edge_index: usize) {
        let lines = self.edge_lines[edge_index].clone().unwrap_or_default();
        draw_straight_ver(self, gap_row, c, upward, edge_index, &lines);
    }
}
