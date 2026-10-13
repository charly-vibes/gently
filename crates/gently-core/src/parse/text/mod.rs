//! Purpose: parser for the Graph::Easy text form — capability ge.text_parser
//! (gently-bzx), the human-readable format nodes/edges/attributes/groups and
//! class sections are written in, feeding [[ge.graph_model]].
//! Responsibilities: parse the full grammar under the c1–c13 contract —
//! bracketed node tokens with anonymous `#N` nodes (c1), the unit-token
//! edge-operator table (c2/c9), chains (c3), inline edge labels with
//! matching flanks (c4), attribute blocks and class sections (c5), group
//! blocks with one-group membership and post-group linkage (c6/c12), `#`
//! comment truncation (c7), typed 1-based-line `ParseError`s (c8),
//! comma-separated node lists (c11), and trailing attribute blocks (c13).
//! Every property row has a scenario under `tests/scenarios/`; observed
//! behavior was pinned against the oracle in tests/repro/claims. Rationale:
//! the tracer slice (gently-2po.9) grew into the full grammar in place.

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

/// One entry of the upstream statement stack: a node or a closed group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackItem {
    Node(usize),
    Group(usize),
}

/// The remembered left side of the last edge operator (upstream
/// `left_edge`/`left_stack`), reused by every following comma-list element.
#[derive(Debug, Clone)]
struct LeftEdge {
    op: edge::EdgeOp,
    attrs: Vec<(String, String)>,
}

