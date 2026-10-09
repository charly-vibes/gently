//! Purpose: renderers for the graph model, one module per output format.
//! Responsibilities: expose the `txt` submodule (canonical Graph::Easy text
//! form) behind the tb.txt-render slice (gently-2po.7) and the `ascii`
//! submodule (ascii-art boxes and arrows) behind the tb.cli slice
//! (gently-2po.9); further formats land as later slices. Rationale: mirroring
//! the pipeline's render stage, the tracer epic wires one format end-to-end
//! first (ge-txt_render, gently-3hv) and the ascii face of the same shape
//! second (ge-ascii_render).

pub mod ascii;
pub mod boxart;
pub mod txt;
