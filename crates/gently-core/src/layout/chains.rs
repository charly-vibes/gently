//! Purpose: the chain-based edge processing order for the east
//! placement pass (gently-pua) — upstream Graph::Easy 0.69's Layout.pm
//! chain tracking, re-derived from the pinned source and the realized
//! action stack (probed with debug: chain sort, _todo placement race,
//! inter-chain splice).
//! Responsibilities: build node chains from the roots along the flow
//! (absorbing already-chained suffixes), lay them out root-anchored
//! first/longest first/start-name, and emit the edge processing order
//! with placement ownership racing to the first chain — a chain whose
//! members were all absorbed re-ranks as a leftover leaf attaching to
//! its placed successor. Rationale: this is the ranking/placement rule
//! behind the shape:edge box-chain divergence (gently-pua,
//! tests/repro/claims/bg-edge-node.observed); kept as its own module so
//! each phase stays under the tidy gates.

use crate::graph::Graph;

/// The chain-based edge processing order (gently-pua): upstream
/// Graph::Easy 0.69's Layout.pm tracks node chains from the roots (nodes
/// never appearing as an edge target, walked along the flow — a `<>`
/// edge counts from both ends, a plain edge only from its source), then
/// lays the chains out root-anchored first, longest first, start name as
/// tie-break. Each chain's layout claims its unclaimed members in order
/// (the walk's opener edge, then the consecutive-pair edges), emits the
/// still-unrouted links between consecutive members, and splices in the
/// chains its members' leftover links reach — so a node is placed by the
/// first chain to claim it, and a chain whose members were all absorbed
/// re-ranks as a leftover leaf attaching to its placed successor (the
/// buy-anon under train). Never-claimed leftovers keep model order.
/// Keyed lookups and ordered vectors only — a pure function (c1).
pub(super) fn chain_order(graph: &Graph) -> Vec<usize> {
    let chains = track_chains(graph);
    let ranked = rank_chains(graph, &chains);
    let n = graph.nodes.len();
    let mut emit = Emitter {
        graph,
        chains: &chains,
        pair_edges: pair_edges(graph),
        order: Vec::new(),
        emitted: vec![false; graph.edges.len()],
        claimed: vec![false; n],
        done: vec![false; chains.chains.len()],
    };
    for &k in &ranked {
        emit.chain(k);
    }
    // never-emitted leftovers in model order
    for ei in 0..graph.edges.len() {
        if !emit.emitted[ei] {
            emit.emitted[ei] = true;
            emit.order.push(ei);
        }
    }
    emit.order
}

/// Track all chains from the name-ordered roots (upstream _find_chains):
/// incidence lists, root set, and the recursive chain walk.
fn track_chains(graph: &Graph) -> Chains {
    let n = graph.nodes.len();
    let mut at: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut targets = vec![false; n];
    for (ei, e) in graph.edges.iter().enumerate() {
        if e.from != e.to {
            at[e.from].push(ei);
            at[e.to].push(ei);
            targets[e.to] = true;
        }
    }
    let mut roots: Vec<usize> = (0..n).filter(|&v| !targets[v]).collect();
    roots.sort_by(|&a, &b| graph.nodes[a].name.cmp(&graph.nodes[b].name));

    let mut chains = Chains {
        chains: Vec::new(),
        chain_of: vec![None; n],
        at,
        root_anchor: roots.first().copied(),
    };
    for &r in &roots {
        if chains.chain_of[r].is_none() {
            chains.follow(graph, r);
        }
    }
    chains
}

/// The layout order (upstream's sort over the chains registry): the
/// root-anchored chain first, then longest first, then start name.
fn rank_chains(graph: &Graph, chains: &Chains) -> Vec<usize> {
    let start = |k: usize| chains.chains[k][0];
    let mut ranked: Vec<usize> = (0..chains.chains.len()).collect();
    ranked.sort_by(|&a, &b| {
        let (ra, rb) = (Some(start(a)), Some(start(b)));
        let (ra, rb) = (ra == chains.root_anchor, rb == chains.root_anchor);
        rb.cmp(&ra)
            .then(chains.chains[b].len().cmp(&chains.chains[a].len()))
            .then(graph.nodes[start(a)].name.cmp(&graph.nodes[start(b)].name))
    });
    ranked
}

/// Chain bookkeeping: the node lists (walked along the flow), which
/// chain owns each node, incidence lists, and the name-first root.
struct Chains {
    chains: Vec<Vec<usize>>,
    chain_of: Vec<Option<usize>>,
    /// Incident non-selfloop edge indices per node, model order.
    at: Vec<Vec<usize>>,
    /// The name-first root (upstream's `_root`): the chain starting here
    /// sorts before every other.
    root_anchor: Option<usize>,
}

