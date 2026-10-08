//! gently-core: the parse -> model -> layout -> render pipeline.
//!
//! Walking-skeleton phase (epic gently-2po): only the tracer shape
//! (`[a] -> [b]`) flows through, deepened one slice at a time by the
//! `tb.*` beads. Capability specs live in `specs/`; deployed deltas in
//! `openspec/specs/`.

/// Walking-skeleton heartbeat. The tracer epic (gently-2po) deepens this
/// crate slice by slice; capability modules land behind the `tb.*` beads.
pub const fn ready() -> bool {
    true
}

pub mod graph;
pub mod render;

#[cfg(test)]
mod tests {
    #[test]
    fn ready_is_true() {
        assert!(super::ready());
    }
}