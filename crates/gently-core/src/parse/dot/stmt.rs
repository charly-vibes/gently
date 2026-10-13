//! Purpose: statement and endpoint parsing for the ge.dot_parser grammar
//! (gently-89p, c2/c3/c5). Responsibilities: one statement = endpoint
//! (operator endpoint)*; the oracle's left-side NODE STACK (surviving scope
//! boundaries, held on `P`) drives linkage — an operator links EVERY node
//! on the stack to the right endpoint, a group-scope node statement
//! replaces the stack, a bare-scope node statement pushes onto it; scope
//! endpoints hand their nodes over through the stack; attribute lists
//! after an endpoint with no operator so far belong to that node, after
//! edge endpoints they belong to the statement's edges (values unquoted
//! exactly once); endpoints resolve lazily — ports at edge-creation time
//! so the oracle's "Cannot find autosplit node" error can name the edge
//! id. Rationale: split from mod.rs so files stay under the ratchets'
//! limits; the stack model re-derives the oracle's handler machine
//! (gently-a0k, gently-2n5 — see scope.rs for the scope-boundary rules).
use super::lex::Tok;
use super::{attrs, err, AttrList, Ep, ParseError, P};
use crate::graph::Scope;

/// One statement's running state.
#[derive(Default)]
struct StmtRun {
    edges: Vec<usize>,
    edge_attrs: Option<AttrList>,
    last: Option<usize>,
}

