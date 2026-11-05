//! Purpose: port classification and span growth — the grid-cell span of
//! each node and how indexed/side ports grow it.
//! Responsibilities: read the `columns`/`rows` attributes, classify
//! every edge end's port spelling, and grow under-provisioned nodes
//! until every free port has a face.
//! Rationale: split from layout.rs past the tidy file limit (gently-cbk);
//! shaped by gently-byq (indexed slots) and gently-a6g (free-ports
//! growth, upstream Node.pm `_grow`).

use super::Route;
use crate::graph::{Graph, Scope};

/// Which way a port-constrained edge must dip (gently-89r). Upstream
/// Attributes.pm `_flow_as_side` maps the flow-relative spellings onto
/// sides: right = inflow+90 = south, left = north, front/forward = the
/// flow direction itself (the default attachment), back = the opposite;
/// absolute spellings (south/north/east/west, down/up, degrees) map
/// through `_direction_as_side`. Only the east pipeline honors ports (the
/// non-east flows realize a transformed canonical layout — no tier-1
/// fixture pins ports there), so the sides are read in flow space and
/// matched against the canonical east axes. Indexed forms (`front,N`)
/// select a numbered slot on the side — variable node heights, out of
/// scope (gently-byq). Conflicting sides on the two ends fall back to the
/// unported route (no recorded shape pins one).
pub(super) fn port_dip(graph: &Graph, ei: usize) -> Option<Route> {
    let side = |v: Option<&str>| match v?.split(',').next()? {
        "right" | "south" | "down" | "180" => Some(Route::PortBelow),
        "left" | "north" | "up" | "0" => Some(Route::PortAbove),
        // front/forward/east and back/west keep the default attachment;
        // anything else (unknown spelling) is ignored.
        _ => None,
    };
    let start = side(graph.get_attr(Scope::Edge(ei), "start"));
    let end = side(graph.get_attr(Scope::Edge(ei), "end"));
    match (start, end) {
        (Some(s), None) | (None, Some(s)) => Some(s),
        (Some(s), Some(e)) if s == e => Some(s),
        _ => None,
    }
}

/// Grid-cell span of node `idx` in columns and rows (gently-awd):
/// the `columns`/`rows` attributes — derived from `size: W,H` in the
/// attribute store — parsed as grid cells. Upstream `_calc_size` reads
/// `abs($attr || 1)`, so an absent, empty, or `0` value is one cell.
/// An INDEXED port attached to the node grows the span (gently-byq,
/// upstream Node.pm ~1170 + Path.pm): an indexed `front,N`-style slot on
/// one of the node's horizontal sides (start port on an edge leaving it,
/// end port on an edge entering it) needs row `N` of the node, so the
/// row span grows to `max(cy, N+1)` — the max-side rule `min_y =
/// max(need.west, need.east)`. Vertical sides (`right,left,south,north`)
/// would grow the column span — no recorded fixture pins them, so they
/// contribute nothing here.
pub(super) fn node_span(graph: &Graph, idx: usize) -> (usize, usize) {
    let side = |key: &str| {
        graph
            .get_attr(Scope::Node(idx), key)
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(1)
    };
    let (cx, cy) = (side("columns"), side("rows"));
    grow_free_ports(graph, idx, cx, cy)
}

/// The indexed port slot one edge carries: `start: <side>,N` (on the
/// edge's start, `end: false`) or `end: <side>,N` (on its end, `end:
/// true`) — the side must be a horizontal spelling (flow front/back,
/// east/west; upstream Attributes.pm `_flow_as_side`) and `N` a number.
/// Unindexed spellings and vertical sides give `None`.
pub(crate) fn indexed_slot(graph: &Graph, ei: usize, end: bool) -> Option<usize> {
    let raw = graph.get_attr(Scope::Edge(ei), if end { "end" } else { "start" })?;
    let (side, n) = raw.split_once(',')?;
    let horizontal = matches!(side, "front" | "forward" | "east" | "back" | "west");
    let n = n.trim().parse::<usize>().ok()?;
    horizontal.then_some(n)
}