impl Chains {
    /// Track the chain starting at `r` (upstream _follow_chain): extend
    /// through the unique unchained successor; run into an already-
    /// chained one and its chain's suffix is absorbed (merge); several
    /// successors recurse first and the longest successor chain is
    /// absorbed (upstream's tie-break keeps the first in model order).
    fn follow(&mut self, graph: &Graph, r: usize) {
        let cid = self.chains.len();
        self.chains.push(vec![r]);
        self.chain_of[r] = Some(cid);
        let mut cur = r;
        loop {
            let sucs = self.successors(graph, cur, cid);
            if sucs.is_empty() {
                break;
            }
            if sucs.len() == 1 {
                let s = sucs[0];
                if self.chain_of[s].is_none() {
                    self.chains[cid].push(s);
                    self.chain_of[s] = Some(cid);
                    cur = s;
                } else {
                    self.absorb(cid, self.chain_of[s].unwrap(), s);
                    break;
                }
            } else {
                self.merge_longest(graph, cid, &sucs);
                break;
            }
        }
    }

    /// Several successors (upstream's multi-branch): recurse into the
    /// unchained ones first, then absorb the longest successor chain's
    /// list from its first node — the first in model order on a tie.
    fn merge_longest(&mut self, graph: &Graph, cid: usize, sucs: &[usize]) {
        let mut best: Option<(usize, usize)> = None;
        for &s in sucs {
            if self.chain_of[s].is_none() {
                self.follow(graph, s);
            }
            let len = self.chains[self.chain_of[s].unwrap()].len();
            if best.is_none_or(|(bl, _)| len > bl) {
                best = Some((len, s));
            }
        }
        if let Some((_, s)) = best {
            self.absorb(cid, self.chain_of[s].unwrap(), s);
        }
    }

    /// Unique flow-successors of `cur` outside `cid`: the target of each
    /// incident edge read along the flow (`<>` reads both ways), self-
    /// loops and multi-edge duplicates dropped, model order kept.
    fn successors(&self, graph: &Graph, cur: usize, cid: usize) -> Vec<usize> {
        let mut sucs: Vec<usize> = Vec::new();
        for &ei in &self.at[cur] {
            let e = &graph.edges[ei];
            let mut to = e.to;
            if e.directed && e.arrows.start && e.arrows.end && to == cur {
                to = e.from;
            }
            if to == cur || sucs.contains(&to) || self.chain_of[to] == Some(cid) {
                continue;
            }
            sucs.push(to);
        }
        sucs
    }

    /// Absorb `k`'s node list from `s` onwards into `cid` (upstream
    /// merge): the suffix is re-owned by `cid` outright — upstream
    /// re-points `_chain` for every absorbed node, so a later root's
    /// walk takes over the shared suffix and the earlier chain keeps its
    /// own walk list for layout (its members lose the placement race).
    fn absorb(&mut self, cid: usize, k: usize, s: usize) {
        let suffix: Vec<usize> = self.chains[k]
            .iter()
            .skip_while(|&&x| x != s)
            .copied()
            .collect();
        for x in suffix {
            if self.chain_of[x] != Some(cid) {
                self.chain_of[x] = Some(cid);
                self.chains[cid].push(x);
            }
        }
    }
}

/// Edge indices between each unordered node pair, model order.
fn pair_edges(graph: &Graph) -> std::collections::BTreeMap<(usize, usize), Vec<usize>> {
    let mut map: std::collections::BTreeMap<(usize, usize), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (ei, e) in graph.edges.iter().enumerate() {
        if e.from != e.to {
            map.entry((e.from.min(e.to), e.from.max(e.to)))
                .or_default()
                .push(ei);
        }
    }
    map
}

/// The chain emitter: walks the sorted chains and produces the edge
/// processing order, with placement ownership racing to the first chain.
struct Emitter<'a> {
    graph: &'a Graph,
    chains: &'a Chains,
    pair_edges: std::collections::BTreeMap<(usize, usize), Vec<usize>>,
    order: Vec<usize>,
    emitted: Vec<bool>,
    claimed: Vec<bool>,
    done: Vec<bool>,
}

impl<'a> Emitter<'a> {
    /// Emit one chain's layout (upstream Chain::layout): claim-place the
    /// unclaimed members (opener edge from the chain start — preferring
    /// an edge to an already-claimed neighbor, else the chain's own
    /// first link), then the links between consecutive members, then
    /// splice in the not-yet-laid chains the members' leftover links
    /// reach.
    fn chain(&mut self, k: usize) {
        if self.done[k] {
            return;
        }
        self.done[k] = true;
        let nodes = self.chains.chains[k].clone();
        self.place_members(&nodes);
        self.link_members(&nodes);
        self.selfloops(&nodes);
        self.splice_links(k, &nodes);
    }

