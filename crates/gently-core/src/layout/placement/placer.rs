//! Purpose: the placement state machine for ge.layout — the `Placer`
//! owns node slots, band allocation, per-edge routes, and the port
//! overrides applied after the chain pass. Rationale: extracted from the
//! placement orchestrator so each file stays under the gate limits; all
//! decisions mirror the upstream placement behavior exactly.

use super::*;

/// The placement pass state: node slots, taken slots, the next free band,
/// per-edge travel kinds, and per-pair edge counts (keyed lookups only —
/// never iterated, so the pass stays a pure function, c1).
pub(super) struct Placer {
    pub(super) cell: Vec<Option<(usize, usize)>>,
    taken: Vec<(usize, usize)>,
    pub(super) next_band: usize,
    pub(super) route: Vec<Route>,
    pairs: HashMap<(usize, usize), usize>,
    /// Longest-path rank of each node (gently-jrr): constrains the east
    /// extension column so a shared-middle bypass stays inline.
    pub(super) ranks: Vec<usize>,
    /// Grid-cell row span of each node (gently-awd): gates the stacked
    /// parallel channels (a multi-row pair stacks channels on its row
    /// span; a one-row pair keeps the bend shape).
    pub(super) spans: Vec<(usize, usize)>,
}

impl Placer {
    pub(super) fn new(edge_count: usize, node_count: usize) -> Placer {
        Placer {
            cell: vec![None; node_count],
            taken: Vec::new(),
            next_band: 0,
            route: vec![Route::Grew; edge_count],
            pairs: HashMap::new(),
            ranks: vec![0; node_count],
            spans: vec![(1, 1); node_count],
        }
    }

    /// A self-loop marks its node; an unplaced self-loop node takes the
    /// next free column on band 0.
    pub(super) fn place_selfloop(&mut self, ei: usize, node: usize) {
        self.route[ei] = Route::Selfloop;
        if self.cell[node].is_none() {
            let col = self
                .taken
                .iter()
                .filter(|&&(_, band)| band == 0)
                .map(|&(col, _)| col)
                .max()
                .map_or(0, |max| max + 1);
            let spot = (col, 0);
            self.cell[node] = Some(spot);
            self.taken.push(spot);
        }
        self.next_band = self.next_band.max(1);
    }

    /// Place one edge's unplaced endpoint(s) and classify its travel.
    pub(super) fn place_edge(&mut self, ei: usize, u: usize, v: usize) {
        let key = (u.min(v), u.max(v));
        let prior = *self.pairs.get(&key).unwrap_or(&0);
        self.pairs.insert(key, prior + 1);
        match (self.cell[u], self.cell[v]) {
            (Some(pu), Some(pv)) => self.place_both_placed(ei, u, v, prior, pu, pv),
            (Some(pu), None) => {
                self.route[ei] = Route::Grew;
                self.extend_or_below(v, pu);
            }
            (None, Some(pv)) => {
                self.route[ei] = Route::Grew;
                self.attach_next_to(u, v, pv);
            }
            (None, None) => {
                self.route[ei] = Route::Grew;
                // A fresh chain starts its own band (oracle: a second
                // disconnected chain renders on its own row).
                let band = self.fresh_band();
                self.claim(u, (0, band));
                self.claim(v, (1, band));
            }
        }
    }

    /// Both endpoints placed: a parallel bend when the pair already has an
    /// edge on one band, otherwise a route around. A multi-row pair
    /// stacks its channels on the boxes' row span instead (gently-awd):
    /// channel k rides the grid row `band + k` while the span has rows
    /// for it (k <= cy - 1), so the channels run straight through the
    /// tall boxes and grow them to their full span (the so-flowchart-size
    /// oracle shape); a one-row pair keeps the recorded bend shape
    /// (parallel.txt: channel 1 bends above the band).
    pub(super) fn place_both_placed(
        &mut self,
        ei: usize,
        u: usize,
        v: usize,
        prior: usize,
        pu: (usize, usize),
        pv: (usize, usize),
    ) {
        self.route[ei] = if prior >= 1 && pu.1 == pv.1 {
            let cy = self.spans[u].1.max(self.spans[v].1);
            if cy > 1 && prior < cy {
                Route::Stacked(prior)
            } else {
                Route::Parallel(prior)
            }
        } else {
            Route::Around
        };
    }

