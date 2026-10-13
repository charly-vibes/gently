//! Purpose: the dashed group-frame overlay (ge.ascii_render.c8, bonus
//! bead gently-7rx) — the `+ - - +` box with the `'` side glyphs the
//! oracle draws around a group's member nodes, plus the `Name:` title
//! band above them.
//! Responsibilities: compute each named group's frame extents from its
//! members' placed boxes, the canvas margins the frames demand, the
//! per-column title-width constraints, and stamping the frames onto the
//! finished grid.
//! Rationale: gently's layout has no group cells (groups are a model
//! stub), so the frame is drawn by the renderer as an overlay around the
//! member bounding box — one char/row of padding plus the 3-row title
//! band above and the blank row + border below, pinned byte-for-byte by
//! tests/repro/claims/pod-group.observed and the probed relatives
//! (single member, stacked members, long titles widening the westmost
//! member's column: `Cities:` → 9 columns, `Verylonggroupname:` → 20).

use super::canvas::Canvas;
use super::styles::{border, byte_width, h_run};
use crate::graph::Graph;
use crate::layout::Layout;

/// One group's frame extents in unshifted char coordinates (negative
/// before the canvas margin shift).
pub(super) struct Frame {
    left: isize,
    right: isize,
    top: isize,
    bottom: isize,
    title: String,
    title_x: isize,
}

/// The group's title text: the group's stored name — upstream's parser
/// keeps the `Name:` colon in the group name (probed: `$grp->{name}` is
/// `Cities:` for the POD example), and the title cell renders it
/// verbatim.
fn title_text(name: &str) -> String {
    name.to_string()
}

/// Column-width constraints from the group titles: the title cell sits
/// in the leftmost member's grid column and its `label+2` minimum
/// widens that column — the member box then fills the widened column
/// with its label re-centered (observed: `Cities:` widens the westmost
/// member's box from 8 to 9 columns, `Verylonggroupname:` to 20).
pub(super) fn title_widths(graph: &Graph, layout: &Layout) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for group in &graph.groups {
        if group.name.is_empty() || group.members.is_empty() {
            continue;
        }
        let leftmost = group
            .members
            .iter()
            .map(|&m| layout.node_cells[m].0)
            .min()
            .expect("non-empty member list");
        let w = byte_width(&title_text(&group.name)) + 2;
        out.push((leftmost, w));
    }
    out
}

/// The frame of every named group with members: the bounding box of the
/// member boxes expanded by one char/row of frame padding, with the
/// 3-row title band (border, title, blank) above and the blank row +
/// border below. The title starts at the leftmost member's box x.
pub(super) fn frames(
    graph: &Graph,
    layout: &Layout,
    col_width: &[usize],
    offsets: &[usize],
    band_tops: &[usize],
    row_height: &[usize],
) -> Vec<Frame> {
    let mut out = Vec::new();
    for group in &graph.groups {
        if group.name.is_empty() || group.members.is_empty() {
            continue;
        }
        let mut left = isize::MAX;
        let mut right = isize::MIN;
        let mut top = isize::MAX;
        let mut bottom = isize::MIN;
        for &m in &group.members {
            let (cx, cy) = layout.node_cells[m];
            left = left.min(offsets[cx] as isize);
            right = right.max((offsets[cx] + col_width[cx]) as isize - 1);
            top = top.min(band_tops[cy] as isize);
            bottom = bottom.max((band_tops[cy] + row_height[cy] - 1) as isize);
        }
        out.push(Frame {
            left: left - 2,
            right: right + 2,
            top: top - 3,
            bottom: bottom + 2,
            title: title_text(&group.name),
            title_x: left,
        });
    }
    out
}

/// Each named group's frame plus the canvas margins those frames demand
/// over the unshifted grid of `width` bytes by `total_lines` lines
/// (c8) — the composite the canvas builds its blank grid from.
pub(super) fn frame_margins(
    graph: &Graph,
    layout: &Layout,
    col_width: &[usize],
    offsets: &[usize],
    band_tops: &[usize],
    row_height: &[usize],
) -> (Vec<Frame>, (usize, usize, usize, usize)) {
    let frames = frames(graph, layout, col_width, offsets, band_tops, row_height);
    let margins = margins(&frames, offsets[layout.width], band_tops[layout.height]);
    (frames, margins)
}

/// The canvas margins (left, top, right, bottom) the frames demand over
/// the unshifted grid of `width` chars by `height` lines — never
/// negative (the margins only grow the canvas).
pub(super) fn margins(
    frames: &[Frame],
    width: usize,
    height: usize,
) -> (usize, usize, usize, usize) {
    let mut ml = 0isize;
    let mut mt = 0isize;
    let mut mr = 0isize;
    let mut mb = 0isize;
    for f in frames {
        ml = ml.max(-f.left);
        mt = mt.max(-f.top);
        mr = mr.max(f.right + 1 - width as isize);
        mb = mb.max(f.bottom + 1 - height as isize);
    }
    (
        ml.max(0) as usize,
        mt.max(0) as usize,
        mr.max(0) as usize,
        mb.max(0) as usize,
    )
}

impl Canvas {
    /// Stamp the group frames: dashed border rows (corners `+`, the
    /// dashed style's `- ` run shifted one — the node-border rule), a
    /// `'` side at both ends of every row between, and the title on the
    /// row under the top border. Everything shifts by the canvas
    /// margins.
    pub(super) fn stamp_frames(&mut self) {
        let b = border("dashed");
        let frames = std::mem::take(&mut self.frames);
        let (ml, mt) = (self.margin_left, self.margin_top);
        for f in &frames {
            let l = (f.left + ml as isize).max(0) as usize;
            let r = (f.right + ml as isize).max(0) as usize;
            let t = (f.top + mt as isize).max(0) as usize;
            let bm = (f.bottom + mt as isize).max(0) as usize;
            let run = h_run(&b, r.saturating_sub(l + 1));
            let border_row = format!("+{run}+");
            self.stamp_at(t, l, &border_row);
            self.stamp_at(bm, l, &border_row);
            for y in t + 1..bm {
                self.stamp_at(y, l, "'");
                self.stamp_at(y, r, "'");
            }
            if !f.title.is_empty() {
                let x = (f.title_x + ml as isize).max(0) as usize;
                self.stamp_at(t + 1, x, &f.title);
            }
        }
    }
}