    /// Placement phase: claim-place the unclaimed members in walk order
    /// — the chain start via its opener edge, the rest via the link to
    /// the previous member; every node is placed by the first chain to
    /// claim it (upstream's _todo race).
    fn place_members(&mut self, nodes: &[usize]) {
        let mut opened = false;
        for (i, &m) in nodes.iter().enumerate() {
            if self.claimed[m] {
                continue;
            }
            self.claimed[m] = true;
            if i == 0 {
                let ei = self.start_edge(m, nodes.len() >= 2 && !opened, nodes);
                if let Some(ei) = ei {
                    self.emit(ei);
                    opened = true;
                }
            } else if let Some(&ei) = self
                .pair_edges
                .get(&pair(nodes[i - 1], m))
                .and_then(|v| v.iter().find(|&&ei| !self.emitted[ei]))
            {
                self.emit(ei);
            }
        }
    }

    /// Linking phase: still-unrouted edges between consecutive members
    /// (upstream's chain-linking, both already placed — trace shape).
    fn link_members(&mut self, nodes: &[usize]) {
        for w in nodes.windows(2) {
            let eis: Vec<usize> = self
                .pair_edges
                .get(&pair(w[0], w[1]))
                .map(|v| v.as_slice())
                .unwrap_or(&[])
                .to_vec();
            for ei in eis {
                if !self.emitted[ei] {
                    self.emit(ei);
                }
            }
        }
    }

    /// Self-loops on members (upstream runs the self-loop phase before
    /// the recursive link resolution).
    fn selfloops(&mut self, nodes: &[usize]) {
        for &m in nodes {
            for &ei in &self.chains.at[m] {
                let e = &self.graph.edges[ei];
                if e.from == e.to && !self.emitted[ei] {
                    self.emit(ei);
                }
            }
        }
    }

    /// Inter-chain splice (upstream's "all chains that start at this
    /// node"): an edge whose to-end belongs to another not-yet-laid
    /// chain pulls that chain's layout in first (edges sorted by the
    /// target's name); the trigger edge itself is emitted by the
    /// spliced chain's own linking pass, else as this chain's trace.
    fn splice_links(&mut self, k: usize, nodes: &[usize]) {
        for &m in nodes {
            let mut links: Vec<usize> = self.chains.at[m]
                .iter()
                .copied()
                .filter(|&ei| {
                    let to = self.graph.edges[ei].to;
                    to != m && self.chains.chain_of[to].is_some()
                })
                .collect();
            links.sort_by(|&a, &b| {
                let ta = self.graph.edges[a].to;
                let tb = self.graph.edges[b].to;
                self.graph.nodes[ta].name.cmp(&self.graph.nodes[tb].name)
            });
            for ei in links {
                self.splice_one(k, ei);
            }
        }
    }

    /// One inter-chain link: pull in the to-end's chain when it is not
    /// laid yet, then emit the link itself if the splice left it free.
    fn splice_one(&mut self, k: usize, ei: usize) {
        let to = self.graph.edges[ei].to;
        if let Some(l) = self.chains.chain_of[to] {
            if l != k && !self.done[l] {
                self.chain(l);
            }
        }
        if !self.emitted[ei] {
            self.emit(ei);
        }
    }

    /// The placing edge for an unclaimed chain start: an edge to an
    /// already-claimed neighbor (model order, self-loops excluded), else
    /// the chain's own first link when it has one.
    fn start_edge(&self, m: usize, allow_opener: bool, nodes: &[usize]) -> Option<usize> {
        let linked = self.chains.at[m]
            .iter()
            .copied()
            .find(|&ei| {
                let e = &self.graph.edges[ei];
                e.from != e.to && self.claimed[self.other_end(ei, m)]
            });
        if linked.is_some() {
            return linked;
        }
        if allow_opener {
            return self.pair_edges.get(&pair(nodes[0], nodes[1])).and_then(
                |v| v.iter().find(|&&ei| !self.emitted[ei]).copied(),
            );
        }
        None
    }

    /// The far end of edge `ei` seen from `m`.
    fn other_end(&self, ei: usize, m: usize) -> usize {
        let e = &self.graph.edges[ei];
        if e.from == m {
            e.to
        } else {
            e.from
        }
    }

    /// Record `ei` in the order and mark it routed.
    fn emit(&mut self, ei: usize) {
        self.emitted[ei] = true;
        self.order.push(ei);
    }
}

/// Unordered pair key.
fn pair(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}
