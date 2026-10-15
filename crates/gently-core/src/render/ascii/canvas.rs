//! Purpose: the char-grid canvas the ascii renderer draws on — the
//! blank grid, per-node box stamping, per-edge dispatch, selfloop
//! elbows and the polyline plumbing.
//! Responsibilities: build the blank grid from the layout's cell grid
//! (column widths per `geometry`, widened by group titles and expanded
//! by the group-frame margins), stamp bordered/rounded/point/invisible
//! nodes and edge runs, stamp the group frames, and join the lines with
//! per-line trailing-space trimming (the oracle does not pad lines).
//! Rationale: split from monolithic mod.rs past the tidy file limit; the
//! glyph tables live in styles.rs, the label/style/column resolution in
//! geometry.rs, the group frames in groups.rs (gently-b0z).

use super::geometry::{
    band_layout, column_geometry, column_offsets, edge_styles, is_anon, label_lines, node_styles,
    selfloops,
};
use super::groups::{self, Frame};
use super::labels::{Align, LabelLine};
use super::styles::{self, border, byte_width, h_run, Shape};
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::{Cell, Layout};
use std::collections::HashMap;

/// The grid-cell span of every node, in model order (gently-awd): the
/// `columns`/`rows` attributes as grid cells — upstream `_calc_size`
/// reads `abs($attr || 1)`, so an absent/empty/0 value is one cell.
pub(super) fn node_spans(graph: &Graph) -> Vec<(usize, usize)> {
    (0..graph.nodes.len())
        .map(|i| {
            let side = |key: &str| {
                graph
                    .get_attr(Scope::Node(i), key)
                    .and_then(|v| v.parse::<usize>().ok())
                    .filter(|&n| n > 0)
                    .unwrap_or(1)
            };
            (side("columns"), side("rows"))
        })
        .collect()
}

/// Width in chars of a gap (edge-routing) grid column.
pub(super) const GAP_WIDTH: usize = 5;
/// Minimum height in lines of one grid row's char band (border, label,
/// border) — grown when a label in the row spans more lines (gently-527:
/// upstream sizes every cell from its label's line count and grows the
/// row to the tallest cell, `Layout::Grid::_prepare_layout`).
pub(super) const BAND_HEIGHT: usize = 3;

/// The blank char grid plus the column-width mapping it is drawn against.
pub(super) struct Canvas {
    /// Byte grid: rows of UTF-8 bytes — label arithmetic counts UTF-8
    /// bytes the way the oracle's byte-string `length()` does
    /// (gently-bvb), so a wide glyph occupies its byte length in the
    /// row, not its display columns.
    pub(super) grid: Vec<Vec<u8>>,
    /// Char-line height of each grid row's band (≥ [`BAND_HEIGHT`]).
    pub(super) row_height: Vec<usize>,
    /// Unshifted start line of each band (prefix of `row_height`),
    /// before the group-frame margin shift.
    pub(super) band_tops: Vec<usize>,
    /// Start char offset of each grid column (`offsets[c]`; length width+1).
    pub(super) offsets: Vec<usize>,
    /// Char width of each grid column.
    pub(super) col_width: Vec<usize>,
    /// Resolved label lines per node, in model order (gently-527/y7i).
    pub(super) node_lines: Vec<Vec<LabelLine>>,
    /// Aligned label lines per edge, in model order — `Some` only for a
    /// straight horizontal labelled edge (the full-gap label cell);
    /// bent and vertical edges keep their single-line surface.
    pub(super) edge_lines: Vec<Option<Vec<LabelLine>>>,
    /// Resolved render data per node, in model order.
    pub(super) node_style: Vec<(String, Shape)>,
    /// Grid-cell span of every node, in model order (gently-awd).
    pub(super) spans: Vec<(usize, usize)>,
    /// Resolved edge style per edge, in model order.
    pub(super) edge_style: Vec<String>,
    /// Target-side arrowhead presence per edge, in model order.
    pub(super) edge_arrow_end: Vec<bool>,
    /// Source-side arrowhead presence per edge, in model order (gently-cbk:
    /// the undirected test for the shape:edge flag stripping).
    pub(super) edge_start_arrow: Vec<bool>,
    /// Bidirectionality (arrowheads at BOTH ends) per edge, in model order.
    pub(super) edge_bidir: Vec<bool>,
    /// Group frames in unshifted coordinates, stamped after the edges.
    pub(super) frames: Vec<Frame>,
    /// Left/top canvas margins the frames demanded.
    pub(super) margin_left: usize,
    pub(super) margin_top: usize,
    /// Last drawn line any node box can reach (its box-bottom border line
    /// or the border:none blank beneath the label) — trailing blank lines
    /// beyond it are dropped (see `new`).
    last_band_floor: usize,
}

