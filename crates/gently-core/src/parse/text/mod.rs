//! Purpose: parser for the Graph::Easy text form — capability ge.text_parser
//! (gently-bzx), the human-readable format nodes/edges/attributes/groups and
//! class sections are written in, feeding [[ge.graph_model]].
//! Responsibilities: parse the full grammar under the c1–c9 contract —
//! bracketed node tokens with anonymous `#N` nodes (c1), the unit-token
//! edge-operator table (c2/c9), chains (c3), inline edge labels with
//! matching flanks (c4), attribute blocks and class sections (c5), group
//! blocks with one-group membership (c6), `#` comment truncation with `\#`
//! escaping and the hex-colour special case (c7), and typed 1-based-line
//! `ParseError`s for malformed input (c8). Every property row has a
//! scenario in `tests/scenarios/ge_text_parser.rs`; observed behavior was
//! pinned against the oracle in `tests/repro/claims/*.observed`.
//! Rationale: the tracer-slice parser (gently-2po.9) grew into the full
//! grammar in place; module files stay under the pretender ratchet's
//! function-size limits, one concern per file.

mod attrs;
mod clean;
mod edge;
mod node;

use crate::graph::{Graph, ObjectKind, Scope};

/// A typed parse failure: 1-based line number and a human description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line number of the offending line.
    pub line: usize,
    /// Human-readable description of why the input cannot be parsed.
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// The "nearest preceding object" an attribute block or a following
/// operator applies to: the last referenced node or the last closed group.
/// Edges receive attributes only in the `--> { … } [ b ]` position, so they
/// never become the standalone target (upstream stack-top semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Node(usize),
    Group(usize),
}

/// Parse the Graph::Easy text form into a `Graph` (ge.text_parser).
/// Malformed input is a typed `ParseError` naming the 1-based line —
/// never a panic, never a partial graph.
pub fn parse(input: &str) -> Result<Graph, ParseError> {
    let cleaned = clean::strip_comments(input);
    let mut parser = Parser::new(&cleaned);
    parser.parse_elements(false)?;
    Ok(parser.graph)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    graph: Graph,
    last: Option<Target>,
    group_stack: Vec<usize>,
    /// Anonymous node name counter (c1): the oracle names them `#N`, N odd.
    anon_node_id: usize,
    /// Anonymous group name counter (c6): the oracle names them `Group #N`.
    anon_group_id: usize,
}

impl Parser {
    fn new(cleaned: &str) -> Parser {
        Parser {
            chars: cleaned.chars().collect(),
            pos: 0,
            line: 1,
            graph: Graph::default(),
            last: None,
            group_stack: Vec::new(),
            anon_node_id: 1,
            anon_group_id: 0,
        }
    }

    fn err(&self, line: usize, message: impl Into<String>) -> ParseError {
        ParseError { line, message: message.into() }
    }

    fn line(&self) -> usize {
        self.line
    }

    fn parse_elements(&mut self, in_group: bool) -> Result<(), ParseError> {
        loop {
            self.skip_all_ws();
            let line = self.line;
            match self.peek() {
                None => {
                    return if in_group {
                        Err(self.err(line, "group is never closed: missing ')'"))
                    } else {
                        Ok(())
                    }
                }
                Some('[') => self.parse_chain()?,
                Some('(') => self.parse_group()?,
                // a `)` closes the innermost group and hands control back to
                // the enclosing level (top level: a typed error — c8)
                Some(')') => {
                    self.parse_group_end()?;
                    return Ok(());
                }
                Some('{') => self.parse_standalone_attrs()?,
                Some(c) if c.is_ascii_alphabetic() => self.parse_class_section()?,
                Some(c) => return Err(self.err(line, format!("'{c}' not recognized"))),
            }
        }
    }

    /// One chain: a node element followed by zero or more edge segments
    /// (c3). Attr blocks between elements attach to the current node; the
    /// chain pauses at anything that is neither an attr block nor an
    /// operator and lets the element loop dispatch (bare `[ b ]` adjacency
    /// starts a fresh chain — no edge, upstream-faithful).
    fn parse_chain(&mut self) -> Result<(), ParseError> {
        let mut current = node::parse_node_element(self)?;
        loop {
            self.skip_all_ws();
            match self.peek() {
                Some('{') => {
                    let pairs = attrs::parse_block(self)?;
                    self.apply_pairs(pairs, Target::Node(current));
                }
                Some(c) if edge::is_op_start(c) => current = self.parse_edge_segment(current)?,
                _ => return Ok(()),
            }
        }
    }

