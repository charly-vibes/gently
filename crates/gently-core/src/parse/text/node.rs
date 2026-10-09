//! Purpose: node-token parsing for ge.text_parser (c1).
//! Responsibilities: read a `[ … ]` token — escaped chars pass through
//! raw, a bare `]` closes, a newline is an unterminated-token error —
//! then unescape, trim, and whitespace-collapse the name; the empty name
//! creates an anonymous node the oracle names `#N` (N odd); the node is
//! interned in the graph, assigned to the innermost open group (c6:
//! exactly one group, move on redeclare), and becomes the standalone
//! attribute target. Rationale: upstream `_new_node` trims and collapses
//! names and the Anon subclass names itself from the object id; the odd
//! counter reproduces the observed generated names (probe:
//! tests/repro/claims/anon-reference.observed).

use super::attrs::{collapse_ws, unescape};
use super::{ParseError, Parser, Target};

/// Parse one `[ … ]` element (caller guarantees the cursor is at `[`),
/// intern it, wire group membership, and set the attribute target.
/// Returns the node index.
pub fn parse_node_element(p: &mut Parser) -> Result<usize, ParseError> {
    let start_line = p.line();
    let raw = scan_name(p).ok_or_else(|| p.err(start_line, "unterminated node token: missing ']'"))?;
    let name = collapse_ws(&unescape(raw.trim()));
    let idx = intern(p, &name);
    if let Some(&group) = p.group_stack.last() {
        p.graph.set_node_group(idx, group);
    }
    p.last = Some(Target::Node(idx));
    Ok(idx)
}

/// The raw text between `[` and the matching `]` (escapes kept verbatim);
/// `None` when the token never closes on its line.
fn scan_name(p: &mut Parser) -> Option<String> {
    p.bump()?; // '['
    let mut name = String::new();
    loop {
        match p.peek() {
            Some(']') => {
                p.bump();
                return Some(name);
            }
            None | Some('\n') => return None,
            Some(c) => {
                name.push(c);
                p.bump();
            }
        }
    }
}

/// Intern by name; the empty name is an anonymous node (c1): named `#N`
/// with the odd counter, findable and reusable by that generated name.
fn intern(p: &mut Parser, name: &str) -> usize {
    if !name.is_empty() {
        return p.graph.add_node(name);
    }
    let anon = format!("#{}", p.anon_node_id);
    p.anon_node_id += 2;
    p.graph.add_node(&anon)
}
