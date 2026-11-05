//! Purpose: the `_assign_ranks` rank machinery for ge.txt_render —
//! upstream Graph::Easy's As_txt node ordering (chain heads at -1
//! deepening per hop, mid-graph nodes promoted from the `also` queue).
//! Rationale: every ordering decision mirrors As_txt.pm exactly so the
//! pinned fixture corpus renders byte-identically.

use crate::graph::Graph;

/// Upstream `_assign_ranks` (v1: no user ranks, no root node, no groups):
/// nodes without predecessors get rank -1 and their chains deepen by one
/// per hop (-2, -3, …); mid-graph nodes are picked up from the `also`
/// queue at rank -1. The todo list stays sorted ascending by rank with
/// FIFO tie order (upstream Graph::Easy::Heap).
pub(super) fn assign_ranks(graph: &Graph) -> Vec<isize> {
    let mut st = RankState::seed(graph);
    while !st.also.is_empty() || !st.todo.is_empty() {
        propagate_ranks(graph, &mut st);
        promote_from_also(&mut st);
    }
    // every node is ranked when both queues drain: propagation covers
    // reachable nodes, promotion seeds the rest
    st.rank.iter().map(|r| r.unwrap_or(-1)).collect()
}

/// The `_assign_ranks` work state: per-node ranks, the ascending-rank
/// todo list (upstream Graph::Easy::Heap), and the mid-graph `also`
/// promotion queue.
struct RankState {
    rank: Vec<Option<isize>>,
    todo: Vec<(isize, usize)>,
    also: std::collections::VecDeque<usize>,
}

impl RankState {
    /// Chain heads (no predecessors) seed the todo at rank -1; every
    /// other node waits in `also` for promotion.
    fn seed(graph: &Graph) -> Self {
        let n = graph.nodes.len();
        let todo: Vec<(isize, usize)> = (0..n)
            .filter(|&i| !has_predecessors(graph, i))
            .map(|i| (-1, i))
            .collect();
        let also: std::collections::VecDeque<usize> =
            (0..n).filter(|&i| has_predecessors(graph, i)).collect();
        let mut rank = vec![None; n];
        for &(r, i) in &todo {
            rank[i] = Some(r);
        }
        RankState { rank, todo, also }
    }
}

/// Drain the todo list, deepening every unranked successor by one hop.
fn propagate_ranks(graph: &Graph, st: &mut RankState) {
    while let Some(&(r, node)) = st.todo.first() {
        st.todo.remove(0);
        let l = if r > 0 { -r } else { r } - 1;
        for succ in successors(graph, node) {
            if st.rank[succ].is_none() {
                st.rank[succ] = Some(l);
                insert_stable(&mut st.todo, (l, succ));
            }
        }
    }
}

/// Promote one still-unranked mid-graph node to a chain head at rank -1.
fn promote_from_also(st: &mut RankState) {
    while let Some(x) = st.also.pop_front() {
        if st.rank[x].is_none() {
            st.rank[x] = Some(-1);
            insert_stable(&mut st.todo, (-1, x));
            break;
        }
    }
}

/// Insert into the ascending-by-rank todo list, after all entries with an
/// equal rank (FIFO tie order — upstream's sorted-linear heap).
fn insert_stable(todo: &mut Vec<(isize, usize)>, entry: (isize, usize)) {
    let pos = todo.partition_point(|&(r, _)| r <= entry.0);
    todo.insert(pos, entry);
}

/// Distinct successors of `node`, in first-seen edge order (upstream's
/// `%suc` weed-out of doubles).
pub(super) fn successors(graph: &Graph, node: usize) -> Vec<usize> {
    graph.edges.iter().fold(Vec::new(), |mut seen, edge| {
        if edge.from == node && !seen.contains(&edge.to) {
            seen.push(edge.to);
        }
        seen
    })
}

pub(super) fn has_predecessors(graph: &Graph, node: usize) -> bool {
    graph.edges.iter().any(|e| e.to == node)
}

/// Upstream `sorted_successors`: successors with more successors first,
/// ties by name (counts from the precomputed successor matrix).
pub(super) fn sorted_successors(graph: &Graph, succ: &[usize], succs: &[Vec<usize>]) -> Vec<usize> {
    sorted(succ.to_vec(), |x, y| {
        succs[*y]
            .len()
            .cmp(&succs[*x].len())
            .then(graph.nodes[*x].name.cmp(&graph.nodes[*y].name))
    })
}

/// The edge/isolated-node pass order (upstream `sorted_nodes('rank','name')`):
/// ascending by absolute rank, ties by name.
pub(super) fn ordered_nodes(graph: &Graph, ranks: &[isize]) -> Vec<usize> {
    sorted((0..graph.nodes.len()).collect(), |x, y| {
        ranks[*x]
            .abs()
            .cmp(&ranks[*y].abs())
            .then(graph.nodes[*x].name.cmp(&graph.nodes[*y].name))
    })
}

/// Sort an owned vec by a node comparator.
fn sorted(mut v: Vec<usize>, cmp: impl Fn(&usize, &usize) -> std::cmp::Ordering) -> Vec<usize> {
    v.sort_by(|x, y| cmp(x, y));
    v
}
