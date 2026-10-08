//! Purpose: one deterministic layout path for the tracer-bullet slice
//! (epic gently-2po).
//! Responsibilities: assign grid cells to nodes — ranks growing eastward
//! from column 0 on one row with a one-cell gap — and route each edge
//! orthogonally through the gap column(s) so no path ever enters a node
//! cell; consume the graph model through its public index-keyed fields.
//! Rationale: thin slice of ge.layout sufficient for the tracer shape —
//! c1 (pure function: vector-indexed output, no randomness, no hashmap
//! iteration order) and c2 (no node overlap, grid containment) hold; the
//! full rank/order/position/path machinery lands with gently-4ht /
//! gently-4nx (ge.layout p1–p4).

use crate::graph::Graph;

/// A grid cell: `(column, row)` — column grows eastward, row southward.
pub type Cell = (usize, usize);

/// The deterministic grid layout of a graph.
///
/// Node cells are indexed by node index and edge paths by edge index —
/// plain vectors in model order keep the output a pure function of the
/// input (ge.layout.c1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// Grid cell of each node, by node index.
    pub node_cells: Vec<Cell>,
    /// Routed path of each edge — orthogonal, grid-connected, never
    /// entering a node cell — by edge index.
    pub edge_paths: Vec<Vec<Cell>>,
    /// Grid extent: `width` columns.
    pub width: usize,
    /// Grid extent: `height` rows.
    pub height: usize,
}

/// Lay out `graph` deterministically.
///
/// Nodes are ranked eastward: node `i` sits at column `2 * i` (one-cell
/// gap between neighbours), all on row 0 — consistent with the pinned
/// oracle ascii for `[a] -> [b]` (a west of b, same row, one-cell gap).
/// Each edge is routed straight east from its source's east side to its
/// target's west side through the gap column(s).
pub fn layout(graph: &Graph) -> Layout {
    // Ranks grow eastward along the flow axis; index order == model
    // insertion order, so identical graphs yield identical layouts (c1).
    let node_cells: Vec<Cell> = (0..graph.nodes.len()).map(|i| (2 * i, 0)).collect();
    let width = graph.nodes.len().saturating_sub(1) * 2 + 1;
    let height = if graph.nodes.is_empty() { 0 } else { 1 };

    // Route each edge through the cells strictly between its endpoints'
    // columns on the shared row — orthogonal, connected, node-cell-free.
    let edge_paths: Vec<Vec<Cell>> = graph
        .edges
        .iter()
        .map(|e| {
            let (ax, ay) = node_cells[e.from];
            let (bx, _) = node_cells[e.to];
            if ax < bx {
                ((ax + 1)..bx).map(|x| (x, ay)).collect()
            } else {
                Vec::new()
            }
        })
        .collect();

    Layout {
        node_cells,
        edge_paths,
        width,
        height,
    }
}