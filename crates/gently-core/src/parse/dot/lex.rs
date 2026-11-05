//! Purpose: the tokenizer for the ge.dot_parser DOT grammar (gently-89p).
//! Responsibilities: turn raw DOT source into a flat `Token` stream —
//! braces, brackets, separators, the two edge operators (`->` directed,
//! `--` undirected), identifiers, quoted strings with escapes kept raw
//! (unquoted exactly once at parse time), and `<...>` HTML-like strings
//! balanced by angle-bracket depth; each token carries its 1-based line
//! and a byte offset into the source for error quoting. Rationale: a
//! small hand-rolled lexer keeps the recursive-descent parser in `mod.rs`
//! free of character-scanning concerns and matches the `text/` grammar
//! module's one-concern-per-file layout.

use super::err;

/// A lexical token of the DOT grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Tok {
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semi,
    Comma,
    Colon,
    Eq,
    /// `->` (directed, `true`) or `--` (undirected, `false`).
    Arrow(bool),
    Ident(String),
    /// Raw quoted-string content, escapes intact (the parser unquotes
    /// exactly once — the model stores values verbatim).
    Str(String),
    /// Raw `<...>` HTML-like string, outer brackets included.
    Html(String),
}

/// One token with its source position for typed errors.
#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub tok: Tok,
    /// 1-based line of the token's first character.
    pub line: usize,
    /// Byte offset of the token's first character in the source.
    pub start: usize,
}

fn ident_char(c: char) -> bool {
    c.is_alphanumeric() || "_.$+".contains(c)
}

/// Single-character tokens (everything `match`-able in one place).
fn simple_token(c: char) -> Option<Tok> {
    Some(match c {
        '{' => Tok::LBrace,
        '}' => Tok::RBrace,
        '[' => Tok::LBracket,
        ']' => Tok::RBracket,
        ';' => Tok::Semi,
        ',' => Tok::Comma,
        ':' => Tok::Colon,
        '=' => Tok::Eq,
        _ => return None,
    })
}

/// Cursor over the char vector: index, 1-based line, byte offset. One
/// cohesive scanner state instead of three parallel mutable locals.
#[derive(Default)]
struct Cursor {
    i: usize,
    line: usize,
    byte: usize,
}

impl Cursor {
    fn byte_len(chars: &[char], from: usize, to: usize) -> usize {
        chars[from..to].iter().map(|c| c.len_utf8()).sum()
    }
}

/// Tokenize `src`; unknown characters and unterminated strings/HTML-like
/// strings are tokenizing errors (c5 — the no-partial-graph path).
pub(super) fn lex(src: &str) -> Result<Vec<Token>, super::ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks = Vec::new();
    let mut cur = Cursor {
        i: 0,
        line: 1,
        byte: 0,
    };
    while cur.i < chars.len() {
        let c = chars[cur.i];
        let scanned = scan_token(&chars, cur.i, cur.line)?;
        let (t, end, newlines) = match scanned {
            Some(scanned) => scanned,
            None => {
                // whitespace: newline chars advance the error line
                cur.line += usize::from(c == '\n');
                cur.i += 1;
                cur.byte += c.len_utf8();
                continue;
            }
        };
        toks.push(Token {
            tok: t,
            line: cur.line,
            start: cur.byte,
        });
        cur.line += newlines;
        cur.byte += Cursor::byte_len(&chars, cur.i, end);
        cur.i = end;
    }
    Ok(toks)
}

/// Scan one token starting at `i`: `Some((tok, end-index, newlines-inside))`
/// for real tokens, `None` for skipped whitespace (newline counting is the
/// caller's business). Unknown characters are tokenizing errors (c5 — the
/// no-partial-graph path).
fn scan_token(
    chars: &[char],
    i: usize,
    line: usize,
) -> Result<Option<(Tok, usize, usize)>, super::ParseError> {
    match chars[i] {
        ' ' | '\t' | '\r' | '\n' => Ok(None),
        '-' => arrow(chars, i, line).map(Some),
        '"' => quoted(chars, i, line).map(Some),
        '<' => html(chars, i, line).map(Some),
        c if ident_char(c) => {
            let (t, end) = ident(chars, i);
            Ok(Some((t, end, 0)))
        }
        c => match simple_token(c) {
            Some(t) => Ok(Some((t, i + 1, 0))),
            None => Err(err(line, format!("unexpected character {c:?}"))),
        },
    }
}

/// `->` and `--` are the only recognized `-` forms.
fn arrow(chars: &[char], i: usize, line: usize) -> Result<(Tok, usize, usize), super::ParseError> {
    match chars.get(i + 1) {
        Some('>') => Ok((Tok::Arrow(true), i + 2, 0)),
        Some('-') => Ok((Tok::Arrow(false), i + 2, 0)),
        _ => Err(err(
            line,
            "unexpected `-' (only `->' and `--' are recognized)",
        )),
    }
}

/// Running scan state for a multi-char token: raw content so far, next
/// index, and newlines seen (keeps error lines honest). One cohesive
/// state instead of parallel mutable locals.
struct StrScan {
    raw: String,
    j: usize,
    newlines: usize,
}

impl StrScan {
    fn at(start: usize) -> Self {
        StrScan {
            raw: String::new(),
            j: start,
            newlines: 0,
        }
    }
    fn push(&mut self, c: char) {
        self.newlines += usize::from(c == '\n');
        self.raw.push(c);
        self.j += 1;
    }
}

/// A quoted string: raw content with escapes intact, through the unescaped
/// closing `"`; counted newlines keep error lines honest.
fn quoted(chars: &[char], i: usize, line: usize) -> Result<(Tok, usize, usize), super::ParseError> {
    let mut s = StrScan::at(i + 1);
    loop {
        match chars.get(s.j) {
            None => return Err(err(line, "unterminated quoted string")),
            Some('"') => return Ok((Tok::Str(s.raw), s.j + 1, s.newlines)),
            // escapes stay raw: the backslash and the escaped char both
            // land in the token verbatim
            Some('\\') => {
                s.raw.push('\\');
                if let Some(&nc) = chars.get(s.j + 1) {
                    s.push(nc); // advances past the escaped char
                }
                s.j += 1; // advance past the backslash
            }
            Some(&nc) => s.push(nc),
        }
    }
}

/// An HTML-like `<...>` string, balanced by angle-bracket depth; the outer
/// brackets stay part of the raw value (the oracle keeps them in the node
/// name after autosplit).
fn html(chars: &[char], i: usize, line: usize) -> Result<(Tok, usize, usize), super::ParseError> {
    let mut s = StrScan::at(i + 1);
    s.raw.push('<');
    s.j = i + 1;
    let mut depth = 1usize;
    while let Some(&c) = chars.get(s.j) {
        let closing = {
            s.push(c);
            if c == '<' {
                depth += 1;
            } else if c == '>' {
                depth -= 1;
            }
            depth == 0
        };
        if closing {
            return Ok((Tok::Html(s.raw), s.j, s.newlines));
        }
    }
    Err(err(line, "unterminated HTML-like string"))
}

/// An identifier: alphanumerics plus `_`, `.`, `$`, `+`.
fn ident(chars: &[char], i: usize) -> (Tok, usize) {
    let end = chars[i..].iter().take_while(|&&c| ident_char(c)).count() + i;
    (Tok::Ident(chars[i..end].iter().collect()), end)
}
