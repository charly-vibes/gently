//! Purpose: the placement phase of the deterministic grid layout
//! (gently-0of) — one insertion-order pass over the edges that assigns
//! every node a `(column-rank, band)` slot and classifies every edge's
//! travel.
//! Responsibilities: extend east on the source's band else attach below;
//! attach an unplaced endpoint to a placed one west-then-east; start fresh
//! chains on their own band; give isolated nodes their own band at column
//! 0; line edgeless graphs up on band 0; count per-pair edges for parallel
//! bends. Rationale: extracted from the monolithic layout() so each phase
//! stays under the tidy gates; placement rules shaped by the recorded
//! v0.69 @ ededa3d7 companions.

use std::collections::HashMap;

use super::{Placement, Route};
use crate::graph::Graph;

/// Arrange the placement for the configured flow: the oracle-compatible
/// insertion-order pipeline for the default east flow, the strictly-
/// ranked layering for every other flow (whose outputs no tier-1 fixture
/// pins). `None` for the empty graph.
pub(super) fn arrange(graph: &Graph, east: bool) -> Option<Placement> {
    if east {
        place(graph)
    } else {
        Some(layered(graph))
    }
}

/// The strictly-ranked layering (ge.layout.c3): rank = longest-path
/// layer over the non-selfloop edges (cycle edges capped by the round
/// bound — they route against the flow, as the probed oracle does),
/// bands = first-seen order within each rank. Node indices order the
/// bands, so the pass stays a pure function (ge.layout.c1).
pub(super) fn layered(graph: &Graph) -> Placement {
    let n = graph.nodes.len();
    let ranks = rank_pass(graph);
    let max_rank = ranks.iter().max().copied().unwrap_or(0);
    let mut cell = vec![None; n];
    let mut next_band = vec![0usize; max_rank + 1];
    let mut bands = 0usize;
    for (node, &rank) in ranks.iter().enumerate() {
        let band = next_band[rank];
        next_band[rank] = band + 1;
        bands = bands.max(band + 1);
        cell[node] = Some((rank, band));
    }
    let route = classify(graph, &cell);
    Placement {
        cell,
        route,
        bands,
    }
}

