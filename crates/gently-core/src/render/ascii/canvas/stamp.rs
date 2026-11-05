//! Purpose: label and crossing stamping — indexed-port slot-edge labels
//! and the shared CROSS cells.
//! Responsibilities: resolve slot-edge labels, place them by orientation,
//! and mark axis-crossing path cells with `+` junctions.
//! Rationale: split from canvas.rs past the tidy file limit (gently-b0z).

use super::super::labels::{default_align, object_label_lines, Align};
use super::super::styles::byte_width;
use super::Canvas;
use crate::graph::{Graph, ObjectKind, Scope};
use crate::layout::Layout;
use std::collections::HashMap;

impl Canvas {
    /// Stamp an INDEXED-port slot edge's label onto its arrow-end cell
    /// (gently-byq, the decoded upstream `_insert_label` placement): a
    /// vertical label cell rides the pipe's east side — 2 chars right of
    /// the `vert_x` glyph, on the cell row's middle line; a horizontal
    /// label cell rides one line above its run line, at offset +2 from
    /// the column start. The SO fixture pins `3. X Msg` at (44,4) — the
    /// `^` cell's east — and `2. read X` at (16,6) — above the `<` run.
    pub(in crate::render::ascii) fn stamp_slot_label(
        &mut self,
        graph: &Graph,
        layout: &Layout,
        ei: usize,
    ) {
        let Some(&(c, r)) = layout.slot_labels[ei].as_ref() else {
            return;
        };
        let Some(label) = graph.get_attr(Scope::Edge(ei), "label").map(str::to_owned) else {
            return;
        };
        let vertical = slot_vertical(&layout.edge_paths[ei], c, r);
        let lines = slot_label_lines(graph, ei, &label);
        let base_y = if vertical {
            self.mid(r)
        } else {
            self.mid(r).saturating_sub(1)
        };
        let start = base_y + 1 - lines.len();
        for (i, line) in lines.iter().enumerate() {
            let x = self.slot_label_x(line, c, vertical);
            self.stamp_at(start + i, x, &line.text);
        }
    }

    /// The char offset of one slot-label line in its column: 2 chars in
    /// from the column start, +2 more on the east side of a vertical run.
    fn slot_label_x(
        &self,
        line: &super::super::labels::LabelLine,
        c: usize,
        vertical: bool,
    ) -> usize {
        let len = byte_width(&line.text);
        let field_w = self.col_width[c] - usize::from(vertical) * 2 - 2;
        let off = match line.align {
            Align::Left => 0,
            Align::Center => field_w.saturating_sub(len) / 2,
            Align::Right => field_w.saturating_sub(len),
        };
        self.offsets[c] + 2 + usize::from(vertical) * 2 + off
    }

    /// Stamp the shared CROSS cells (gently-byq): where two edges' paths
    /// share a cell and traverse it on different axes — one run
    /// horizontal, the other vertical — the oracle draws a `+` junction
    /// at their intersection (the SO fixture's cell (2,2): the `^`
    /// vertical and the `<` run cross, rendered `+` at char 42, line 7).
    pub(in crate::render::ascii) fn stamp_crossings(&mut self, layout: &Layout) {
        let at = crossing_axes(layout);
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
}

/// Orientation of the slot-label cell `(c, r)` on its edge's path:
/// vertical when a path neighbour shares the cell's column (the same
/// rule the column widths use).
fn slot_vertical(path: &[crate::layout::Cell], c: usize, r: usize) -> bool {
    let idx = path.iter().position(|&p| p == (c, r)).unwrap_or(0);
    (idx > 0 && path[idx - 1].0 == c) || path.get(idx + 1).is_some_and(|&n| n.0 == c)
}

/// The wrapped label lines of edge `ei` (defaults applied).
fn slot_label_lines(graph: &Graph, ei: usize, label: &str) -> Vec<super::super::labels::LabelLine> {
    object_label_lines(
        graph,
        Scope::Edge(ei),
        ObjectKind::Edge,
        label,
        default_align(graph, Scope::Edge(ei), ObjectKind::Edge),
    )
}

/// The axes each edge-path cell is traversed on: `[horizontal, vertical]`.
fn crossing_axes(layout: &Layout) -> HashMap<crate::layout::Cell, [bool; 2]> {
    let mut at: HashMap<crate::layout::Cell, [bool; 2]> = HashMap::new();
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
    at
}
