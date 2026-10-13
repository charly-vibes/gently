//! Purpose: scope and subgraph handling for the ge.dot_parser grammar
//! (gently-89p, c4). Responsibilities: the statement loop of one scope
//! body — including graph-scope attribute statements (`key=value`,
//! gently-a0k); the scope-boundary rules of the oracle's node stack (held
//! on `P`, see stmt.rs): an edge operator directly after `}` keeps the
//! stack alive (`} ->` continuation), a plain `}` resets it; bare `{ }`
//! scopes keep their nodes ungrouped; `subgraph NAME { ... }` statements
//! and inline subgraph endpoints — groups under their verbatim name except
//! for the graphviz cluster convention (gently-j2i, see `cluster_convention`),
//! with the nameless keyword form and any attribute list before `{` as
//! tokenizing errors. Rationale: split from mod.rs so files stay under the
//! ratchets' limits.
use super::{attrs, err, ParseError, P, ScopeSt};
use crate::graph::Scope;
use super::lex::Tok;

impl P<'_> {
    /// Parse statements until the closing `}` (not consumed — the caller
    /// applies the scope-boundary rules via `scope_end`).
    pub(super) fn scope_body(&mut self, is_group: bool, group: Option<usize>) -> Result<(), ParseError> {
        self.scopes.push(ScopeSt { is_group, group });
        loop {
            self.skip_semis();
            match self.peek_tok() {
                Some(Tok::RBrace) | None => break,
                Some(Tok::LBrace) | Some(Tok::Ident(_)) | Some(Tok::Str(_)) => {
                    if self.graph_attr_stmt()? {
                        continue;
                    }
                    self.statement()?;
                }
                Some(_) => return Err(err(self.line(), "expected a statement")),
            }
        }
        self.scopes.pop();
        Ok(())
    }

    /// Consume the closing `}` of a scope. An edge operator directly after
    /// it (`} ->`) keeps the node stack alive — the operator links the
    /// scope's nodes (plus anything leaked onto the stack); a plain `}`
    /// resets the stack (the oracle's "always reset the stack"). Returns
    /// whether an operator follows. The pending left side survives both
    /// forms: a later bare-scope push still links it (observed: the nodes
    /// before `-> { b }` link a `y` pushed after the scope).
    pub(super) fn scope_end(&mut self) -> Result<bool, ParseError> {
        self.expect_rbrace()?;
        let arrow = matches!(self.peek_tok(), Some(Tok::Arrow(_)));
        if !arrow {
            self.stack.clear();
        }
        Ok(arrow)
    }

    /// A graph-scope attribute statement `key=value` (the oracle's
    /// `_match_graph_attribute` form: bare key, quoted or bare value, at
    /// statement position). For Graphviz, stray attributes apply to the
    /// parent — the innermost enclosing named subgraph (group) or the
    /// graph. The graph target remaps `rankdir` to `flow` (gently-a0k);
    /// every other key lands verbatim. Returns false when the lookahead
    /// is not one (the statement is parsed normally).
    fn graph_attr_stmt(&mut self) -> Result<bool, ParseError> {
        let is_stmt = matches!(self.peek_tok(), Some(Tok::Ident(_)))
            && matches!(self.toks.get(self.pos + 1).map(|t| &t.tok), Some(Tok::Eq));
        if !is_stmt {
            return Ok(false);
        }
        let Tok::Ident(key) = self.toks[self.pos].tok.clone() else {
            unreachable!("matched Ident above")
        };
        self.pos += 2;
        let value = match self.peek_tok() {
            Some(Tok::Str(q)) => {
                let v = attrs::unquote(q);
                self.pos += 1;
                v
            }
            Some(Tok::Ident(s)) => {
                let v = s.clone();
                self.pos += 1;
                v
            }
            _ => return Err(err(self.line(), "expected an attribute value")),
        };
        match self.scopes.iter().rev().find_map(|s| s.group) {
            Some(gidx) => self.g.set_attr(Scope::Group(gidx), &key, &value),
            None if key == "rankdir" => {
                self.g.set_attr(Scope::Graph, "flow", attrs::rankdir_flow(&value));
            }
            None => self.g.set_attr(Scope::Graph, &key, &value),
        }
        Ok(true)
    }

    /// Named subgraph shared by statement and endpoint positions; the
    /// `subgraph` keyword is already peeked, not consumed. The scope's
    /// nodes reach the stack (a group-scope node statement replaces it),
    /// so `subgraph g { x } -> y` links x — no node is created for the
    /// subgraph's name.
    pub(super) fn named_subgraph(&mut self) -> Result<(), ParseError> {
        let start = self.toks[self.pos].start;
        self.pos += 1;
        let name = match self.peek_tok() {
            Some(Tok::Ident(s)) => s.clone(),
            _ => return Err(self.not_recognized(start)),
        };
        if matches!(self.toks.get(self.pos + 1).map(|t| &t.tok), Some(Tok::LBracket)) {
            return Err(self.not_recognized(start));
        }
        self.pos += 1;
        match self.peek_tok() {
            Some(Tok::LBrace) => self.pos += 1,
            _ => return Err(self.not_recognized(start)),
        }
        let gidx = self.g.add_group(&cluster_convention(&name));
        self.scope_body(true, Some(gidx))?;
        self.scope_end()?;
        Ok(())
    }
}

/// The graphviz renderer's cluster convention (gently-j2i,
/// ge.graphviz_render.c4): the renderer emits a named group `name` as the
/// subgraph id `cluster_<name>` and an anonymous group as `cluster<N>`
/// (bare digits after `cluster`, `<N>` the group's internal id). Restoring
/// the dot round trip therefore means: a subgraph id of the form
/// `cluster_<name>` re-parses with group name `<name>` (verbatim restore),
/// and an id of the form `cluster` + one-or-more ASCII digits re-parses as
/// an ANONYMOUS group (empty name). Single deterministic interpretation
/// rule, applied uniformly: every `cluster_`-prefixed id strips the prefix
/// and every `cluster`+digits id is anonymous — a model group genuinely
/// named `cluster_foo` is unambiguous (the renderer emits
/// `cluster_cluster_foo`, which strips back to `cluster_foo`), while a
/// hand-written `subgraph cluster_foo` in foreign DOT resolves to group
/// `foo`. No escape hatches beyond this one rule.
fn cluster_convention(name: &str) -> String {
    if let Some(rest) = name.strip_prefix("cluster_") {
        return rest.to_string();
    }
    if let Some(digits) = name.strip_prefix("cluster") {
        if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
            return String::new(); // anonymous group restored
        }
    }
    name.to_string()
}