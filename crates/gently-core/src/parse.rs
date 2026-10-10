//! Purpose: input parsers for the graph model, one module per input format.
//! Responsibilities: `text` implements the full ge.text_parser capability
//! (gently-bzx) — the Graph::Easy text form: node tokens, the unit-token
//! edge-operator grammar, chains, inline labels, attribute blocks and class
//! sections, groups, and `#` comments — each property row of
//! specs/ge-text_parser.md bound by a scenario test; `dot` implements
//! the ge.dot_parser capability (gently-89p): the Graphviz DOT form.
//! Rationale: the tracer epic (gently-2po.9) wired one text
//! form end-to-end first; the grammar deepened in place without changing
//! renderer behavior.

pub mod dot;
pub mod text;