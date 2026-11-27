//! Purpose: node stamping — bordered/rounded/point/invisible boxes and
//! selfloop elbows.
//! Responsibilities: dispatch each node by shape, stamp borders, side
//! glyphs, and aligned label rows, and draw selfloop elbows.
//! Rationale: split from canvas.rs past the tidy file limit (gently-b0z).

use super::super::labels::{Align, LabelLine};
use super::super::styles::{self, border, byte_width, h_run, Shape};
use super::super::Canvas;
use crate::graph::Graph;
use crate::layout::Layout;

/// The label field inside a box: `(x0, width, label-row height, y pad)`.
/// A `border: none` box widens the field to `w-2` at `x0 = 1` with no
/// y pad (upstream `_draw_label`); a bordered one is `x0 = 2`, `w-4`.
fn label_field(w: usize, h: usize, none: bool) -> (usize, usize, usize, usize) {
    if none {
        (1, w - 2, h, 0)
    } else {
        (2, w - 4, h - 2, 1)
    }
}

/// The x offset of one label line inside its field (`field_x` base).
fn label_x(field_x: usize, field_w: usize, align: Align, len: usize) -> usize {
    field_x
        + match align {
            Align::Left => 0,
            Align::Center => (field_w.saturating_sub(len)) / 2,
            Align::Right => field_w.saturating_sub(len),
        }
}

impl Canvas {
    /// Stamp every node's box per its resolved shape: bordered box,
    /// rounded box (blank corners), point (`*`), edge cell, or invisible
    /// (nothing).
    pub(in crate::render::ascii) fn draw_nodes(&mut self, _graph: &Graph, layout: &Layout) {
        for (idx, &(cx, cy)) in layout.node_cells.iter().enumerate() {
            let geom = self.node_geom(idx, cx, cy);
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

    /// The node's drawn geometry spans its whole grid-cell span
    /// (gently-awd): the box covers the summed column widths and row
    /// heights from its anchor cell.
    fn node_geom(&self, idx: usize, cx: usize, cy: usize) -> super::NodeGeom {
        let (span_c, span_r) = self.spans[idx];
        super::NodeGeom {
            x0: self.offsets[cx],
            top: self.band_top(cy),
            w: (cx..cx + span_c)
                .map(|c| self.col_width.get(c).copied().unwrap_or(0))
                .sum(),
            h: (cy..cy + span_r)
                .map(|r| self.row_height.get(r).copied().unwrap_or(0))
                .sum(),
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
    fn draw_box(
        &mut self,
        geom: &super::NodeGeom,
        lines: &[LabelLine],
        b: styles::Border,
        none: bool,
    ) {
        let w = geom.w;
        self.stamp_box_border(geom, &b);
        let (field_x, field_w, label_h, y_pad) = label_field(w, geom.h, none);
        let start = geom.top + y_pad + (label_h.saturating_sub(lines.len())) / 2;
        // The box's side glyphs run the FULL grown height — every
        // interior row carries them (the oracle's box is drawn border
        // first, then the label over it).
        self.stamp_box_sides(geom.top + 1..geom.top + geom.h - 1, geom.x0, w, b.side);
        for (i, line) in lines.iter().enumerate() {
            self.stamp_label_row(start + i, line, geom.x0, w, &b, field_x, field_w);
        }
    }

    /// Stamp the two side glyphs on every row of `rows`.
    fn stamp_box_sides(&mut self, rows: std::ops::Range<usize>, x0: usize, w: usize, side: char) {
        for row in rows {
            self.stamp_at(row, x0, &side.to_string());
            self.stamp_at(row, x0 + w - 1, &side.to_string());
        }
    }

    /// Stamp one label row: the side glyphs it covers, then the text at
    /// its alignment inside the label field.
    #[allow(clippy::too_many_arguments)]
    fn stamp_label_row(
        &mut self,
        row: usize,
        line: &LabelLine,
        x0: usize,
        w: usize,
        b: &styles::Border,
        field_x: usize,
        field_w: usize,
    ) {
        self.stamp_at(row, x0, &b.side.to_string());
        self.stamp_at(row, x0 + w - 1, &b.side.to_string());
        let len = byte_width(&line.text);
        let x = label_x(field_x, field_w, line.align, len);
        self.stamp_at(row, x0 + x, &line.text);
    }

    /// The box's top and bottom border rows.
    fn stamp_box_border(&mut self, geom: &super::NodeGeom, b: &styles::Border) {
        let run = h_run(b, geom.w - 2);
        self.stamp_at(geom.top, geom.x0, &format!("{}{}{}", b.tl, run, b.tr));
        self.stamp_at(
            geom.top + geom.h - 1,
            geom.x0,
            &format!("{}{}{}", b.bl, run, b.br),
        );
    }

    fn draw_point(&mut self, geom: &super::NodeGeom) {
        let mid = centered_point_row(geom.w - 2);
        self.stamp_at(geom.top + (geom.h - 1) / 2, geom.x0, &mid);
    }

    /// Draw a selfloop: a small elbow over the box, departing through the
    /// east corner and arriving through the west one with a `v` arrowhead.
    pub(in crate::render::ascii) fn draw_selfloop(&mut self, (c, r): (usize, usize)) {
        let x0 = self.offsets[c];
        let w = self.col_width[c];
        let y_mid = self.mid(r - 1);
        let y_attach = self.band_top(r) - 1;
        let (xl, xr) = (x0 + 2, x0 + w - 3);
        for cell in &mut self.grid[y_mid][xl + 1..xr] {
            *cell = b'-';
        }
        self.grid[y_mid][xl] = b'+';
        self.grid[y_mid][xr] = b'+';
        self.grid[y_attach][xr] = b'|';
        self.grid[y_attach][xl] = b'v';
    }
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
