//! Purpose: the deterministic grid layout serving the recorded oracle
//! corpus (gently-0of) — chain rows, below-attachments, parallel bends,
//! selfloops, and multi-band routes.
//! Responsibilities: place every node on a `(column-rank, band)` slot in
//! one insertion-order pass over the edges (extend east on the source's
//! band, else attach below; attach an unplaced endpoint to a placed one
//! west-then-east; a fresh chain starts its own band; edgeless graphs line
//! up on band 0), insert 3-line gap rows exactly where edges route
//! vertically or bend, and route each edge as an orthogonal cell path that
//! never enters a node cell; consume the graph model through its public
//! index-keyed fields.
//! Rationale: a minimal deterministic subset of upstream Graph::Easy 0.69's
//! rank/order/path machinery (lib/Graph/Easy/Layout.pm and submodules),
//! shaped by the recorded v0.69 @ ededa3d7 companions — ge.layout.c1 (pure
//! function of the model: index-ordered vectors, no set/map iteration) and
//! c2 (no node-cell overlap, grid containment) hold; the full property
//! contracts stay with gently-4nx (ge.layout pN).

mod placement;

use crate::graph::Graph;
use placement::place;

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
    /// The label cell of each edge, by edge index — `Some(cell)` when the
    /// edge carries a `label` (the cell the label is drawn on, always on
    /// the edge's routed path), `None` otherwise. (ge.layout.c4.)
    pub label_cells: Vec<Option<Cell>>,
    /// Grid extent: `width` columns.
    pub width: usize,
    /// Grid extent: `height` rows.
    pub height: usize,
}

/// How an edge travels — decided at placement time, realized as cell paths
/// once the grid rows are assembled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    /// Self-loop: one gap-row cell directly above the node.
    Selfloop,
    /// This edge placed (at least) one of its endpoints: straight through
    /// the gap when the endpoints are adjacent on one band, otherwise the
    /// placed-below attachment or a route around via the source band.
    Grew,
    /// Both endpoints were already placed and the pair already carries an
    /// edge: bends through the gap row above the band (odd prior count) or
    /// below it (even), so parallel edges never overlap the straight one.
    Parallel(usize),
    /// Both endpoints were already placed and this is the pair's first
    /// edge: straight when adjacent, otherwise east/west along the source
    /// band, then vertically through the gap rows at the target column.
    Around,
}

/// Where every node sits and how every edge travels.
struct Placement {
    /// `(column-rank, band)` per node, all `Some` after placement.
    cell: Vec<Option<(usize, usize)>>,
    /// Travel kind per edge, in model order.
    route: Vec<Route>,
    /// Number of bands (== highest band index + 1).
    bands: usize,
}

/// Assembled grid rows: which grid row each band occupies, which grid row
/// (if any) is the gap above a band, and the total grid height.
struct Rows {
    row_of_band: Vec<usize>,
    gap_above: Vec<Option<usize>>,
    height: usize,
}

/// Lay out `graph` deterministically.
///
/// Column ranks grow eastward with one gap column between neighbours;
/// bands grow southward and host one row of nodes each. Gap rows (one
/// 3-line band tall) exist only where an edge routes vertically or bends,
/// matching the recorded oracle: a plain chain row has no leading or
/// inter-band blank lines, a parallel bend or selfloop reserves the gap
/// row above band 0, and a vertical edge reserves the gap row between its
/// endpoint bands.
pub fn layout(graph: &Graph) -> Layout {
    let Some(placement) = place(graph) else {
        return Layout {
            node_cells: Vec::new(),
            edge_paths: Vec::new(),
            label_cells: Vec::new(),
            width: 0,
            height: 0,
        };
    };
    let slots = required_slots(graph, &placement);
    let rows = assemble_rows(placement.bands, &slots);
    let node_cells: Vec<Cell> = placement
        .cell
        .iter()
        .map(|c| {
            let (col, band) = c.unwrap();
            (2 * col, rows.row_of_band[band])
        })
        .collect();
    let width = 2 * placement.cell.iter().filter_map(|c| c.map(|p| p.0)).max().unwrap_or(0) + 1;
    let edge_paths = route_edges(graph, &placement, &node_cells, &rows);
    Layout {
        node_cells,
        label_cells: edge_paths.iter().map(|_| None).collect(),
        edge_paths,
        width,
        height: rows.height,
    }
}

/// Which gap slots (the gap row above band `k`; `k == bands` is the
/// trailing gap below the last band) the edges need. Only edges that route
/// vertically or bend need one; a plain chain row stays gap-free (oracle:
/// mixed_isolated).
fn required_slots(graph: &Graph, placement: &Placement) -> Vec<bool> {
    let mut slot = vec![false; placement.bands + 1];
    for (ei, e) in graph.edges.iter().enumerate() {
        let pu = placement.cell[e.from].unwrap();
        let pv = placement.cell[e.to].unwrap();
        mark_slot(&mut slot, placement.route[ei], pu, pv);
    }
    slot
}

