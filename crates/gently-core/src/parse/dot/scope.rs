//! Purpose: scope and subgraph handling for the ge.dot_parser grammar
//! (gently-89p, c4). Responsibilities: the statement loop of one scope
//! body; bare `{ }` scopes — nodes ungrouped, with the oracle's surviving
//! left-edge stack (the pinned spurious edge from the enclosing chain
//! tail to the scope's last node); `subgraph NAME { ... }` statements and
//! inline subgraph endpoints — groups under their verbatim name,
//! innermost-only membership, with the nameless keyword form and any
//! attribute list before `{` as tokenizing errors. Rationale: split from
//! mod.rs so files stay under the pretender ratchet's limits.
use super::{err,ParseError,P,ScopeSt};
use super::lex::Tok;

impl P<'_> {
    /// Parse statements until the closing `}` (not consumed). Returns the
    /// scope's chain tail — its last endpoint node, if any.
    pub(super) fn scope_body(&mut self, group: Option<usize>) -> Result<Option<usize>, ParseError> {
        self.scopes.push(ScopeSt {
            pending: None,
            group,
        });
        loop {
            self.skip_semis();
            match self.peek_tok() {
                Some(Tok::RBrace) | None => break,
                Some(Tok::LBrace) => self.bare_scope()?,
                Some(Tok::Ident(s)) if s == "subgraph" => self.subgraph_stmt()?,
                Some(Tok::Ident(_)) | Some(Tok::Str(_)) => {
                    if let Some(last) = self.statement()? {
                        if let Some(scope) = self.scopes.last_mut() {
                            scope.pending = Some(last);
                        }
                    }
                }
                Some(_) => return Err(err(self.line(), "expected a statement")),
            }
        }
        Ok(self.scopes.pop().map(|s| s.pending).unwrap_or(None))
    }

    /// A bare `{ ... }` scope at statement position: nodes ungrouped, and
    /// the oracle's surviving left-edge stack links the enclosing scope's
    /// chain tail to the scope's last node (the pinned spurious edge).
    pub(super) fn bare_scope(&mut self) -> Result<(), ParseError> {
        self.pos += 1;
        let last = self.scope_body(None)?;
        self.expect_rbrace()?;
        let inner = match last {
            Some(inner) => inner,
            None => return Ok(()),
        };
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(outer) = scope.pending {
                self.g.add_edge(outer, inner, true);
            }
            scope.pending = Some(inner);
        }
        Ok(())
    }

    /// A `subgraph NAME { ... }` statement: the group gets the name
    /// verbatim; a nameless keyword form or an attribute list before `{`
    /// is a tokenizing error. Returns nothing — named scopes do not
    /// participate in the edge stack (no spurious edge).
    pub(super) fn subgraph_stmt(&mut self) -> Result<(), ParseError> {
        self.named_subgraph()?;
        Ok(())
    }

    /// Named subgraph shared by statement and endpoint positions; the
    /// `subgraph` keyword is already peeked, not consumed.
    pub(super) fn named_subgraph(&mut self) -> Result<String, ParseError> {
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
        let gidx = self.g.add_group(&name);
        self.scope_body(Some(gidx))?;
        self.expect_rbrace()?;
        Ok(name)
    }

}
