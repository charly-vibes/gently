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
}