/// Parse the text form into a `Graph` (ge.text_parser); malformed input is
/// a typed 1-based-line `ParseError` — never a panic, never a partial graph.
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
    /// The statement's objects (upstream `stack`): nodes and, after `)`, a
    /// closed group.
    stack: Vec<StackItem>,
    /// The last edge operator's left side + the stack it linked from.
    left: Option<LeftEdge>,
    left_stack: Vec<StackItem>,
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
            stack: Vec::new(),
            left: None,
            left_stack: Vec::new(),
            group_stack: Vec::new(),
            anon_node_id: 1,
            anon_group_id: 0,
        }
    }

    fn err(&self, line: usize, message: impl Into<String>) -> ParseError {
        ParseError {
            line,
            message: message.into(),
        }
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
                Some('[') => self.parse_node_statement()?,
                Some(',') => self.parse_comma_statement()?,
                Some('(') => self.parse_group()?,
                // `)` closes the innermost group (top level: a c8 error)
                Some(')') => {
                    self.parse_group_end()?;
                    return Ok(());
                }
                Some('{') => self.parse_standalone_attrs()?,
                // a statement-level operator continues the current statement
                // (e.g. after a closed group); none in flight: c8 error
                Some(c) if edge::is_op_start(c) && !self.stack.is_empty() => {
                    self.parse_edge_statement()?;
                }
                Some(c) if c.is_ascii_alphabetic() => self.parse_class_section()?,
                Some(c) => return Err(self.err(line, format!("'{c}' not recognized"))),
            }
        }
    }

    /// One node statement: `[ A ]` replaces the statement stack with the
    /// node and resets the remembered left side (c1/c3; upstream handler).
    /// What follows — attrs, operators, commas — dispatches in the loop.
    fn parse_node_statement(&mut self) -> Result<(), ParseError> {
        let idx = node::parse_node_element(self)?;
        self.stack = vec![StackItem::Node(idx)];
        self.left = None;
        self.left_stack.clear();
        Ok(())
    }

    /// `, [ B ]` extends the current statement's list (c11): the node is
    /// pushed onto the stack, and a previously matched edge operator links
    /// every node of its remembered left side to the new node.
    fn parse_comma_statement(&mut self) -> Result<(), ParseError> {
        self.bump(); // ',' — upstream matches `\s*,\s*`
        self.skip_all_ws();
        let idx = node::parse_node_element(self)?;
        if let Some(left) = self.left.clone() {
            for item in self.left_stack.clone() {
                self.link(item, StackItem::Node(idx), &left.op, &left.attrs);
            }
        }
        self.stack.push(StackItem::Node(idx));
        Ok(())
    }

    /// One edge statement: operator + optional edge attrs + right node
    /// (c2). Every object of the current stack links to the new node; the
    /// operator's left side is remembered for comma lists (c11).
    fn parse_edge_statement(&mut self) -> Result<(), ParseError> {
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
                match self.finish_edge(op) {
                    Ok(Some(())) => return Ok(()),
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }
        }
        Err(self.err(start.1, "edge operator not recognized"))
    }

    /// Consume an edge operator's tail: optional edge attrs, then the right
    /// node token (c2). A broken node token is hard; else a soft miss.
    fn finish_edge(&mut self, op: edge::EdgeOp) -> Result<Option<()>, ParseError> {
        self.skip_all_ws();
        let mut attrs = Vec::new();
        if self.peek() == Some('{') {
            attrs = attrs::parse_block(self)?;
            self.skip_all_ws();
        }
        if self.peek() != Some('[') {
            return Ok(None);
        }
        let right = node::parse_node_element(self)?;
        let lefts = std::mem::take(&mut self.stack);
        for item in &lefts {
            self.link(*item, StackItem::Node(right), &op, &attrs);
        }
        self.left = Some(LeftEdge { op, attrs });
        self.left_stack = lefts;
        self.stack = vec![StackItem::Node(right)];
        Ok(Some(()))
    }

    /// Link one stack item to a node (c3/c11): node-to-node creates the
    /// edge per `op`; a group endpoint records no edge (bounded divergence:
    /// even the pinned oracle's own layouter drops group-endpoint edges).
    fn link(
        &mut self,
        from: StackItem,
        to: StackItem,
        op: &edge::EdgeOp,
        attrs: &[(String, String)],
    ) {
        let (f, t) = match (from, to) {
            (StackItem::Node(f), StackItem::Node(t)) => (f, t),
            _ => return,
        };
        let eidx = self.new_edge(f, t, op);
        for (k, v) in attrs {
            self.graph.set_attr(Scope::Edge(eidx), k, v);
        }
    }

    /// Add the edge for `op`: direction/arrows/style/label per c2/c4 (solid
    /// carries no style attribute — the class default).
    fn new_edge(&mut self, left: usize, right: usize, op: &edge::EdgeOp) -> usize {
        let eidx = self
            .graph
            .add_edge(left, right, op.directed)
            .expect("live endpoints");
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
        if self.peek() == Some(')') {
            self.bump(); // empty group like "( G: )" — closed, on the stack
            self.stack = vec![StackItem::Group(gidx)];
            return Ok(());
        }
        self.group_stack.push(gidx);
        self.parse_elements(true)
    }

    /// The text between `(` and the next `[`/`(`/`)`; empty = anon group.
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

    /// `)` closing the innermost open group + optional attr block (c5/c6);
    /// the closed group becomes the statement's object (c12).
    fn parse_group_end(&mut self) -> Result<(), ParseError> {
        let line = self.line;
        self.bump();
        let gidx = self
            .group_stack
            .pop()
            .ok_or_else(|| self.err(line, "')' without a matching group"))?;
        self.stack = vec![StackItem::Group(gidx)];
        self.skip_all_ws();
        if self.peek() == Some('{') {
            let pairs = attrs::parse_block(self)?;
            self.apply_pairs(&pairs, StackItem::Group(gidx));
        }
        Ok(())
    }

    /// A `{ … }` with no nearer context applies to every object of the
    /// statement stack — the comma list, or (empty stack) every open group;
    /// with neither it is dropped silently (upstream: no error).
    fn parse_standalone_attrs(&mut self) -> Result<(), ParseError> {
        let pairs = attrs::parse_block(self)?;
        if !self.stack.is_empty() {
            for item in self.stack.clone() {
                self.apply_pairs(&pairs, item);
            }
        } else {
            for &g in self.group_stack.clone().iter() {
                self.apply_pairs(&pairs, StackItem::Group(g));
            }
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
        // a class section forgets the statement stack + remembered side
        self.stack.clear();
        self.left = None;
        self.left_stack.clear();
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

    fn apply_pairs(&mut self, pairs: &[(String, String)], t: StackItem) {
        let scope = match t {
            StackItem::Node(i) => Scope::Node(i),
            StackItem::Group(i) => Scope::Group(i),
        };
        for (k, v) in pairs {
            self.graph.set_attr(scope.clone(), k, v);
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