/// The drawn geometry of one node: its anchor offset and top line plus
/// the summed width/height over its grid-cell span (gently-awd).
pub(super) struct NodeGeom {
    pub(super) x0: usize,
    pub(super) top: usize,
    pub(super) w: usize,
    pub(super) h: usize,
}

/// The point node's label row: blank sides with a centered `*`.
fn centered_point_row(inner: usize) -> String {
    let left = (inner - 1) / 2;
    format!(" {}*{} ", " ".repeat(left), " ".repeat(inner - 1 - left))
}

/// The style's border with all four corners blanked (the `rounded`
/// shape's ASCII rendering: ` --- ` top and bottom rows).
fn blank_corners(b: styles::Border) -> styles::Border {
    styles::Border {
        tl: ' ',
        tr: ' ',
        br: ' ',
        bl: ' ',
        ..b
    }
}

/// `text` centered in a span of `width` byte cells.
pub(super) fn centered(text: &str, width: usize) -> String {
    let len = super::styles::byte_width(text);
    let left = (width - len) / 2;
    format!(
        "{}{}{}",
        " ".repeat(left),
        text,
        " ".repeat(width - len - left)
    )
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

/// The group-frame margins (c8), the blank byte grid padded by them,
/// the offsets shifted by the left margin, and the last drawable line.
#[allow(clippy::type_complexity)]
fn frame_grid(
    graph: &Graph,
    layout: &Layout,
    col_width: &[usize],
    offsets: &[usize],
    band_tops: &[usize],
    row_height: &[usize],
    spans: &[(usize, usize)],
) -> (Vec<Vec<u8>>, Vec<usize>, Vec<Frame>, usize, usize, usize) {
    let (frames, margins) =
        groups::frame_margins(graph, layout, col_width, offsets, band_tops, row_height);
    let (ml, mt, mr, mb) = margins;
    let total_lines = band_tops[layout.height];
    let (grid, offsets) = shifted_grid(offsets.to_vec(), layout.width, ml, mr, mt, mb, total_lines);
    let draws: Vec<bool> = (0..graph.nodes.len())
        .map(|i| !is_anon(&graph.nodes[i].name))
        .collect();
    let last_band_floor =
        last_band_floor(layout, &draws, row_height, band_tops, grid.len(), mt, spans);
    (grid, offsets, frames, ml, mt, last_band_floor)
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
    /// margins, then one blank byte row per canvas line.
    pub(super) fn new(graph: &Graph, layout: &Layout) -> Canvas {
        let selfloop_node = selfloops(graph);
        let node_style = node_styles(graph);
        let edge_style = edge_styles(graph);
        // The layout's own spans (gently-byq): the grown row spans of
        // indexed-port nodes live on the Layout — re-deriving them from
        // the attrs would shrink the boxes back.
        let spans = if layout.spans.len() == graph.nodes.len() {
            layout.spans.clone()
        } else {
            node_spans(graph)
        };
        // label lines resolve FIRST (gently-emd: the wrapped lines drive
        // the box widths), then the column geometry consumes them
        let (node_lines, edge_lines) = label_lines(graph, layout, &spans);
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
        // Row band heights + their unshifted start lines: every cell is
        // grown to the tallest cell of its row (upstream
        // `_prepare_layout` rewrites each cell's h to the row height).
        let edge_rows: Vec<usize> = layout
            .edge_paths
            .iter()
            .flat_map(|p| p.iter().map(|&(_, r)| r))
            .collect();
        let (row_height, band_tops) =
            band_layout(graph, layout, &node_lines, &edge_lines, &spans, &edge_rows);
        let (grid, offsets, frames, ml, mt, last_band_floor) = frame_grid(
            graph,
            layout,
            &col_width,
            &offsets,
            &band_tops,
            &row_height,
            &spans,
        );
        let (edge_arrow_end, edge_start_arrow, edge_bidir) = edge_arrow_flags(graph);
        Canvas {
            grid,
            row_height,
            band_tops,
            node_lines,
            edge_lines,
            offsets,
            col_width,
            node_style,
            edge_style,
            spans,
            edge_arrow_end,
            edge_start_arrow,
            edge_bidir,
            frames,
            margin_left: ml,
            margin_top: mt,
            last_band_floor,
        }
    }

    /// Stamp every node's box per its resolved shape: bordered box,
    /// rounded box (blank corners), point (`*`), edge cell, or invisible
    /// (nothing).
    pub(super) fn draw_nodes(&mut self, _graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            // The node's drawn geometry spans its whole grid-cell span
            // (gently-awd): the box covers the summed column widths and
            // row heights from its anchor cell.
            let (span_c, span_r) = self.spans[idx];
            let geom = NodeGeom {
                x0: self.offsets[cx],
                top: self.band_top(cy),
                w: (cx..cx + span_c)
                    .map(|c| self.col_width.get(c).copied().unwrap_or(0))
                    .sum(),
                h: (cy..cy + span_r)
                    .map(|r| self.row_height.get(r).copied().unwrap_or(0))
                    .sum(),
            };
            let lines = self.node_lines[idx].clone();
            let (style, shp) = self.node_style[idx].clone();
            let b = border(&style);
            match shp {
                Shape::Invisible => {}
                Shape::Point => self.draw_point(&geom),
                Shape::Rounded => self.draw_box(&geom, &lines, blank_corners(b), false),
                Shape::Edge => self.draw_edge_cell(&geom, &lines),
                Shape::Box => self.draw_box(&geom, &lines, b, style == "none"),
            }
        }
    }

    /// Stamp one bordered box: top border, the label rows (one per
    /// aligned-label line, gently-527/y7i), bottom border. The box fills
    /// the row band's height — a multiline label or a taller
    /// neighbor's label grows the band, and the label rows center
    /// vertically in the grown box (upstream `_draw_label`'s middle
    /// alignment with `ys = 0.5`: rows `1 + int((H-2-lines)/2 + i)` for
    /// a bordered box). Each line is placed by its `l`/`r`/`c`
    /// alignment inside the label field (`x0 = 2`, width `w-4`;
    /// `none` borders widen the field to `w-2` at `x0 = 1`); widths
    /// count UTF-8 bytes (gently-bvb). `none` is a `border: none` box:
    /// its field widens to `w-2` at `x0 = 1` (upstream `_draw_label`).
    fn draw_box(&mut self, geom: &NodeGeom, lines: &[LabelLine], b: styles::Border, none: bool) {
        let w = geom.w;
        let inner = w - 2;
        let run = h_run(&b, inner);
        let top_line = geom.top;
        let h = geom.h;
        self.stamp_at(top_line, geom.x0, &format!("{}{}{}", b.tl, run, b.tr));
        self.stamp_at(
            top_line + h - 1,
            geom.x0,
            &format!("{}{}{}", b.bl, run, b.br),
        );
        let (field_x, field_w, label_h, y_pad) = if none {
            (1, w - 2, h, 0)
        } else {
            (2, w - 4, h - 2, 1)
        };
        let start = top_line + y_pad + (label_h.saturating_sub(lines.len())) / 2;
        let x0 = geom.x0;
        // the box's side glyphs run the FULL grown height — every
        // interior row carries them (the oracle's box is drawn border
        // first, then the label over it)
        for row in top_line + 1..top_line + h - 1 {
            self.stamp_at(row, x0, &b.side.to_string());
            self.stamp_at(row, x0 + w - 1, &b.side.to_string());
        }
        for (i, line) in lines.iter().enumerate() {
            let row = start + i;
            self.stamp_at(row, x0, &b.side.to_string());
            self.stamp_at(row, x0 + w - 1, &b.side.to_string());
            let len = byte_width(&line.text);
            let x = field_x
                + match line.align {
                    Align::Left => 0,
                    Align::Center => (field_w.saturating_sub(len)) / 2,
                    Align::Right => field_w.saturating_sub(len),
                };
            self.stamp_at(row, x0 + x, &line.text);
        }
    }

    /// Stamp a point node: blank borders, a centered `*` (oracle bytes:
    /// blank, `  *`, blank for a width-5 point node) on the band's
    /// vertically centered line.
    fn draw_point(&mut self, geom: &NodeGeom) {
        let mid = centered_point_row(geom.w - 2);
        self.stamp_at(geom.top + (geom.h - 1) / 2, geom.x0, &mid);
    }

    /// Draw a selfloop: a small elbow over the box, departing through the
    /// east corner and arriving through the west one with a `v` arrowhead.
    pub(super) fn draw_selfloop(&mut self, (c, r): (usize, usize)) {
        let x0 = self.offsets[c];
        let w = self.col_width[c];
        let y_mid = self.mid(r - 1);
        let y_attach = self.band_top(r) - 1;
        let (xl, xr) = (x0 + 2, x0 + w - 3);
        for x in xl + 1..xr {
            self.grid[y_mid][x] = b'-';
        }
        self.grid[y_mid][xl] = b'+';
        self.grid[y_mid][xr] = b'+';
        self.grid[y_attach][xr] = b'|';
        self.grid[y_attach][xl] = b'v';
    }

    /// Char line of band row `r`'s top border line (margin-shifted).
    pub(super) fn band_top(&self, r: usize) -> usize {
        self.band_tops[r] + self.margin_top
    }

    /// Char line of row `r`'s middle (label) line — the vertically
    /// centered line of the grown band (a one-line label centers at
    /// `int((H-1)/2)`, which is the standard band's `top+1`).
    pub(super) fn mid(&self, r: usize) -> usize {
        self.band_top(r) + (self.row_height[r] - 1) / 2
    }

    /// Char line one past a node box's bottom border over its row span
    /// (gently-awd) — for a 1×1 node this is `band_below`.
    pub(super) fn box_below(&self, r: usize, span_r: usize) -> usize {
        let h: usize = (r..r + span_r)
            .map(|rr| self.row_height.get(rr).copied().unwrap_or(0))
            .sum();
        self.band_top(r) + h
    }

    /// Stamp an INDEXED-port slot edge's label onto its arrow-end cell
    /// (gently-byq, the decoded upstream `_insert_label` placement): a
    /// vertical label cell rides the pipe's east side — 2 chars right of
    /// the `vert_x` glyph, on the cell row's middle line; a horizontal
    /// label cell rides one line above its run line, at offset +2 from
    /// the column start. The SO fixture pins `3. X Msg` at (44,4) — the
    /// `^` cell's east — and `2. read X` at (16,6) — above the `<` run.
    pub(super) fn stamp_slot_label(&mut self, graph: &Graph, layout: &Layout, ei: usize) {
        let Some(&(c, r)) = layout.slot_labels[ei].as_ref() else {
            return;
        };
        let Some(label) = graph.get_attr(Scope::Edge(ei), "label").map(str::to_owned) else {
            return;
        };
        // Orientation: vertical when a path neighbour shares the cell's
        // column (the same rule the column widths use).
        let path = &layout.edge_paths[ei];
        let idx = path.iter().position(|&p| p == (c, r)).unwrap_or(0);
        let vertical =
            (idx > 0 && path[idx - 1].0 == c) || path.get(idx + 1).is_some_and(|&n| n.0 == c);
        let lines = super::labels::object_label_lines(
            graph,
            Scope::Edge(ei),
            ObjectKind::Edge,
            &label,
            super::labels::default_align(graph, Scope::Edge(ei), ObjectKind::Edge),
        );
        let base_y = if vertical {
            self.mid(r)
        } else {
            self.mid(r).saturating_sub(1)
        };
        let start = base_y + 1 - lines.len();
        let field_w = self.col_width[c] - usize::from(vertical) * 2 - 2;
        for (i, line) in lines.iter().enumerate() {
            let len = super::styles::byte_width(&line.text);
            let off = match line.align {
                super::labels::Align::Left => 0,
                super::labels::Align::Center => field_w.saturating_sub(len) / 2,
                super::labels::Align::Right => field_w.saturating_sub(len),
            };
            let x = self.offsets[c] + 2 + usize::from(vertical) * 2 + off;
            self.stamp_at(start + i, x, &line.text);
        }
    }

    /// Stamp the shared CROSS cells (gently-byq): where two edges' paths
    /// share a cell and traverse it on different axes — one run
    /// horizontal, the other vertical — the oracle draws a `+` junction
    /// at their intersection (the SO fixture's cell (2,2): the `^`
    /// vertical and the `<` run cross, rendered `+` at char 42, line 7).
    pub(super) fn stamp_crossings(&mut self, layout: &Layout) {
        let mut at: HashMap<Cell, [bool; 2]> = HashMap::new();
        for path in &layout.edge_paths {
            for (i, &(c, r)) in path.iter().enumerate() {
                let vertical =
                    (i > 0 && path[i - 1].0 == c) || path.get(i + 1).is_some_and(|&n| n.0 == c);
                let slot = at.entry((c, r)).or_insert([false, false]);
                if vertical {
                    slot[1] = true;
                } else {
                    slot[0] = true;
                }
            }
        }
        for ((c, r), &[h, v]) in at.iter() {
            if !h || !v {
                continue;
            }
            let (x, y) = (self.vert_x(*c), self.mid(*r));
            if y < self.grid.len() && x < self.grid[0].len() {
                self.grid[y][x] = b'+';
            }
        }
    }

    /// Char column of a vertical run through grid column `c`: offset +2
    /// from the cell's left edge — upstream edge cells draw their vertical
    /// line at fixed offset 2 (`As_ascii::_draw_cross`:
    /// `_printfb_ver($fb, 2, 0, $line)`), not at the node's center, so a
    /// wide box's north/south attachment sits 2 chars in from its left
    /// border (gently-klf: tests/repro/claims/pod-triangle.observed,
    /// pod-fan.observed).
    pub(super) fn vert_x(&self, c: usize) -> usize {
        self.offsets[c] + 2
    }

    /// Write `text` starting at grid column `cx`'s byte offset on `line`.
    pub(super) fn stamp(&mut self, line: usize, cx: usize, text: &str) {
        let x0 = self.offsets[cx];
        self.stamp_at(line, x0, text);
    }

    /// Write `text` starting at absolute byte offset `x` on `line`,
    /// bounds-guarded (never panics; typed render philosophy).
    pub(super) fn stamp_at(&mut self, line: usize, x: usize, text: &str) {
        if line >= self.grid.len() {
            return;
        }
        for (k, byte) in text.bytes().enumerate() {
            if x + k < self.grid[line].len() {
                self.grid[line][x + k] = byte;
            }
        }
    }

    /// Join the grid lines, each with one trailing newline. Trailing
    /// filler spaces are trimmed per line — the oracle does not pad lines
    /// out to the canvas width (the isolated node's box row is just
    /// `+---+`).
    pub(super) fn into_string(self) -> String {
        // Trailing blank lines never render: the oracle's as_ascii ends at
        // the last drawn line, so a trailing gap row's unused lines are
        // dropped (the port-* fixtures' dip-below runs end the output).
        let mut lines: &[Vec<u8>] = &self.grid;
        while lines.len() > self.last_band_floor + 1 {
            match lines.last() {
                Some(last) if last.iter().all(|&c| c == b' ') => {
                    lines = &lines[..lines.len() - 1];
                }
                _ => break,
            }
        }
        let mut out = String::new();
        for line in lines {
            let text = String::from_utf8_lossy(line);
            out.push_str(text.trim_end_matches(' '));
            out.push('\n');
        }
        out
    }
}
