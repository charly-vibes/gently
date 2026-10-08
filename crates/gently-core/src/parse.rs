//! Purpose: input parsers for the graph model, one module per input format.
//! Responsibilities: expose the `text` submodule (Graph::Easy text form)
//! behind the tb.cli slice (gently-2po.9); DOT and the full ge.text_parser
//! capability land with gently-bzx. Rationale: mirroring the pipeline's
//! parse stage, the tracer epic wires one text form end-to-end first
//! (ge.text_parser, gently-bzx).

pub mod text;