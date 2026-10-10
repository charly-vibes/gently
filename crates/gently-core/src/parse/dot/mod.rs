//! Purpose: parser for the Graphviz DOT form — capability ge.dot_parser
//! (gently-89p), the input side Graph::Easy provides via
//! `Graph::Easy::Parser::Graphviz`, feeding [[ge.graph_model]].
//! Responsibilities: `digraph`/`graph` headers set only the model graph's
//! `type`/`title` attributes — edge DIRECTION comes from the operator,
//! never the header (c1/c2, per the pinned probes); attribute lists
//! unescape quoted values exactly once onto the right object (c3); named
//! subgraphs become groups under their verbatim name with innermost-only
//! membership, the nameless `subgraph` keyword form (with or without an
//! attribute list) is a tokenizing error, and a bare `{ }` scope keeps its
//! nodes ungrouped — with the oracle's surviving left-edge stack, so the
//! node preceding a bare scope is still linked to the scope's last node
//! (c4); record and HTML-like labels autosplit into `name.N` part nodes
//! with port markers stripped and edges reattached, while unresolvable
//! port references and malformed labels fail with the oracle's typed
//! errors and no partial graph (c5). Every property row has a scenario in
//! `tests/scenarios/ge_dot_parser.rs`; observed behavior is pinned in
//! `tests/repro/claims/*.observed` (Graph::Easy v0.69 @ ededa3d7) — where
//! the POD diverges, the probes win.
//! Rationale: a recursive-descent port of the oracle's scope/edge-stack
//! behavior; grammar helpers live in the `lex`/`attrs`/`record`
//! submodules so every file stays under the pretender ratchet's limits.

mod attrs;
mod lex;
mod record;
mod scope;
mod stmt;

use crate::graph::{Graph, Scope};
use lex::{Tok, Token};

/// An attribute list: ordered `(key, value)` pairs, values unquoted once.
type AttrList = Vec<(String, String)>;
use std::collections::HashMap;

/// A typed parse failure: 1-based line number and a human description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line number of the offending token.
    pub line: usize,
    /// Human-readable description of why the input cannot be parsed.
    pub message: String,
}

pub(super) fn err(line: usize, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        message: message.into(),
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parse a Graphviz DOT source into the graph model. On any failure the
/// typed error is returned and no partial graph escapes.
pub fn parse(src: &str) -> Result<Graph, ParseError> {
    let toks = lex::lex(src)?;
    let mut p = P {
        toks,
        pos: 0,
        src,
        g: Graph::default(),
        scopes: vec![ScopeSt {
            pending: None,
            group: None,
        }],
        split: HashMap::new(),
    };
    p.header()?;
    Ok(p.g)
}

/// An edge-statement endpoint as written, resolved lazily — port refs and
/// split-node references resolve at edge-creation time so the oracle's
/// "Cannot find autosplit node" error can name the failing edge id.
#[derive(Debug, Clone)]
pub(super) enum Ep {
    /// A resolved node index (bare scope consumed as an endpoint).
    Idx(usize),
    /// A node by name (interning + split-map lookup happen at resolve).
    Name(String),
    /// `name:port[:compass]`.
    Port {
        base: String,
        port: String,
        compass: Option<String>,
    },
}

/// One nesting level of the parse: the chain tail (`pending` — the oracle's
/// left-edge stack, the source of the bare-scope spurious edge) and the
/// group nodes declared here belong to (`None` at graph level and in bare
/// scopes — bare scopes keep their nodes ungrouped).
pub(super) struct ScopeSt {
    pub(super) pending: Option<usize>,
    pub(super) group: Option<usize>,
}

pub(super) struct P<'a> {
    pub(super) toks: Vec<Token>,
    pub(super) pos: usize,
    pub(super) src: &'a str,
    pub(super) g: Graph,
    pub(super) scopes: Vec<ScopeSt>,
    /// Original node name → autosplit part indices (`parts[0]` first).
    pub(super) split: HashMap<String, Vec<usize>>,
}

impl<'a> P<'a> {
    pub(super) fn peek_tok(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }

    pub(super) fn line(&self) -> usize {
        self.toks.get(self.pos).map(|t| t.line).unwrap_or(1)
    }

    pub(super) fn not_recognized(&self, start: usize) -> ParseError {
        err(
            self.line(),
            format!("'{}' not recognized by Graph::Easy::Parser::Graphviz", self.src[start..].trim()),
        )
    }

    fn header(&mut self) -> Result<(), ParseError> {
        let kind = match self.peek_tok() {
            Some(Tok::Ident(s)) if s == "digraph" || s == "graph" => s.clone(),
            _ => {
                return Err(err(
                    self.line(),
                    "expected a `digraph' or `graph' header",
                ))
            }
        };
        self.pos += 1;
        if kind == "graph" {
            self.g.attributes.set("type", "undirected");
        }
        let name = match self.peek_tok() {
            Some(Tok::Ident(s)) => Some(s.clone()),
            Some(Tok::Str(q)) => Some(attrs::unquote(q)),
            _ => None,
        };
        if let Some(name) = &name {
            self.pos += 1;
            self.g.attributes.set("title", name);
        }
        match self.peek_tok() {
            Some(Tok::LBrace) => self.pos += 1,
            _ => return Err(err(self.line(), "expected `{' to open the graph body")),
        }
        self.scope_body(None)?;
        self.expect_rbrace()?;
        self.skip_semis();
        if self.pos < self.toks.len() {
            return Err(err(self.line(), "trailing input after the graph body"));
        }
        Ok(())
    }

    fn skip_semis(&mut self) {
        while matches!(self.peek_tok(), Some(Tok::Semi)) {
            self.pos += 1;
        }
    }