    /// Extend east on the source's band when the slot is free, else attach
    /// below the source on its own band (oracle: the diamond's c under a).
    ///
    /// The east column honors the longest-path rank of `node` (gently-jrr:
    /// the shared-middle bypass — Koblenz -> Trier -> Frankfurt forces
    /// Frankfurt east of Trier even though the Koblenz -> Frankfurt edge
    /// is processed first): a node whose rank demands a column beyond the
    /// source's east slot takes that column, leaving room for the middle
    /// node the later edge will place inline. Tests/repro/claims/
    /// bg-minlen-bypass.observed pins the inline rank
    /// (Bonn, Koblenz, Trier, Frankfurt, Dresden) with the skip-edge
    /// bending over the band.
    fn extend_or_below(&mut self, node: usize, pu: (usize, usize)) {
        // Rank-aware column (gently-jrr): the shared-middle bypass — the
        // node's longest-path rank sits exactly one column east of the
        // source's immediate east slot, whose occupant must itself yield
        // east (rank = source col + 2). A deeper rank demand has no
        // recorded shape pinning it, so the placement stays insertion-order.
        let col = if self.ranks[node] == pu.0 + 2 {
            pu.0 + 2
        } else {
            pu.0 + 1
        };
        let east = (col, pu.1);
        if self.taken.contains(&east) {
            let band = self.fresh_band();
            self.claim(node, (pu.0, band));
        } else {
            self.claim(node, east);
        }
    }

    /// Attach next to the placed target (gently-a6g, upstream
    /// _place_node + _near_places): walk the target's slot ring in the
    /// oracle order — east side top-to-bottom, below, west side top-to-
    /// bottom, above — at ring distance one (the single gap column/row
    /// band), then distance two; the first slot clear of every placed
    /// node's span (Chebyshev distance one: no corner-to-corner touch,
    /// upstream _clear_tries) wins. The fan-in star: the first attacher
    /// takes the east ring slot at the hub's top row, the second slots
    /// two rows south (the adjacent row is cleared against the first),
    /// the third hangs below the hub's grown span; a target whose west
    /// slot is free takes an east-side attacher there (shared-target's
    /// b east of c — the east side is tried first, the edge drawn
    /// westward). When no ring slot clears, fall back to a fresh band
    /// below (the recorded leaf-under-train attachment).
    fn attach_next_to(&mut self, node: usize, target: usize, pv: (usize, usize)) {
        let cy = self.spans[target].1;
        for d in [1usize, 2] {
            let mut ring = Vec::new();
            // east side: one gap column east per ring step (a rank is
            // two grid columns), every row of the span
            for db in 0..cy {
                ring.push((pv.0 + d, pv.1 + db));
            }
            // below: two band rows south of the span's last row per
            // ring step (upstream d=2 cells from the border)
            ring.push((pv.0, pv.1 + cy - 1 + 2 * d));
            // west side
            if pv.0 >= d {
                for db in 0..cy {
                    ring.push((pv.0 - d, pv.1 + db));
                }
            }
            // above (negative bands have no grid row)
            if pv.1 >= 2 * d {
                ring.push((pv.0, pv.1 - 2 * d));
            }
            if let Some(&spot) = ring.iter().find(|&&s| self.clear_of_spans(s)) {
                self.claim(node, spot);
                return;
            }
        }
        let band = self.fresh_band();
        self.claim(node, (pv.0, band));
    }