    /// `--> { edge attrs } [ right ]` from a validated operator match.
    fn parse_edge_segment(&mut self, left: usize) -> Result<usize, ParseError> {
        let start = (self.pos, self.line);
        let branches = [
            edge::Branch::Directed,
            edge::Branch::Labeled,
            edge::Branch::DotArrowless,
            edge::Branch::PlainArrowless,
        ];
        for branch in branches {
            self.pos = start.0;
            self.line = start.1;
            if let Some((end, op)) = edge::match_branch(&self.chars, self.pos, branch) {
                self.pos = end;
                match self.finish_edge(left, op) {
                    Ok(Some(right)) => return Ok(right),
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }
        }
        Err(self.err(start.1, "edge operator not recognized"))
    }

    /// Consume an edge operator's tail: optional attr block (edge attrs),
    /// then the right node token (c2: both endpoints are required). A
    /// structurally broken node token is a hard error; anything else where
    /// a node should be is a soft miss (the caller tries the next branch).
    fn finish_edge(&mut self, left: usize, op: edge::EdgeOp) -> Result<Option<usize>, ParseError> {
        self.skip_all_ws();
        let mut pairs = Vec::new();
        if self.peek() == Some('{') {
            pairs = attrs::parse_block(self)?;
            self.skip_all_ws();
        }
        if self.peek() != Some('[') {
            return Ok(None);
        }
        let right = node::parse_node_element(self)?;
        let eidx = self.new_edge(left, right, &op);
        for (k, v) in pairs {
            self.graph.set_attr(Scope::Edge(eidx), &k, &v);
        }
        Ok(Some(right))
    }

    /// Add the edge for `op`: direction, arrow heads, style and label land
    /// per c2/c4 (solid edges carry no style attribute — solid is the
    /// class-inherited default).
    fn new_edge(&mut self, left: usize, right: usize, op: &edge::EdgeOp) -> usize {
        let eidx = self.graph.add_edge(left, right, op.directed).expect("live endpoints");
        if op.bidirectional {
            self.graph.edges[eidx].arrows.start = true;
        }
        if let Some(style) = op.style {
            self.graph.set_attr(Scope::Edge(eidx), "style", style);
        }
        if let Some(label) = &op.label {
            self.graph.set_attr(Scope::Edge(eidx), "label", label);
        }
        eidx
    }

    fn parse_group(&mut self) -> Result<(), ParseError> {
        self.bump(); // '('
        let name = self.read_group_name()?;
        let gidx = self.new_group(&name);
        self.last = Some(Target::Group(gidx));
        if self.peek() == Some(')') {
            self.bump(); // empty group like "( G: )" — not opened for content
            return Ok(());
        }
        self.group_stack.push(gidx);
        self.parse_elements(true)
    }

    /// The text between `(` and the next `[`, `(`, or `)` — verbatim after
    /// unquoting; the empty text marks an anonymous group (c6).
    fn read_group_name(&mut self) -> Result<String, ParseError> {
        let line = self.line;
        let mut name = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err(line, "unterminated group: missing ')'")),
                Some('[' | '(' | ')') => break,
                Some('\n') => {
                    self.bump();
                }
                Some(_) => {
                    name.push(self.bump().unwrap());
                }
            }
        }
        Ok(attrs::collapse_ws(&attrs::unescape(name.trim())))
    }

    fn new_group(&mut self, name: &str) -> usize {
        if name.is_empty() {
            let name = format!("Group #{}", self.anon_group_id);
            self.anon_group_id += 1;
            return self.graph.add_group(&name);
        }
        self.graph.add_group(name)
    }

    /// `)` closing the innermost open group, optionally followed by an
    /// attribute block for the group (c5/c6).
    fn parse_group_end(&mut self) -> Result<(), ParseError> {
        let line = self.line;
        self.bump();
        let gidx = self
            .group_stack
            .pop()
            .ok_or_else(|| self.err(line, "')' without a matching group"))?;
        self.last = Some(Target::Group(gidx));
        self.skip_all_ws();
        if self.peek() == Some('{') {
            let pairs = attrs::parse_block(self)?;
            self.apply_pairs(pairs, Target::Group(gidx));
        }
        Ok(())
    }

    /// A `{ … }` with no nearer context applies to the nearest preceding
    /// object (c5); with none at all the attributes are dropped silently
    /// (upstream-observed: `{ … }` before any object is not an error).
    fn parse_standalone_attrs(&mut self) -> Result<(), ParseError> {
        let pairs = attrs::parse_block(self)?;
        if let Some(t) = self.last {
            self.apply_pairs(pairs, t);
        }
        Ok(())
    }

    /// `graph { … }` is graph-level; `node`/`edge`/`group { … }` are
    /// class-scoped (c5). Any other identifier is an unknown class (c8).
    fn parse_class_section(&mut self) -> Result<(), ParseError> {
        let line = self.line;
        let ident = self.read_ident();
        let kind = match ident.as_str() {
            "graph" => None,
            "node" => Some(ObjectKind::Node),
            "edge" => Some(ObjectKind::Edge),
            "group" => Some(ObjectKind::Group),
            _ => return Err(self.err(line, format!("unknown class '{ident}'"))),
        };
        self.skip_all_ws();
        if self.peek() != Some('{') {
            return Err(self.err(line, format!("'{ident}' not recognized")));
        }
        let pairs = attrs::parse_block(self)?;
        let scope = kind.map_or(Scope::Graph, |k| Scope::Class(k, String::new()));
        for (k, v) in pairs {
            self.graph.set_attr(scope.clone(), &k, &v);
        }
        Ok(())
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if !(c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-') {
                break;
            }
            ident.push(c);
            self.bump();
        }
        ident
    }

    fn apply_pairs(&mut self, pairs: Vec<(String, String)>, t: Target) {
        let scope = match t {
            Target::Node(i) => Scope::Node(i),
            Target::Group(i) => Scope::Group(i),
        };
        for (k, v) in pairs {
            self.graph.set_attr(scope.clone(), &k, &v);
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
            if c == Some('\n') {
                self.line += 1;
            }
        }
        c
    }

    fn skip_all_ws(&mut self) {
        loop {
            match self.peek() {
                Some('\n') => {
                    self.pos += 1;
                    self.line += 1;
                }
                Some(' ') | Some('\t') | Some('\r') => self.pos += 1,
                _ => return,
            }
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