impl P<'_> {
    /// One statement: an endpoint, then zero or more `(operator endpoint)`
    /// rounds. Attribute lists after an endpoint with no operator so far
    /// belong to that node; after an edge's endpoints they belong to the
    /// edges. Returns the statement's last resolved node.
    pub(super) fn statement(&mut self) -> Result<(), ParseError> {
        let mut run = StmtRun::default();
        // the pending operator: direction, the arrow token's byte offset
        // and line (for the oracle's empty-stack not-recognized error)
        let mut pending: Option<(bool, usize, usize)> = None;
        loop {
            let (ep, ep_attrs) = self.endpoint(pending)?; // may consume a scope
            self.stmt_step(&mut run, &mut pending, ep, ep_attrs)?;
            let dir = match self.peek_tok() {
                Some(Tok::Arrow(dir)) => *dir,
                _ => break,
            };
            let tok = self.toks[self.pos].clone();
            self.pos += 1;
            pending = Some((dir, tok.start, tok.line));
        }
        self.apply_edge_attrs(&run);
        Ok(())
    }

    /// One statement iteration: apply the endpoint's attributes to the
    /// right object, then form edges (a pending operator's right side —
    /// one edge per stack node) or update the stack for the next operator.
    fn stmt_step(
        &mut self,
        run: &mut StmtRun,
        pending: &mut Option<(bool, usize, usize)>,
        ep: Ep,
        ep_attrs: Option<AttrList>,
    ) -> Result<(), ParseError> {
        if matches!(ep, Ep::Scope) {
            // the scope's nodes are already on the stack; an attribute list
            // after the scope is dropped (the oracle's stray-attr handler
            // finds an empty stack after the plain-`}` reset — an operator
            // after one fails in the next round via the empty stack). The
            // arrow, if any, is still pending: the scope IS its left side.
            run.last = self.stack.last().copied();
            return Ok(());
        }
        if let Some((dir, start, line)) = pending.take() {
            self.edge_step(run, dir, start, line, ep, ep_attrs)?;
        } else {
            self.node_step(run, ep, ep_attrs)?;
        }
        Ok(())
    }

    /// The right side of a pending operator: one edge per node on the
    /// stack (the oracle's `_link_lists`), the stack replaced by the right
    /// node, the left snapshot kept as the pending left side for bare-scope
    /// pushes. An empty stack is the oracle's not-recognized backbuffer
    /// error (nothing can link).
    fn edge_step(
        &mut self,
        run: &mut StmtRun,
        dir: bool,
        start: usize,
        line: usize,
        ep: Ep,
        ep_attrs: Option<AttrList>,
    ) -> Result<(), ParseError> {
        if self.stack.is_empty() {
            return Err(self.arrow_not_recognized(start, line));
        }
        let to = self.resolve(&ep)?;
        if let Some(list) = ep_attrs {
            run.edge_attrs = Some(list);
        }
        let left = std::mem::take(&mut self.stack);
        for &from in &left {
            let id = self
                .g
                .add_edge(from, to, dir)
                .expect("both endpoints are live nodes");
            run.edges.push(id);
        }
        self.left = Some((dir, left));
        self.stack = vec![to];
        run.last = Some(to);
        Ok(())
    }

    /// The first endpoint of a statement: a group-scope node statement
    /// replaces the stack and forgets any pending left side (the oracle's
    /// node-with-attributes handler); a bare-scope node statement pushes
    /// onto the stack, linking the pending left side if one is set (kept —
    /// later pushes link it too, per the oracle's `-> { b c }` form).
    fn node_step(
        &mut self,
        run: &mut StmtRun,
        ep: Ep,
        ep_attrs: Option<AttrList>,
    ) -> Result<(), ParseError> {
        let idx = self.resolve(&ep)?;
        if let Some(list) = &ep_attrs {
            self.apply_node_attrs(idx, list)?;
        }
        match self.scopes.last().map(|s| s.is_group) {
            Some(true) | None => {
                self.stack = vec![idx];
                self.left = None;
            }
            Some(false) => {
                if let Some((dir, left)) = &self.left {
                    let left = left.clone();
                    for &from in &left {
                        let id = self
                            .g
                            .add_edge(from, idx, *dir)
                            .expect("both endpoints are live nodes");
                        run.edges.push(id);
                    }
                }
                self.stack.push(idx);
            }
        }
        run.last = Some(idx);
        Ok(())
    }

    /// Statement-end edge attributes land on every edge the statement
    /// formed (DOT's attr_list semantics for edge statements).
    fn apply_edge_attrs(&mut self, run: &StmtRun) {
        if let (Some(list), edges) = (&run.edge_attrs, &run.edges) {
            for &id in edges {
                for (k, v) in list {
                    self.g.set_attr(Scope::Edge(id), k, v);
                }
            }
        }
    }

    /// One endpoint unit: name (optionally `:port[:compass]`), quoted
    /// name, bare scope, inline named subgraph — plus an optional
    /// attribute list. A scope endpoint parses its whole body; its nodes
    /// reach linkage through the stack (`Ep::Scope`). The pending operator
    /// decides the scope form: after an operator the scope's pushed nodes
    /// link the left snapshot; at statement position a fresh left side.
    pub(super) fn endpoint(
        &mut self,
        pending: Option<(bool, usize, usize)>,
    ) -> Result<(Ep, Option<AttrList>), ParseError> {
        let ep = match self.peek_tok() {
            Some(Tok::Str(q)) => {
                let name = attrs::unquote(q);
                self.pos += 1;
                Ep::Name(name)
            }
            Some(Tok::Ident(s)) => {
                if s == "subgraph" {
                    self.named_subgraph()?;
                    Ep::Scope
                } else {
                    let base = s.clone();
                    self.pos += 1;
                    if matches!(self.peek_tok(), Some(Tok::Colon)) {
                        self.port_ep(&base)?
                    } else {
                        Ep::Name(base)
                    }
                }
            }
            Some(Tok::LBrace) => self.scope_endpoint(pending)?,
            _ => return Err(err(self.line(), "expected a node or subgraph")),
        };
        let list = if matches!(self.peek_tok(), Some(Tok::LBracket)) {
            Some(self.attr_list()?)
        } else {
            None
        };
        Ok((ep, list))
    }

    /// A scope endpoint (bare `{ ... }` or, via the `subgraph` keyword,
    /// an inline named subgraph): the `{` is already peeked, not consumed.
    /// The pending operator decides the scope form: after an operator the
    /// scope's pushed nodes link the left snapshot (the oracle's
    /// edge+scope form); at statement position a fresh left side.
    fn scope_endpoint(&mut self, pending: Option<(bool, usize, usize)>) -> Result<Ep, ParseError> {
        self.pos += 1;
        match pending {
            Some((dir, _, _)) => {
                self.left = Some((dir, std::mem::take(&mut self.stack)));
            }
            None => self.left = None, // fresh left side at `{`
        }
        self.scope_body(false, None)?;
        self.scope_end()?;
        Ok(Ep::Scope)
    }

    /// The `:port[:compass]` suffix after a base name; the `:` is peeked.
    pub(super) fn port_ep(&mut self, base: &str) -> Result<Ep, ParseError> {
        self.pos += 1;
        let port = match self.peek_tok() {
            Some(Tok::Ident(s)) => s.clone(),
            _ => return Err(err(self.line(), "expected a port after `:'")),
        };
        self.pos += 1;
        let compass = if matches!(self.peek_tok(), Some(Tok::Colon)) {
            self.pos += 1;
            match self.peek_tok() {
                Some(Tok::Ident(s)) => Some(s.clone()),
                _ => return Err(err(self.line(), "expected a compass point after `:'")),
            }
        } else {
            None
        };
        if compass.is_some() {
            self.pos += 1;
        }
        Ok(Ep::Port {
            base: base.to_string(),
            port,
            compass,
        })
    }

    /// An attribute list `[k=v, k2=v2]`; values unescape exactly once.
    pub(super) fn attr_list(&mut self) -> Result<Vec<(String, String)>, ParseError> {
        self.pos += 1;
        let mut list = Vec::new();
        loop {
            while matches!(self.peek_tok(), Some(Tok::Comma) | Some(Tok::Semi)) {
                self.pos += 1;
            }
            match self.peek_tok() {
                Some(Tok::RBracket) => {
                    self.pos += 1;
                    return Ok(list);
                }
                Some(Tok::Ident(_)) | Some(Tok::Str(_)) => {
                    let key = match self.toks[self.pos].tok.clone() {
                        Tok::Str(q) => attrs::unquote(&q),
                        Tok::Ident(s) => s,
                        _ => unreachable!("matched above"),
                    };
                    self.pos += 1;
                    let value = if matches!(self.peek_tok(), Some(Tok::Eq)) {
                        self.pos += 1;
                        self.attr_value()?
                    } else {
                        String::new()
                    };
                    list.push((key, value));
                }
                _ => return Err(err(self.line(), "expected an attribute key or `]'")),
            }
        }
    }

    /// One attribute value: bare identifier, quoted string (unquoted
    /// exactly once), or raw HTML-like `<...>` string (brackets kept —
    /// the oracle keeps them in the node name after autosplit).
    pub(super) fn attr_value(&mut self) -> Result<String, ParseError> {
        let value = match self.peek_tok() {
            Some(Tok::Ident(s)) => s.clone(),
            Some(Tok::Str(q)) => attrs::unquote(q),
            Some(Tok::Html(h)) => h.clone(),
            _ => return Err(err(self.line(), "expected an attribute value")),
        };
        self.pos += 1;
        Ok(value)
    }
}
