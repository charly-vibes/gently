//! Purpose: statement, endpoint, and attribute-list parsing for the
//! ge.dot_parser grammar (gently-89p, c2/c3/c5). Responsibilities: one
//! statement = endpoint (operator endpoint)*; attribute lists after an
//! endpoint with no operator so far belong to that node, after edge
//! endpoints they belong to the statement's edges (values unquoted
//! exactly once); endpoints resolve lazily — ports at edge-creation time
//! so the oracle's "Cannot find autosplit node" error can name the edge
//! id. Rationale: split from mod.rs so files stay under the pretender
//! ratchet's limits.
use super::{attrs, err, AttrList, Ep, ParseError, P};
use crate::graph::Scope;
use super::lex::Tok;

/// One statement's running state.
#[derive(Default)]
struct StmtRun {
    prev: Option<Ep>,
    pending_dir: Option<bool>,
    edges: Vec<usize>,
    edge_attrs: Option<AttrList>,
    last: Option<usize>,
}

impl P<'_> {
    /// One statement: an endpoint, then zero or more `(operator endpoint)`
    /// rounds. Attribute lists after an endpoint with no operator so far
    /// belong to that node; after an edge's endpoints they belong to the
    /// edges. Returns the statement's last resolved node for the scope's
    /// chain tail.
    pub(super) fn statement(&mut self) -> Result<Option<usize>, ParseError> {
        let mut run = StmtRun::default();
        loop {
            let (ep, ep_attrs) = self.endpoint()?;
            self.stmt_step(&mut run, ep, ep_attrs)?;
            let dir = match self.peek_tok() {
                Some(Tok::Arrow(dir)) => *dir,
                _ => break,
            };
            self.pos += 1;
            run.pending_dir = Some(dir);
        }
        self.apply_edge_attrs(&run);
        Ok(run.last)
    }

    /// One statement iteration: apply the endpoint's attributes to the
    /// right object, then form an edge (a pending operator's right side)
    /// or remember the endpoint as the next edge's left side.
    fn stmt_step(
        &mut self,
        run: &mut StmtRun,
        ep: Ep,
        ep_attrs: Option<AttrList>,
    ) -> Result<(), ParseError> {
        let mut resolved = None;
        match &ep_attrs {
            Some(list) if run.pending_dir.is_none() => {
                let idx = self.resolve(&ep)?;
                self.apply_node_attrs(idx, list)?;
                resolved = Some(idx);
                run.last = Some(idx);
            }
            Some(list) => run.edge_attrs = Some(list.clone()),
            None => {}
        }
        if let Some(dir) = run.pending_dir.take() {
            let to = self.resolve(&ep)?;
            let from_ep = run
                .prev
                .take()
                .ok_or_else(|| err(self.line(), "edge without a left endpoint"))?;
            let from = self.resolve(&from_ep)?;
            let id = self
                .g
                .add_edge(from, to, dir)
                .expect("both endpoints are live nodes");
            run.edges.push(id);
            run.last = Some(to);
            run.prev = Some(Ep::Idx(to));
        } else if resolved.is_none() && !matches!(self.peek_tok(), Some(Tok::Arrow(_))) {
            let idx = self.resolve(&ep)?;
            run.last = Some(idx);
            run.prev = Some(Ep::Idx(idx));
        } else {
            run.prev = Some(ep);
        }
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
    /// name, bare scope, or inline named subgraph — plus an optional
    /// attribute list.
    pub(super) fn endpoint(&mut self) -> Result<(Ep, Option<AttrList>), ParseError> {
        let ep = match self.peek_tok() {
            Some(Tok::Str(q)) => {
                let name = attrs::unquote(q);
                self.pos += 1;
                Ep::Name(name)
            }
            Some(Tok::Ident(s)) => {
                if s == "subgraph" {
                    Ep::Name(self.named_subgraph()?)
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
            Some(Tok::LBrace) => {
                self.pos += 1;
                let last = self.scope_body(None)?;
                self.expect_rbrace()?;
                match last {
                    Some(idx) => Ep::Idx(idx),
                    None => return Err(err(self.line(), "empty subgraph in edge statement")),
                }
            }
            _ => return Err(err(self.line(), "expected a node or subgraph")),
        };
        let list = if matches!(self.peek_tok(), Some(Tok::LBracket)) {
            Some(self.attr_list()?)
        } else {
            None
        };
        Ok((ep, list))
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