    fn expect_rbrace(&mut self) -> Result<(), ParseError> {
        match self.peek_tok() {
            Some(Tok::RBrace) => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(err(self.line(), "expected `}'")),
        }
    }

    /// Resolve an endpoint to a live node index at use time (edge
    /// creation or node-attribute application).
    pub(super) fn resolve(&mut self, ep: &Ep) -> Result<usize, ParseError> {
        match ep {
            Ep::Idx(i) => Ok(*i),
            Ep::Name(name) => self.intern(name),
            Ep::Port {
                base,
                port,
                compass,
            } => self.resolve_port(base, port, compass.as_deref()),
        }
    }

    /// Intern a name: split-map hit resolves to the first part node;
    /// otherwise the node is created (and joins the current group, if
    /// any — innermost named scope wins, membership at creation).
    pub(super) fn intern(&mut self, name: &str) -> Result<usize, ParseError> {
        if let Some(parts) = self.split.get(name) {
            return Ok(parts[0]);
        }
        let fresh = self.g.node_by_name(name).is_none();
        let idx = self.g.add_node(name);
        if fresh {
            if let Some(grp) = self.scopes.last().and_then(|s| s.group) {
                self.g.set_node_group(idx, grp);
            }
        }
        Ok(idx)
    }

    /// Resolve `base:port[:compass]` (c5): the base must be an autosplit
    /// node; numeric ports index the parts, compass points resolve to the
    /// first part; everything else fails with the oracle's typed error.
    pub(super) fn resolve_port(
        &mut self,
        base: &str,
        port: &str,
        compass: Option<&str>,
    ) -> Result<usize, ParseError> {
        let port_err = |p: &P| {
            err(
                p.line(),
                format!("Cannot find autosplit node for {base}:{port} on edge {}", p.g.edges.len() + 1),
            )
        };
        let parts = match self.split.get(base) {
            Some(parts) => parts.clone(),
            None => return Err(port_err(self)),
        };
        if let Ok(i) = port.parse::<usize>() {
            return parts.get(i).copied().ok_or_else(|| port_err(self));
        }
        let is_compass =
            compass.is_some() || matches!(port, "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "nw" | "c");
        if is_compass {
            Ok(parts[0])
        } else {
            Err(port_err(self))
        }
    }

    /// Apply an attribute list to a node, then autosplit if the resulting
    /// node is a record or HTML-like label (c5).
    pub(super) fn apply_node_attrs(
        &mut self,
        idx: usize,
        list: &[(String, String)],
    ) -> Result<(), ParseError> {
        for (k, v) in list {
            self.g.set_attr(Scope::Node(idx), k, v);
        }
        self.maybe_autosplit(idx)
    }

    /// Autosplit a finished node label (c5): a `shape=record` label with
    /// a vertical bar splits flat into parts; an HTML-like table label
    /// splits into one part per `<TD>` cell. Unsplit labels are kept.
    fn maybe_autosplit(&mut self, idx: usize) -> Result<(), ParseError> {
        let shape = self.g.get_attr(Scope::Node(idx), "shape").unwrap_or("").to_string();
        let label = match self.g.get_attr(Scope::Node(idx), "label") {
            Some(label) => label.to_string(),
            None => return Ok(()),
        };
        if shape == "record" {
            if label.contains('{') || label.contains('}') {
                let escaped = record::escape_record_braces(&label);
                self.g.set_attr(Scope::Node(idx), "label", &escaped);
                self.split_record(idx, &escaped)
            } else if label.contains('|') {
                self.split_record(idx, &label)
            } else {
                Ok(())
            }
        } else if label.starts_with('<') && label.ends_with('>') && label.len() >= 2 {
            self.split_html(idx, &label)
        } else {
            Ok(())
        }
    }

    /// The record split: the original node is renamed to the label text
    /// (its `label` attribute dropped, `basename` set to the old name) and
    /// `name.N` part nodes are created — flat on every `|`, the oracle's
    /// own surviving behavior (not Graphviz's nesting semantics).
    fn split_record(&mut self, idx: usize, label: &str) -> Result<(), ParseError> {
        let base = self.g.nodes[idx].name.clone();
        self.g.nodes[idx].name = label.to_string();
        self.g.nodes[idx].attributes.remove("label");
        self.g.set_attr(Scope::Node(idx), "basename", &base);
        self.make_parts(base, label.split('|').count());
        Ok(())
    }

    /// The HTML-like split: bare text outside `<TD>` cells is malformed
    /// (oracle: "not recognized"); parts are the `<TD>` cell texts, the
    /// original node renamed to the raw label with `basename` unset.
    fn split_html(&mut self, idx: usize, label: &str) -> Result<(), ParseError> {
        let texts = match record::html_parts(&label[1..label.len() - 1]) {
            Ok(texts) if !texts.is_empty() => texts,
            _ => {
                return Err(err(
                    self.line(),
                    format!("'{}' not recognized by Graph::Easy::Parser::Graphviz", label),
                ))
            }
        };
        let base = self.g.nodes[idx].name.clone();
        self.g.nodes[idx].name = label.to_string();
        self.g.nodes[idx].attributes.remove("label");
        self.make_parts(base, texts.len());
        Ok(())
    }

    /// Create the `name.N` part nodes for a split node and register the
    /// split map (references to `name` resolve to part 0 from now on).
    fn make_parts(&mut self, base: String, count: usize) {
        let parts: Vec<usize> = (0..count)
            .map(|i| self.g.add_node(&format!("{}.{}", base, i)))
            .collect();
        self.split.insert(base, parts);
    }
}
