//! Purpose: attribute-block parsing for ge.text_parser (c5).
//! Responsibilities: read a `{ key: value; … }` block's content (already
//! comment-cleaned), split it into declarations on unquoted/unescaped `;`,
//! split each on its `:`, and unquote values — quoted values lose their
//! quotes, all values are unescaped and whitespace-collapsed verbatim
//! (upstream `_match_single_attribute` + `_unquote` semantics). Also owns
//! the shared unescape/collapse helpers used for node and group names and
//! edge labels. Rationale: values are stored verbatim — no key or value
//! validation here; the model accepts unknown keys by contract
//! (ge.graph_model supported-attribute-set).

use super::ParseError;

/// Parse the block content between a `{` at the cursor and its closing
/// `}` into `(key, value)` pairs (c5). Line numbers come from the cursor.
pub fn parse_block(p: &mut super::Parser) -> Result<Vec<(String, String)>, ParseError> {
    let start_line = p.line();
    p.bump(); // the '{' the caller peeked
    let content = scan_to_close(p).ok_or_else(|| p.err(start_line, "unterminated attribute block: missing '}'"))?;
    parse_declarations(&content, start_line)
}

/// Scan chars until the closing `}`, returning the raw content.
fn scan_to_close(p: &mut super::Parser) -> Option<String> {
    let mut content = String::new();
    loop {
        match p.peek() {
            None => return None,
            Some('}') => {
                p.bump();
                return Some(content);
            }
            Some(c) => {
                content.push(c);
                p.bump();
            }
        }
    }
}

/// Split the content on `;` (not inside quotes, not escaped as `\;`) and
/// parse each non-empty declaration.
fn parse_declarations(content: &str, line: usize) -> Result<Vec<(String, String)>, ParseError> {
    let mut out = Vec::new();
    for decl in split_declarations(content) {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        out.push(parse_single(decl, line)?);
    }
    Ok(out)
}

/// Split on `;` honoring quotes and backslash escapes.
fn split_declarations(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut escaped = false;
    for c in content.chars() {
        if escaped {
            cur.push(c);
            escaped = false;
        } else if c == '\\' {
            cur.push(c);
            escaped = true;
        } else if c == '"' {
            in_quote = !in_quote;
            cur.push(c);
        } else if c == ';' && !in_quote {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

/// One `name : value` declaration; the first `:` separates.
fn parse_single(decl: &str, line: usize) -> Result<(String, String), ParseError> {
    let err = |m: String| ParseError { line, message: m };
    let (name, value) = decl
        .split_once(':')
        .ok_or_else(|| err(format!("attribute '{decl}' doesn't look valid: missing ':'")))?;
    let name = name.trim();
    if name.is_empty() {
        return Err(err("empty attribute name".to_string()));
    }
    let value = unquote_value(value.trim(), line)?;
    Ok((name.to_string(), value))
}

/// The value: strip one pair of quotes when present, then unescape and
/// collapse whitespace (upstream `_unquote`).
fn unquote_value(value: &str, line: usize) -> Result<String, ParseError> {
    let unquoted = match value.starts_with('"') {
        true => value
            .strip_suffix('"')
            .and_then(|v| v.strip_prefix('"'))
            .ok_or_else(|| ParseError { line, message: "mismatched quotes in attribute value".to_string() })?,
        false => value,
    };
    Ok(collapse_ws(&unescape(unquoted)))
}

/// Unquote escapes for the upstream special-character set: `\X` → X for X
/// in `[ ( { } ] ) # < > - . =`; other backslashes stay literal.
pub fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for c in s.chars() {
        if escaped && matches!(c, '[' | '(' | '{' | '}' | ']' | ')' | '#' | '<' | '>' | '-' | '.' | '=') {
            out.push(c);
        } else {
            if escaped {
                out.push('\\');
            }
            out.push(c);
        }
        escaped = c == '\\' && !escaped;
    }
    if escaped {
        out.push('\\');
    }
    out
}

/// Collapse whitespace runs to single spaces (upstream `_unquote`).
pub fn collapse_ws(s: &str) -> String {
    let mut out = String::new();
    let mut in_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            in_ws = !out.is_empty();
        } else {
            if in_ws {
                out.push(' ');
            }
            in_ws = false;
            out.push(c);
        }
    }
    out
}
