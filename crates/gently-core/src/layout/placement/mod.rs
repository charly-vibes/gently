//! Purpose: the placement phase of the deterministic grid layout
//! (gently-0of) — one chain-order pass over the edges that assigns
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

mod placer;

use super::chains::chain_order;
use super::{node_span, Placement, Route};
use crate::graph::Graph;
use placer::Placer;

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
    Placement { cell, route, bands }
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
    let spans: Vec<(usize, usize)> = (0..graph.nodes.len())
        .map(|i| node_span(graph, i))
        .collect();
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
                let cy = spans[e.from].1.max(spans[e.to].1);
                if cy > 1 && count < cy {
                    Route::Stacked(count)
                } else {
                    Route::Parallel(count)
                }
            } else {
                Route::Grew
            }
        })
        .collect()
}

/// Place every node: one chain-order pass over the edges, then the
/// isolated nodes. `None` for the empty graph.
pub(super) fn place(graph: &Graph) -> Option<Placement> {
    let n = graph.nodes.len();
    if n == 0 {
        return None;
    }
    let mut placer = Placer::new(graph.edges.len(), n);
    placer.ranks = rank_pass(graph);
    placer.spans = (0..n).map(|i| node_span(graph, i)).collect();
    if graph.edges.is_empty() {
        placer.place_edgeless();
    } else {
        placer.place_chained(graph);
        placer.apply_ports(graph);
        placer.place_isolated();
    }
    Some(Placement {
        cell: placer.cell,
        route: placer.route,
        bands: placer.next_band,
    })
}
