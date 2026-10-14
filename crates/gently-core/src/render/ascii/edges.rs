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
use super::styles::Shape;
use super::styles::{self, byte_width, edge_fill, edge_run_at};
use super::RenderError;
use crate::graph::Graph;
use crate::layout::Layout;

/// The run mirrored for a westward edge: reversed, head flipped.
fn mirror_run(run: &str) -> String {
    run.chars()
        .rev()
        .map(|c| if c == '>' { '<' } else { c })
        .collect()
}

/// Stamp the straight run of one edge: the style run centered in the gap
/// column, mirrored when the target lies west; a labelled run is the
/// style's fill pattern widened to the gap plus a `>` head (no head on an
/// undirected run — the oracle draws none, gently-bpq), with the label
/// centered on the box-top row (oracle bytes). A bidirectional edge
/// (gently-7qz) renders the BD run ` <fill> ` — both arrowheads, one
/// column wider — with the label in [2, w-3) of the widened cell
/// (upstream _insert_label's field).
///
/// Endpoint flag stripping for `shape: edge` nodes (gently-cbk, upstream
/// Scout.pm's short-cell rules): a shape:edge SOURCE loses every START
/// flag (the run hangs flush off its border — `--->` into Ulm), a
/// shape:edge TARGET loses the START flag on the edge's far side — for an
/// undirected run only `START_W` survives (the `--------` through
/// Bonn–train–Ulm); a DIRECTED run's arrowhead always survives (rent's
/// `>` rides into the car edge cell).
pub(super) fn draw_straight(
    canvas: &mut Canvas,
    sr: usize,
    c: usize,
    westward: bool,
    edge_index: usize,
    lines: &[LabelLine],
    ends: EdgeNodeEnds,
) {
    if (ends.src || ends.dst) && !canvas.edge_bidir[edge_index] {
        stamp_flagged(canvas, sr, c, westward, edge_index, lines, ends);
        return;
    }
    let style = canvas.edge_style[edge_index].clone();
    let width = canvas.col_width[c];
    let arrowed = canvas.edge_arrow_end[edge_index];
    let bidir = canvas.edge_bidir[edge_index];
    let run = if bidir {
        styles::bd_run_at(&style, width, canvas.offsets[c])
    } else if lines.is_empty() {
        edge_run_at(&style, arrowed, width, canvas.offsets[c])
    } else {
        labelled_straight_run(&style, width, arrowed)
    };
    let run = if westward && !bidir {
        mirror_run(&run)
    } else {
        run
    };
    canvas.stamp(
        canvas.band_top(sr) + canvas.row_height[sr] - 2,
        c,
        &centered(&run, width),
    );
    stamp_label(canvas, sr, c, westward, lines);
}

/// Stamp the shape:edge-stripped straight run (gently-cbk): the effective
/// `_draw_hor` START flags after the Scout.pm stripping — eastward the
/// source's `START_W` dies with a shape:edge source and the undirected
/// `START_E` dies with a shape:edge target; westward the roles mirror
/// (`START_E` is the source's, `START_W` the undirected target's) — then
/// the run and its label, the run NOT centered (it hangs off the
/// surviving border).
fn stamp_flagged(
    canvas: &mut Canvas,
    sr: usize,
    c: usize,
    westward: bool,
    edge_index: usize,
    lines: &[LabelLine],
    ends: EdgeNodeEnds,
) {
    let style = canvas.edge_style[edge_index].clone();
    let arrowed = canvas.edge_arrow_end[edge_index];
    let undirected = !arrowed && !canvas.edge_bidir[edge_index] && !start_arrow(canvas, edge_index);
    let (start_w, start_e) = if westward {
        (undirected && !ends.src && !ends.dst, !ends.src)
    } else {
        (!ends.src, undirected && !ends.src && !ends.dst)
    };
    let (run, x) = flagged_run(
        &style,
        canvas.col_width[c],
        canvas.offsets[c],
        arrowed,
        westward,
        undirected,
        (start_w, start_e),
    );
    let line = canvas.band_top(sr) + canvas.row_height[sr] - 2;
    canvas.stamp_at(line, canvas.offsets[c] + x, &run);
    stamp_label(canvas, sr, c, westward, lines);
}

/// Whether the edge carries a start-side arrowhead (a `<--` edge): the
/// canvas records only the end arrow and the bidirectional pair, so the
/// start-only arrow is recovered from the graph-side flag.
fn start_arrow(canvas: &Canvas, edge_index: usize) -> bool {
    canvas.edge_start_arrow[edge_index]
}

/// The straight short-cell run for an explicitly flagged cell (upstream
/// `_draw_hor` with the shape:edge-stripped flags, gently-cbk): the style
/// pattern fills the width (the undirected `rx % len` ofs shift applies —
/// directed SHORT cells skip it), one char drops per present START flag
/// (`START_W` also shifts the run one cell east), and a directed end
/// flag chops one more and replaces the new last char with the head.
/// Returns the run and its cell-x offset — a stripped cell is NOT
/// centered; the run hangs off the surviving border.
fn flagged_run(
    style: &str,
    w: usize,
    rx: usize,
    arrowed: bool,
    westward: bool,
    undirected: bool,
    (start_w, start_e): (bool, bool),
) -> (String, usize) {
    let pat = styles::edge_pattern(style);
    let len = pat.len();
    let mut line = pat.repeat(2 + w / len);
    if undirected {
        let ofs = rx % len;
        if ofs != 0 {
            line.replace_range(..ofs, "");
        }
    }
    line.truncate(w);
    let mut x = 0;
    if start_w {
        line.pop();
        x = 1;
    }
    if start_e {
        line.pop();
    }
    if westward {
        // END_W: the head rides the run's first two cells (` <` + fill).
        if arrowed {
            line.replace_range(..2, " <");
        }
    } else if arrowed {
        // END_E: chop one more, then the head takes the new last char.
        line.pop();
        line.pop();
        line.push('>');
    }
    (line, x)
}