    /// Is `spot` free of every placed node's span at Chebyshev distance
    /// one in GRID cells (upstream _clear_tries: no shared cell, no side
    /// adjacency, no corner touch)? Columns live on the even grid (one
    /// rank is two grid columns), bands are grid rows; span arithmetic
    /// stays keyed — a pure predicate (c1).
    fn clear_of_spans(&self, spot: (usize, usize)) -> bool {
        let sx = 2 * spot.0;
        self.cell
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.map(|c| (i, c)))
            .all(|(i, (c, b))| {
                let (cx, cy) = self.spans[i];
                let (c0, c1) = (2 * c, 2 * c + cx.saturating_sub(1));
                let dx = if sx < c0 {
                    c0 - sx
                } else {
                    sx.saturating_sub(c1)
                };
                let dy = if spot.1 < b {
                    b - spot.1
                } else {
                    spot.1.saturating_sub(b + cy - 1)
                };
                dx > 1 || dy > 1
            })
    }

    /// Port-constrained same-band edges (gently-89r): a `start:`/`end:`
    /// port naming a south/north side forces the edge to dip below / bend
    /// above the band, whatever its unported travel — placement is never
    /// changed (tests/repro/claims/port-side-minimal.observed: the
    /// no-port chain lays Trier inline exactly like the ported one).
    /// Parallel channels, self-loops, and cross-band routes keep their
    /// recorded shapes (no recorded shape ports them).
    pub(super) fn apply_ports(&mut self, graph: &Graph) {
        for (ei, e) in graph.edges.iter().enumerate() {
            if e.from == e.to || !self.horizontal(e.from, e.to) {
                continue;
            }
            self.apply_port(graph, ei);
        }
    }

    /// Both endpoints placed on the same band.
    fn horizontal(&self, from: usize, to: usize) -> bool {
        match (self.cell[from], self.cell[to]) {
            (Some(pu), Some(pv)) => pu.1 == pv.1,
            _ => false,
        }
    }

    /// Port overrides for one horizontal edge: an INDEXED horizontal
    /// port slot (gently-byq) overrides the parallel/stacked
    /// classification too — the slot names the attachment row, whatever
    /// the unported travel would be. A slot on BOTH ends is unpinned —
    /// the start slot wins. Slot 0 is the default attachment (no route
    /// change).
    fn apply_port(&mut self, graph: &Graph, ei: usize) {
        let slot = super::super::ports::indexed_slot(graph, ei, false)
            .filter(|&n| n > 0)
            .map(|n| Route::Slot { end: false, n })
            .or_else(|| {
                super::super::ports::indexed_slot(graph, ei, true)
                    .filter(|&n| n > 0)
                    .map(|n| Route::Slot { end: true, n })
            });
        if let Some(slot) = slot {
            if !matches!(self.route[ei], Route::Selfloop) {
                self.route[ei] = slot;
            }
        } else if let Some(port) = crate::layout::ports::port_dip(graph, ei) {
            if matches!(self.route[ei], Route::Grew | Route::Around) {
                self.route[ei] = port;
            }
        }
    }

    /// Edgeless graphs line up on band 0 in insertion order — the
    /// tracer-era behavior pinned by the tb.ascii_render tests.
    pub(super) fn place_edgeless(&mut self) {
        for (i, slot) in self.cell.iter_mut().enumerate() {
            *slot = Some((i, 0));
        }
        self.next_band = 1;
    }

    /// Every chain edge in upstream order: self-loops and pairs placed
    /// as encountered.
    pub(super) fn place_chained(&mut self, graph: &Graph) {
        for ei in chain_order(graph) {
            let e = &graph.edges[ei];
            if e.from == e.to {
                self.place_selfloop(ei, e.from);
            } else {
                self.place_edge(ei, e.from, e.to);
            }
        }
    }

    /// Isolated nodes (no incident edges) each take the next free band,
    /// column 0, in insertion order (oracle: d / z below the chain).
    pub(super) fn place_isolated(&mut self) {
        for node in 0..self.cell.len() {
            if self.cell[node].is_none() {
                let band = self.fresh_band();
                self.claim(node, (0, band));
            }
        }
    }

    fn claim(&mut self, node: usize, spot: (usize, usize)) {
        self.cell[node] = Some(spot);
        self.taken.push(spot);
        // Ring slots can land beyond the sequential band cursor (the
        // fan-in star's east ring spans the grown hub's rows).
        self.next_band = self.next_band.max(spot.1 + 1);
    }

    fn fresh_band(&mut self) -> usize {
        let band = self.next_band;
        self.next_band += 1;
        band
    }
}