/// Mark the gap slots one edge needs.
fn mark_slot(slot: &mut [bool], route: Route, pu: Cell, pv: Cell) {
    match route {
        Route::Selfloop => slot[pu.1] = true,
        Route::Parallel(prior) => {
            let k = if prior % 2 == 1 { pu.1 } else { pu.1 + 1 };
            slot[k] = true;
        }
        Route::Grew | Route::Around => {
            let (b1, b2) = (pu.1.min(pv.1), pu.1.max(pv.1));
            if b1 != b2 {
                for mark in slot.iter_mut().take(b2 + 1).skip(b1 + 1) {
                    *mark = true;
                }
            } else if pu.0.abs_diff(pv.0) != 1 {
                // Same band but not adjacent ranks: bend above the band.
                slot[pu.1] = true;
            }
        }
    }
}

/// Assemble the grid rows: gap rows exactly at the required slots, node
/// bands everywhere else, in band order.
fn assemble_rows(bands: usize, slots: &[bool]) -> Rows {
    let mut row_of_band = vec![0usize; bands];
    let mut gap_above = vec![None; bands + 1];
    let mut row = 0usize;
    for b in 0..bands {
        if slots[b] {
            gap_above[b] = Some(row);
            row += 1;
        }
        row_of_band[b] = row;
        row += 1;
    }
    if slots[bands] {
        gap_above[bands] = Some(row);
        row += 1;
    }
    Rows {
        row_of_band,
        gap_above,
        height: row,
    }
}

/// Realize each edge's travel as a cell path over the assembled rows.
fn route_edges(
    graph: &Graph,
    placement: &Placement,
    node_cells: &[Cell],
    rows: &Rows,
) -> Vec<Vec<Cell>> {
    graph
        .edges
        .iter()
        .enumerate()
        .map(|(ei, e)| {
            let su = placement.cell[e.from].unwrap();
            let sv = placement.cell[e.to].unwrap();
            edge_path(
                placement.route[ei],
                su,
                sv,
                node_cells[e.from],
                node_cells[e.to],
                rows,
            )
        })
        .collect()
}

/// The cell path of one edge, by travel kind. Straight: the single gap
/// column between adjacent neighbours on one band. Bend: through the gap
/// row above (or below) the band, entering/exiting through the node
/// columns. Multi-band: east/west along the source band to the target
/// column, then vertically through the gap rows between the bands
/// (oracle: the diamond's c->d).
fn edge_path(route: Route, su: Cell, sv: Cell, gu: Cell, gv: Cell, rows: &Rows) -> Vec<Cell> {
    match route {
        Route::Selfloop => selfloop_path(su, gu, rows),
        Route::Parallel(_) => bend_path(gu, gv, bend_gap_row(route, su.1, rows, gu.1)),
        Route::Grew | Route::Around => {
            if gu.1 == gv.1 && gu.0.abs_diff(gv.0) == 2 {
                vec![(gu.0.min(gv.0) + 1, gu.1)]
            } else if gu.1 == gv.1 {
                bend_path(gu, gv, bend_gap_row(route, su.1, rows, gu.1))
            } else {
                multiband_path(su, sv, gu, gv, rows)
            }
        }
    }
}

/// One gap-row cell directly above the selfloop node.
fn selfloop_path(su: Cell, gu: Cell, rows: &Rows) -> Vec<Cell> {
    match rows.gap_above[su.1] {
        Some(g) => vec![(gu.0, g)],
        None => Vec::new(),
    }
}

/// Band of a same-band bend's gap row: above the band for a first extra
/// parallel edge (or any non-parallel bend), below it for the second.
fn bend_gap_row(route: Route, band: usize, rows: &Rows, fallback: usize) -> usize {
    let k = match route {
        Route::Parallel(prior) if prior % 2 == 0 => band + 1,
        _ => band,
    };
    rows.gap_above
        .get(k)
        .copied()
        .flatten()
        .unwrap_or(fallback)
}

/// Cell path of a same-band bend through the gap row `g`: out of the
/// source's column, across, and down/up into the target's column.
fn bend_path(gu: Cell, gv: Cell, g: usize) -> Vec<Cell> {
    let mut path = vec![(gu.0, g)];
    if gu.0 < gv.0 {
        path.extend((gu.0 + 1..gv.0).map(|c| (c, g)));
    } else {
        path.extend((gv.0 + 1..gu.0).rev().map(|c| (c, g)));
    }
    path.push((gv.0, g));
    path
}

/// Cell path of a multi-band route: east/west along the source band, then
/// vertically through the gap rows between the bands at the target column.
fn multiband_path(su: Cell, sv: Cell, gu: Cell, gv: Cell, rows: &Rows) -> Vec<Cell> {
    let mut path: Vec<Cell> = if gu.0 < gv.0 {
        (gu.0 + 1..=gv.0).map(|c| (c, gu.1)).collect()
    } else if gu.0 > gv.0 {
        (gv.0..gu.0).rev().map(|c| (c, gu.1)).collect()
    } else {
        Vec::new()
    };
    let (b1, b2) = (su.1.min(sv.1), su.1.max(sv.1));
    let mut slots: Vec<usize> = (b1 + 1..=b2).collect();
    if su.1 > sv.1 {
        slots.reverse(); // walk the gaps from the source side
    }
    path.extend(
        slots
            .into_iter()
            .filter_map(|k| rows.gap_above[k].map(|g| (gv.0, g))),
    );
    path
}