//! Purpose: renderers for the graph model, one module per output format.
//! Responsibilities: expose the `txt` submodule (canonical Graph::Easy text
//! form) behind the tb.txt-render slice (gently-2po.7); further formats land
//! as later slices. Rationale: mirroring the pipeline's render stage, the
//! tracer epic wires one format end-to-end first (ge-txt_render, gently-3hv).

pub mod txt;
