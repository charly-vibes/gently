//! Purpose: minimal parser for the Graph::Easy text form (tb.cli slice,
//! gently-2po.9).
//! Responsibilities: parse `[ <name> ] --> [ <name> ]` edge lines (the
//! `->` spelling accepted too) and bare `[ <name> ]` node lines with
//! flexible whitespace, interning nodes by name in first-seen order; return
//! typed `ParseError`s for anything unparseable — never panic.
//! Rationale: thin slice feeding the tracer pipeline (cli.c3's "nonzero on
//! any parse error" needs a typed failure, not a panic); the full
//! ge.text_parser capability (classes, styles, quoting, attributes) lands
//! with gently-bzx.

use crate::graph::{Edge, Graph, Node};

/// A typed parse failure: 1-based line number and a human description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line number of the offending line.
    pub line: usize,
    /// Human-readable description of why the line cannot be parsed.
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parse the tracer-slice text form into a `Graph`.
///
/// Each non-blank line must be an edge line (`[ a ] --> [ b ]` or
/// `[ a ] -> [ b ]`) or a bare node line (`[ a ]`); node names are interned
/// in first-seen order so edge endpoints index into `graph.nodes`. Blank
/// lines are skipped. Anything else is a typed `ParseError` naming the
/// 1-based line.
pub fn parse(input: &str) -> Result<Graph, ParseError> {
    let mut graph = Graph::default();
    for (idx, raw) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((from, to)) = split_edge(line) {
            let from = intern(&mut graph, from);
            let to = intern(&mut graph, to);
            graph.edges.push(Edge { from, to });
        } else if let Some(name) = single_node(line) {
            intern(&mut graph, name);
        } else {
            return Err(ParseError {
                line: line_no,
                message: format!("cannot parse line: {line:?}"),
            });
        }
    }
    Ok(graph)
}

/// Split an edge line into its two node names, or `None` if the line is not
/// an edge line. Requires the exact shape
/// `[ <from> ] (-->|->) [ <to> ]` with flexible inner whitespace and no
/// trailing junk.
fn split_edge(line: &str) -> Option<(&str, &str)> {
    if !line.starts_with('[') {
        return None;
    }
    let close1 = line.find(']')?;
    let from = node_name(&line[1..close1])?;
    let rest = line[close1 + 1..].trim_start();
    let after_arrow = rest
        .strip_prefix("-->")
        .or_else(|| rest.strip_prefix("->"))?;
    let open2 = after_arrow.find('[')?;
    let body2 = &after_arrow[open2 + 1..];
    let close2 = body2.find(']')?;
    let to = node_name(&body2[..close2])?;
    if body2[close2 + 1..].trim().is_empty() {
        Some((from, to))
    } else {
        None
    }
}

/// Recognize a bare node line `[ <name> ]`, or `None` if the line is not
/// exactly that shape.
fn single_node(line: &str) -> Option<&str> {
    if !line.starts_with('[') {
        return None;
    }
    let close = line.find(']')?;
    let name = node_name(&line[1..close])?;
    if line[close + 1..].trim().is_empty() {
        Some(name)
    } else {
        None
    }
}

/// A node name: non-empty, no brackets. Whitespace-trimmed by the caller.
fn node_name(inner: &str) -> Option<&str> {
    let name = inner.trim();
    if name.is_empty() || name.contains('[') || name.contains(']') {
        None
    } else {
        Some(name)
    }
}

/// Intern `name` into `graph.nodes`, returning its index (first-seen order).
fn intern(graph: &mut Graph, name: &str) -> usize {
    if let Some(i) = graph.nodes.iter().position(|n| n.name == name) {
        i
    } else {
        graph.nodes.push(Node {
            name: name.to_string(),
        });
        graph.nodes.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::graph::{Edge, Graph, Node};

    #[test]
    fn tracer_line_yields_tracer_graph() {
        let g = parse("[ a ] --> [ b ]\n").expect("must parse");
        assert_eq!(g, Graph::tracer());
    }

    #[test]
    fn duplicate_edges_share_interned_nodes() {
        let g = parse("[ a ] --> [ b ]\n[ a ] --> [ b ]\n").expect("must parse");
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(
            g.edges,
            vec![
                Edge { from: 0, to: 1 },
                Edge { from: 0, to: 1 },
            ]
        );
    }

    #[test]
    fn errors_carry_one_based_lines() {
        let e = parse("[ a ]\nnope\n").expect_err("line 2 is junk");
        assert_eq!(e.line, 2);
        let e = parse("[ a ]\n[ b ]\n[ c -->\n").expect_err("line 3 is junk");
        assert_eq!(e.line, 3);
    }

    #[test]
    fn missing_to_node_is_an_error() {
        assert!(parse("[ a ] -->\n").is_err());
        assert!(parse("[ a ] --> [ ]\n").is_err(), "empty node name");
    }

    #[test]
    fn bracket_soup_does_not_panic() {
        assert!(parse("]]][[[\n").is_err());
        assert!(parse("[\n").is_err());
        assert!(parse("]\n").is_err());
        assert!(parse("-->\n").is_err());
    }

    #[test]
    fn blank_lines_and_lone_nodes_parse() {
        let g = parse("\n[ a ]\n\n  \n[ b ]\n").expect("must parse");
        assert_eq!(
            g.nodes,
            vec![Node { name: "a".into() }, Node { name: "b".into() }]
        );
        assert!(g.edges.is_empty());
    }
}