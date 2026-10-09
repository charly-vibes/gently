//! Purpose: the configured flow direction of the layout (ge.layout.c3) —
//! parsing the graph-level `flow` attribute and realizing the canonical
//! east-space grid in that direction.
//! Responsibilities: parse `flow` (the spec's `down/up/left/right` and
//! upstream's `south/north/west/east` spellings, defaulting to east — the
//! recorded oracle renders `a --> b` horizontally), and map the canonical
//! east-space grid to the configured direction by pure cell transforms
//! (west: column mirror; south: transpose; north: transpose + row flip),
//! which preserve orthogonality, adjacency, and grid containment.
//! Rationale: the default east flow is byte-pinned by the recorded oracle
//! corpus (tb.corpus), so east runs the oracle-compatible pipeline
//! unchanged; non-east flows appear in no tier-1 fixture and are realized
//! by transforming a strictly-ranked canonical layout so sources sit
//! strictly earlier along the flow axis (ge.layout.c3). Cycles route
//! against the flow, exactly as the probed oracle does
//! (tests/repro/claims/layout-flow-direction.observed — gently-89d owns
//! the c3 re-derivation).

use super::Layout;
use crate::graph::Graph;

/// The configured flow direction (ge.layout.c3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Flow {
    /// Flow grows eastward — the recorded oracle default.
    #[default]
    East,
    /// Flow grows westward: the east layout, columns mirrored.
    West,
    /// Flow grows southward: the strictly-ranked east layout, transposed.
    South,
    /// Flow grows northward: the south layout, rows flipped.
    North,
}

impl Flow {
    /// The graph-level `flow` attribute, defaulting to east (right).
    pub(crate) fn of(graph: &Graph) -> Flow {
        match graph.attributes.get("flow") {
            Some("left") | Some("west") => Flow::West,
            Some("down") | Some("south") => Flow::South,
            Some("up") | Some("north") => Flow::North,
            _ => Flow::East,
        }
    }
}

/// Realize the canonical east-space grid in the configured direction.
pub(crate) fn realize(grid: Layout, flow: Flow) -> Layout {
    match flow {
        Flow::East => grid,
        Flow::West => mirrored_west(grid),
        Flow::South => transposed(grid),
        Flow::North => flipped_rows(transposed(grid)),
    }
}

/// Re-map every published cell (node cells, edge paths, label cells)
/// through `f`.
fn remap<F>(grid: Layout, f: F) -> Layout
where
    F: Fn((usize, usize)) -> (usize, usize),
{
    let mut g = grid;
    g.node_cells = g.node_cells.into_iter().map(&f).collect();
    g.edge_paths = g
        .edge_paths
        .into_iter()
        .map(|path| path.into_iter().map(&f).collect())
        .collect();
    g.label_cells = g.label_cells.into_iter().map(|c| c.map(&f)).collect();
    g
}

/// The west layout: columns mirrored so sources sit east of their
/// targets (strictly earlier along the leftward flow axis); each edge
/// path is reversed so it still starts at the source's port.
fn mirrored_west(grid: Layout) -> Layout {
    let w = grid.width;
    let mut g = remap(grid, |(x, y)| (w - 1 - x, y));
    g.edge_paths = g.edge_paths.into_iter().map(|mut p| { p.reverse(); p }).collect();
    g
}

/// The south layout: the strictly-ranked east layout transposed, so
/// ranks run southward — sources sit strictly north of their targets.
fn transposed(grid: Layout) -> Layout {
    let (w, h) = (grid.width, grid.height);
    let mut g = remap(grid, |(x, y)| (y, x));
    g.width = h;
    g.height = w;
    g}

/// The north layout: the south layout with rows flipped, so sources sit
/// strictly south of their targets (earlier along the northward axis).
fn flipped_rows(grid: Layout) -> Layout {
    let (w, h) = (grid.width, grid.height);
    let mut g = remap(grid, |(x, y)| (x, h - 1 - y));
    g.width = w;
    g.height = h;
    g}

#[cfg(test)]
mod tests {
    use super::{Flow, realize};
    use crate::layout::Layout;

    /// A 2x1 grid (one node at (0,0), width 3, height 1) under each flow.
    fn sample() -> Layout {
        Layout {
            node_cells: vec![(0, 0)],
            edge_paths: vec![vec![(1, 0)]],
            label_cells: vec![None],
            width: 3,
            height: 1,
        }
    }

    #[test]
    fn east_is_identity() {
        assert_eq!(realize(sample(), Flow::East), sample());
    }

    #[test]
    fn west_mirrors_columns_and_reverses_paths() {
        let g = realize(sample(), Flow::West);
        assert_eq!(g.node_cells, vec![(2, 0)]);
        assert_eq!(g.edge_paths, vec![vec![(1, 0)]], "path starts at the source port");
    }

    #[test]
    fn south_transposes() {
        let g = realize(sample(), Flow::South);
        assert_eq!(g.node_cells, vec![(0, 0)]);
        assert_eq!(g.edge_paths, vec![vec![(0, 1)]]);
        assert_eq!((g.width, g.height), (1, 3));
    }

    #[test]
    fn north_flips_rows() {
        let g = realize(sample(), Flow::North);
        assert_eq!(g.node_cells, vec![(0, 2)], "the node row flips with the grid");
        assert_eq!((g.width, g.height), (1, 3));
    }
}