/// Longest-path ranks over the non-selfloop edges: relax in model edge
/// order for at most `n` rounds — a DAG converges strictly earlier, and
/// cycle edges hit the round cap and simply stop inflating (deterministic
/// pure function, ge.layout.c1).
fn rank_pass(graph: &Graph) -> Vec<usize> {
    let n = graph.nodes.len();
    let mut rank = vec![0usize; n];
    for _ in 0..n {
        let mut changed = false;
        for e in &graph.edges {
            if e.from != e.to && rank[e.from] + 1 > rank[e.to] {
                rank[e.to] = rank[e.from] + 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    rank
}

/// Classify every edge's travel over pre-assigned cells: self-loop,
/// parallel bend (same band, prior edge on the pair), or grew (straight
/// when the bands align, bend/multi-band otherwise — the routing pass
/// decides by geometry).
fn classify(graph: &Graph, cell: &[Option<(usize, usize)>]) -> Vec<Route> {
    let mut pairs: HashMap<(usize, usize), usize> = HashMap::new();
    graph
        .edges
        .iter()
        .map(|e| {
            let key = (e.from.min(e.to), e.from.max(e.to));
            let prior = pairs.entry(key).or_insert(0);
            let count = *prior;
            *prior += 1;
            let pu = cell[e.from].unwrap();
            let pv = cell[e.to].unwrap();
            if e.from == e.to {
                Route::Selfloop
            } else if count >= 1 && pu.1 == pv.1 {
                Route::Parallel(count)
            } else {
                Route::Grew
            }
        })
        .collect()
}

/// Place every node: one insertion-order pass over the edges, then the
/// isolated nodes. `None` for the empty graph.
pub(super) fn place(graph: &Graph) -> Option<Placement> {
    let n = graph.nodes.len();
    if n == 0 {
        return None;
    }
    let mut placer = Placer::new(graph.edges.len(), n);
    if graph.edges.is_empty() {
        // Edgeless graphs line up on band 0 in insertion order — the
        // tracer-era behavior pinned by the tb.ascii_render tests.
        for (i, slot) in placer.cell.iter_mut().enumerate() {
            *slot = Some((i, 0));
        }
        placer.next_band = 1;
    } else {
        for (ei, e) in graph.edges.iter().enumerate() {
            if e.from == e.to {
                placer.place_selfloop(ei, e.from);
            } else {
                placer.place_edge(ei, e.from, e.to);
            }
        }
        placer.place_isolated();
    }
    Some(Placement {
        cell: placer.cell,
        route: placer.route,
        bands: placer.next_band,
    })
}

/// The placement pass state: node slots, taken slots, the next free band,
/// per-edge travel kinds, and per-pair edge counts (keyed lookups only —
/// never iterated, so the pass stays a pure function, c1).
struct Placer {
    cell: Vec<Option<(usize, usize)>>,
    taken: Vec<(usize, usize)>,
    next_band: usize,
    route: Vec<Route>,
    pairs: HashMap<(usize, usize), usize>,
}

impl Placer {
    fn new(edge_count: usize, node_count: usize) -> Placer {
        Placer {
            cell: vec![None; node_count],
            taken: Vec::new(),
            next_band: 0,
            route: vec![Route::Grew; edge_count],
            pairs: HashMap::new(),
        }
    }

    /// A self-loop marks its node; an unplaced self-loop node takes the
    /// next free column on band 0.
    fn place_selfloop(&mut self, ei: usize, node: usize) {
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
    fn place_edge(&mut self, ei: usize, u: usize, v: usize) {
        let key = (u.min(v), u.max(v));
        let prior = *self.pairs.get(&key).unwrap_or(&0);
        self.pairs.insert(key, prior + 1);
        match (self.cell[u], self.cell[v]) {
            (Some(pu), Some(pv)) => self.place_both_placed(ei, prior, pu, pv),
            (Some(pu), None) => {
                self.route[ei] = Route::Grew;
                self.extend_or_below(v, pu);
            }
            (None, Some(pv)) => {
                self.route[ei] = Route::Grew;
                self.attach_next_to(u, pv);
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
    /// edge on one band, otherwise a route around.
    fn place_both_placed(&mut self, ei: usize, prior: usize, pu: (usize, usize), pv: (usize, usize)) {
        self.route[ei] = if prior >= 1 && pu.1 == pv.1 {
            Route::Parallel(prior)
        } else {
            Route::Around
        };
    }

    /// Extend east on the source's band when the slot is free, else attach
    /// below the source on its own band (oracle: the diamond's c under a).
    fn extend_or_below(&mut self, node: usize, pu: (usize, usize)) {
        let east = (pu.0 + 1, pu.1);
        if self.taken.contains(&east) {
            let band = self.fresh_band();
            self.claim(node, (pu.0, band));
        } else {
            self.claim(node, east);
        }
    }

    /// Attach next to the placed target: west slot first, then east
    /// (oracle: shared-target's b east of c, the edge drawn westward),
    /// else below on its own band.
    fn attach_next_to(&mut self, node: usize, pv: (usize, usize)) {
        let west_free = pv.0 >= 1 && !self.taken.contains(&(pv.0 - 1, pv.1));
        let spot = if west_free {
            (pv.0 - 1, pv.1)
        } else if !self.taken.contains(&(pv.0 + 1, pv.1)) {
            (pv.0 + 1, pv.1)
        } else {
            let band = self.fresh_band();
            (pv.0, band)
        };
        self.claim(node, spot);
    }

    /// Isolated nodes (no incident edges) each take the next free band,
    /// column 0, in insertion order (oracle: d / z below the chain).
    fn place_isolated(&mut self) {
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
    }

    fn fresh_band(&mut self) -> usize {
        let band = self.next_band;
        self.next_band += 1;
        band
    }
}
