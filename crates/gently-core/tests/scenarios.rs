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
}
