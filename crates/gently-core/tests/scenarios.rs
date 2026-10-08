//! Scenario tests, grouped per capability.
//!
//! Filters used by espectacular contracts (`ah check --run-tests`) match
//! these module paths: `scenarios::<cap>::<pid>`.

mod tb {
    /// Walking-skeleton smoke test (epic gently-2po): the workspace links,
    /// gently-core is loadable, and the tracer shape plumbing is reachable.
    /// Deepened by gently-2po.7/.8/.9/.10.
    #[test]
    fn workspace_smoke() {
        assert!(gently_core::ready());
    }

    /// tb.txt-render (gently-2po.7): minimal canonical txt serialization of
    /// the tracer graph shape, byte-identical to the pinned oracle.
    ///
    /// Oracle (Graph::Easy v0.69 @ ededa3d7, recorded 2026-10-08):
    ///   perl -IGraph-Easy-0.69/lib -MGraph::Easy \
    ///     -e 'my $g = Graph::Easy->new; $g->add_edge("a","b"); print $g->as_txt'
    ///   → "[ a ] --> [ b ]\n"
    /// Deepened by gently-3hv (capability contracts p1–p4).
    mod txt_render {
        use gently_core::{graph::Graph, render::txt};

        #[test]
        fn tracer_shape_matches_oracle() {
            let g = Graph::tracer();
            assert_eq!(txt::render(&g), "[ a ] --> [ b ]\n");
        }
    }

    /// tb.layout (gently-2po.8): one deterministic layout path for the
    /// tracer graph — ranks grow eastward (a west of b, same row), node
    /// cells never overlap and stay inside the grid, and the edge routes
    /// orthogonally from a's east side to b's west side without entering a
    /// node cell. Oracle shape (Graph::Easy v0.69 @ ededa3d7, as_ascii):
    /// `+---+     +---+` / `| a | --> | b |` / `+---+     +---+` —
    /// a west of b, same row, one-cell gap. Deepened by gently-4ht /
    /// gently-4nx (capability contracts p1–p4).
    mod layout {
        use gently_core::{graph::Graph, layout};

        /// ge.layout.c1: the layout is a pure function — identical inputs
        /// produce identical output.
        #[test]
        fn tracer_layout_is_deterministic() {
            let g = Graph::tracer();
            assert_eq!(layout::layout(&g), layout::layout(&g));
        }

        /// Tracer semantics: rank(a) = 0 < rank(b) = 1, ranks growing
        /// eastward, both nodes on the same row.
        #[test]
        fn ranks_grow_eastward_on_one_row() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            let a = l.node_cells[0];
            let b = l.node_cells[1];
            assert!(a.0 < b.0, "a must sit west of b (rank(a)=0 < rank(b)=1)");
            assert_eq!(a.1, b.1, "tracer nodes share one row");
        }

        /// ge.layout.c2: no two node cells overlap; every node fully
        /// inside the grid.
        #[test]
        fn node_cells_do_not_overlap_and_stay_in_grid() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            assert_ne!(l.node_cells[0], l.node_cells[1]);
            for &(x, y) in &l.node_cells {
                assert!(x < l.width && y < l.height, "node ({x},{y}) outside grid");
            }
        }

        /// ge.layout.c4 (tracer slice): the edge path is orthogonal and
        /// grid-connected, touches neither node cell, and runs from a's
        /// east side to b's west side through the gap column(s).
        #[test]
        fn edge_path_is_orthogonal_and_avoids_node_cells() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            let (a, b) = (l.node_cells[0], l.node_cells[1]);
            let path = &l.edge_paths[0];
            assert!(!path.is_empty(), "edge must be routed");
            for w in path.windows(2) {
                let (p, q) = (w[0], w[1]);
                assert!(p.0 == q.0 || p.1 == q.1, "segment {p:?}->{q:?} not orthogonal");
                let dx = (p.0 as isize - q.0 as isize).abs();
                let dy = (p.1 as isize - q.1 as isize).abs();
                assert!(dx + dy == 1, "cells {p:?}->{q:?} not grid-adjacent");
            }
            for &c in path {
                assert_ne!(c, a, "path enters a's cell");
                assert_ne!(c, b, "path enters b's cell");
            }
            assert_eq!(path[0], (a.0 + 1, a.1), "path must start at a's east side");
            assert_eq!(*path.last().unwrap(), (b.0 - 1, b.1), "path must end at b's west side");
        }
    }
}