/// Per-side port accounting for one node (gently-a6g): side-constrained
/// ends without a number (`cnt`), indexed ends (`portnr` + the deepest
/// `max` slot), and the totals the growth loop needs. Sides: 0=north,
/// 1=south, 2=east/west, 3=north/south spellings.
#[derive(Default)]
struct PortStats {
    cnt: [usize; 4],
    portnr: [usize; 4],
    max: [usize; 4],
    unspecified: usize,
    outgoing: usize,
    slot_floor: usize,
}

/// Scan the edges attached to node `idx`: one branch pair per edge,
/// port classification only at the ends that touch `idx` (a String per
/// visit would dominate the layout, ge.layout.c6).
fn port_scan(graph: &Graph, idx: usize) -> PortStats {
    let mut stats = PortStats::default();
    for (ei, e) in graph.edges.iter().enumerate() {
        if e.from == idx {
            stats.outgoing += 1;
            classify_port(graph, ei, false, &mut stats);
        }
        if e.to == idx {
            classify_port(graph, ei, true, &mut stats);
        }
    }
    stats
}

/// Classify one edge end attached to the growing node (start port when
/// `end == false`): indexed horizontal slots count as east/west ports
/// (plus the row floor), unindexed side spellings count on their side,
/// anything else is an unspecified end.
fn classify_port(graph: &Graph, ei: usize, end: bool, stats: &mut PortStats) {
    if let Some(n) = indexed_slot(graph, ei, end) {
        stats.portnr[2] += 1;
        stats.max[2] = stats.max[2].max(n + 1);
        stats.slot_floor = stats.slot_floor.max(n + 1);
        return;
    }
    let head = graph
        .get_attr(Scope::Edge(ei), if end { "end" } else { "start" })
        .map(|v| v.split(',').next().unwrap_or(""));
    match head {
        Some("front" | "forward" | "east" | "back" | "west") => stats.cnt[2] += 1,
        Some("right" | "south" | "down" | "left" | "north" | "up") => stats.cnt[3] += 1,
        _ => stats.unspecified += 1,
    }
}

/// The free-ports growth entry (gently-a6g): a node carrying FOUR OR
/// MORE unspecified edge-ends grows until every end has a free port.
/// Fewer than four unspecified ends keep the base span (upstream's
/// shortcut), except the indexed-port floor (gently-byq: an indexed
/// slot needs row N+1).
fn grow_free_ports(graph: &Graph, idx: usize, cx: usize, cy: usize) -> (usize, usize) {
    let stats = port_scan(graph, idx);
    if stats.unspecified < 4 {
        return (cx, cy.max(stats.slot_floor));
    }
    Grow {
        cx: cx.max(stats.max[0]).max(stats.max[1]),
        cy: cy.max(stats.max[2]).max(stats.max[3]),
        row_next: true,
    }
    .until_free(&stats)
}

/// Minimum spans demanded by the constrained ports (upstream min_x from
/// north/south, min_y from east/west); then the growth loop.
struct Grow {
    cx: usize,
    cy: usize,
    row_next: bool,
}

impl Grow {
    /// Free ports per side: `1 + (size - cnt - portnr) / 2`, summed over
    /// north/south (the column span) and east/west (the row span) — a
    /// pure sink skips its front side (east for the east flow), so the
    /// fan-in hub grows on its row span (four sources: 1+1+1 = 3 < 4,
    /// then 1+1+2 = 4 → a 3-row span, the fan-layout.observed hub).
    fn free_ports(&self, stats: &PortStats) -> usize {
        let mut free = 0;
        for (si, size) in [(0, self.cx), (1, self.cx), (2, self.cy), (3, self.cy)] {
            if stats.outgoing == 0 && si == 2 {
                continue;
            }
            let taken = stats.cnt[si] + stats.portnr[si];
            free += 1 + size.saturating_sub(taken) / 2;
        }
        free
    }

    /// One grow step bumps the row span (then the column span) by two —
    /// upstream's grow_what order for the east flow.
    fn step(&self) -> Grow {
        Grow {
            cx: if self.row_next { self.cx } else { self.cx + 2 },
            cy: if self.row_next { self.cy + 2 } else { self.cy },
            row_next: !self.row_next,
        }
    }

    /// Grow until every unspecified end has a free port.
    fn until_free(&self, stats: &PortStats) -> (usize, usize) {
        let mut span = Grow { ..*self };
        while span.free_ports(stats) < stats.unspecified {
            span = span.step();
        }
        (span.cx, span.cy)
    }
}