/// Place a straight edge's aligned label lines in its gap column
/// (gently-527/gently-y7i): the field starts at byte offset 2 (upstream
/// `_draw_hor`'s `xs = 2`), is `width-5` bytes wide eastward (`ws =
/// xs + xr = 3`, the east arrowhead reserving the last byte) and
/// `width-4` westward (`ws = 2`, the west head reserving byte 1), and
/// the rows sit bottom-aligned in the band's `H-2` field lines
/// (upstream `align_ver => 'bottom'`) — a single line rides the band's
/// top row, two lines ride the top two rows. Widths count UTF-8 bytes
/// (gently-bvb).
fn stamp_label(canvas: &mut Canvas, sr: usize, c: usize, westward: bool, lines: &[LabelLine]) {
    if lines.is_empty() {
        return;
    }
    let field_w = canvas.col_width[c] - 5 + usize::from(westward);
    let start = canvas.band_top(sr) + canvas.row_height[sr].saturating_sub(2 + lines.len());
    for (i, line) in lines.iter().enumerate() {
        let len = byte_width(&line.text);
        let x = 2 + match line.align {
            Align::Left => 0,
            Align::Center => (field_w.saturating_sub(len)) / 2,
            Align::Right => field_w.saturating_sub(len),
        };
        canvas.stamp_at(start + i, canvas.offsets[c] + x, &line.text);
    }
}

/// The labelled straight run: the style's fill pattern widened to the
/// gap, with a `>` head only when the target end carries an arrowhead
/// (gently-bpq: the oracle draws no head on an undirected labeled run).
fn labelled_straight_run(style: &str, width: usize, arrowed: bool) -> String {
    let run_len = width - 2;
    let fill = edge_fill(style);
    let chars: Vec<char> = fill.chars().collect();
    let head = usize::from(arrowed);
    let mut s: String = (0..run_len - head)
        .map(|i| chars[i % chars.len()])
        .collect();
    if head == 1 {
        s.push('>');
    }
    s
}
/// The target-adjacent row index of a vertical gap-row run: 0 when the
/// target sits above (the run's first row), h-1 when it sits below.
fn target_row(upward: bool, h: usize) -> usize {
    if upward {
        0
    } else {
        h - 1
    }
}

/// Stamp the straight run of one VERTICAL edge through its single gap row
/// (gently-cbk, upstream `_draw_ver` for the EDGE_VER short cell): pipes
/// at the fixed offset-2 column down the gap band's full height, the
/// arrowhead at the target-adjacent end (`^` entering the target's south
/// from below, `v` its north from above; both for a bidirectional edge),
/// and the label lines left of the pipe on the band's middle line at
/// byte offset 4 (upstream `_insert_label`'s `xs = 4`) — left-aligned
/// (the edge `align` default), bottom-anchored around the middle line.
/// START flags never render on a vertical run (upstream: "starting
/// points are currently invisible").
pub(super) fn draw_straight_ver(
    canvas: &mut Canvas,
    gap_row: usize,
    c: usize,
    upward: bool,
    edge_index: usize,
    lines: &[LabelLine],
) {
    let h = canvas.row_height[gap_row];
    let top = canvas.band_top(gap_row);
    let x = canvas.vert_x(c);
    let arrowed = canvas.edge_arrow_end[edge_index];
    let bidir = canvas.edge_bidir[edge_index];
    for row in top..top + h {
        canvas.grid[row][x] = b'|';
    }
    // END_N puts the head on the run's first (target-adjacent) row, END_S
    // on its last — an upward run heads its first char, a downward one
    // its last; a bidirectional run heads both ends.
    let head_row = target_row(upward, h);
    if arrowed {
        canvas.grid[top + head_row][x] = if upward { b'^' } else { b'v' };
    }
    if bidir {
        canvas.grid[top + h - 1 - head_row][x] = if upward { b'v' } else { b'^' };
    }
    if lines.is_empty() {
        return;
    }
    let mid = canvas.mid(gap_row);
    let start = mid - (lines.len() - 1) / 2;
    for (i, line) in lines.iter().enumerate() {
        canvas.stamp_at(start + i, canvas.offsets[c] + 4, &line.text);
    }
}

// The per-edge dispatch and the `shape: edge` node cell moved here from
// canvas.rs past the tidy file limit (gently-cbk).

/// Which endpoint of an edge is a `shape: edge` node (gently-cbk) — the
/// Scout.pm endpoint-flag stripping signal passed to the edge drawers.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct EdgeNodeEnds {
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
            let (span_f, span_t) = (self.spans[edge.from], self.spans[edge.to]);
            if sr == tr && (tc == sc + span_f.0 + 1 || sc == tc + span_t.0 + 1) {
                // The run rides the PATH's row (gently-awd): a stacked
                // parallel channel's single cell sits on a continuation
                // row of the boxes' row span, south of the band row.
                self.draw_gap_column(
                    path[0].1,
                    path[0].0,
                    tc < sc,
                    edge_index,
                    edge.from,
                    edge.to,
                );
                return Ok(());
            }
            if sc == tc && sr.abs_diff(tr) == 2 {
                self.draw_gap_row(path[0].1, path[0].0, tr < sr, edge_index);
                return Ok(());
            }
        }
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
