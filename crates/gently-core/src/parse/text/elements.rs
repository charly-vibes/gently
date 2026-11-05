//! Purpose: the statement loop for ge.text_parser — the dispatcher that
//! walks cleaned source and hands each statement to its element parser.
//! Rationale: extracted from the `Parser` core so the file stays under
//! the gate limits; the decision-trace `parse` stage emits one gated
//! event per statement here.

use super::*;

impl Parser {
    pub(super) fn trace_statement_kind(&mut self, line: usize) {
        if !crate::trace::enabled("parse") {
            return;
        }
        let kind = match self.peek() {
            Some('[') => "node",
            Some(',') => "comma",
            Some('(') => "group",
            Some(')') => "group_end",
            Some('{') => "attrs",
            Some(c) if edge::is_op_start(c) && !self.stack.is_empty() => "edge",
            Some(c) if c.is_ascii_alphabetic() => "class",
            _ => " unrecognized",
        };
        crate::trace::emit("parse", &format!("stmt line={line} kind={kind}"));
    }

    /// `None` (end of source) and `)` (group close) terminate the
    /// statement loop; everything else dispatches to an element parser.
    pub(super) fn statement_done(
        &mut self,
        line: usize,
        in_group: bool,
    ) -> Result<bool, ParseError> {
        match self.peek() {
            None => {
                if in_group {
                    Err(self.err(line, "group is never closed: missing ')'"))
                } else {
                    Ok(true)
                }
            }
            // `)` closes the innermost group (top level: a c8 error)
            Some(')') => {
                self.parse_group_end()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Dispatch one statement to its element parser.
    pub(super) fn parse_statement(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some('[') => self.parse_node_statement()?,
            Some(',') => self.parse_comma_statement()?,
            Some('(') => self.parse_group()?,
            Some('{') => self.parse_standalone_attrs()?,
            // a statement-level operator continues the current statement
            // (e.g. after a closed group); none in flight: c8 error
            Some(c) if edge::is_op_start(c) && !self.stack.is_empty() => {
                self.parse_edge_statement()?;
            }
            Some(c) if c.is_ascii_alphabetic() => self.parse_class_section()?,
            Some(c) => {
                let line = self.line;
                return Err(self.err(line, format!("'{c}' not recognized")));
            }
            None => unreachable!("statement_done handled end of source"),
        }
        Ok(())
    }
}
