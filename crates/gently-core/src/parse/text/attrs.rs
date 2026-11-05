//! Purpose: attribute-block parsing for ge.text_parser (c5, c10).
//! Responsibilities: read a `{ key: value; … }` block's content (already
//! comment-cleaned), split it into declarations on `;` (mirroring the
//! upstream `_match_single_attribute` alternation: a leading-`"` value is
//! captured as a quoted string only when its unescaped closing `"` is
//! immediately followed by `;` or the end of the block — otherwise the
//! declaration runs to the first `;` not preceded by a backslash), split
//! each on its first `:`, and finalize values with the upstream TWO-LAYER
//! unquoting (c10): parser layer (`_unquote` — unescape the special set,
//! collapse whitespace), then store layer (`Attributes::unquote_attribute`
//! — strip one greedy pair of surrounding quotes, either quote char,
//! then unescape `\# \" \' \; \\`, strip exploit-band %XX escapes, and
//! decode printable-band %XX entities). Also
//! owns the shared unescape/collapse helpers used for node and group
//! names and edge labels (parser layer only — names never see the store
//! layer). Rationale: values are stored verbatim — no key or value
//! validation here; the model accepts unknown keys by contract
//! (ge.graph_model supported-attribute-set).

use super::ParseError;

/// Parse the block content between a `{` at the cursor and its closing
/// `}` into `(key, value)` pairs (c5). Line numbers come from the cursor.
pub fn parse_block(p: &mut super::Parser) -> Result<Vec<(String, String)>, ParseError> {
    let start_line = p.line();
    p.bump(); // the '{' the caller peeked
    let content = scan_to_close(p)
        .ok_or_else(|| p.err(start_line, "unterminated attribute block: missing '}'"))?;
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

/// Split the content into declarations, mirroring the upstream
/// `_match_single_attribute` alternation: a value that starts with `"` is
/// captured as a quoted string only when its unescaped closing `"` is
/// immediately followed by `;` or the end of the content — otherwise (and
/// whenever the value doesn't start with `"`) the declaration runs to the
/// first `;` not preceded by a backslash. A `\;` never splits.
fn split_declarations(content: &str) -> Vec<String> {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= n {
            break;
        }
        let start = i;
        // the name runs to the first `:` (`[^:]+?` upstream); without one
        // the rest is a single (invalid) declaration that parse_single
        // rejects, mirroring the upstream regex failure
        let end = match (i..n).find(|&k| chars[k] == ':') {
            None => {
                out.push(chars[start..n].iter().collect());
                break;
            }
            Some(c) => declaration_end(&chars, c),
        };
        out.push(chars[start..end].iter().collect());
        i = end + 1; // step past the `;` (or clamp at n for the final decl)
    }
    out
}

/// Exclusive end of one declaration whose `:` sits at `colon`: from the
/// value start, the quoted branch when the value opens with `"` (with
/// the unquoted branch as its backtrack), the unquoted branch otherwise.
fn declaration_end(chars: &[char], colon: usize) -> usize {
    let n = chars.len();
    let v = colon
        + 1
        + chars[colon + 1..]
            .iter()
            .take_while(|&&c| c.is_whitespace())
            .count();
    if v < n && chars[v] == '"' {
        quoted_branch_end(chars, v).unwrap_or_else(|| unquoted_branch_end(chars, v))
    } else {
        unquoted_branch_end(chars, v)
    }
}

/// End of the quoted branch: the value from `open` (a `"`) through its
/// unescaped closing `"`, when that close is immediately followed (after
/// whitespace) by `;` or the end of the content. Returns the index of the
/// `;` (exclusive end of the declaration) or `n`; `None` when the closing
/// quote never comes or the terminator doesn't follow it — upstream
/// backtracks to the unquoted branch in exactly that case.
fn quoted_branch_end(chars: &[char], open: usize) -> Option<usize> {
    let n = chars.len();
    let close = (open + 1..n).find(|&k| chars[k] == '"' && chars[k - 1] != '\\')?;
    let m = close
        + 1
        + chars[close + 1..]
            .iter()
            .take_while(|&&c| c.is_whitespace())
            .count();
    if m == n {
        Some(n) // `\s*\z` terminator
    } else if chars[m] == ';' {
        Some(m) // `\s*;\s*` terminator; declaration ends at the `;`
    } else {
        None // close not followed by a terminator: backtrack
    }
}

/// End of the unquoted branch: the first `;` not preceded by a backslash
/// (upstream pairs every `\;` as a unit, so the alignment is always at
/// the last backslash — `\\;` does not split either), or the end of the
/// content. Returns the index of the `;` (exclusive) or `n`.
fn unquoted_branch_end(chars: &[char], from: usize) -> usize {
    (from..chars.len())
        .find(|&k| chars[k] == ';' && (k == from || chars[k - 1] != '\\'))
        .unwrap_or(chars.len())
}

/// One `name : value` declaration; the first `:` separates.
fn parse_single(decl: &str, line: usize) -> Result<(String, String), ParseError> {
    let err = |m: String| ParseError { line, message: m };
    let (name, value) = decl.split_once(':').ok_or_else(|| {
        err(format!(
            "attribute '{decl}' doesn't look valid: missing ':'"
        ))
    })?;
    let name = name.trim();
    if name.is_empty() {
        return Err(err("empty attribute name".to_string()));
    }
    let value = unquote_value(value.trim());
    Ok((name.to_string(), value))
}

/// Finalize the value with the upstream TWO-LAYER unquoting (c10): the
/// parser layer (`_unquote`: unescape the special set, collapse
/// whitespace), then the store layer (`Attributes::unquote_attribute`:
/// strip one greedy pair of surrounding quotes — either quote char,
/// mixed ends allowed — and unescape `\# \" \' \; \\`, strip exploit-band
/// %XX escapes, decode printable-band %XX entities). A value whose
/// opening quote never closes is kept verbatim; mid-value quotes survive.
fn unquote_value(value: &str) -> String {
    let parsed = collapse_ws(&unescape(value));
    let unescaped = store_unescape(&strip_quote_pair(&parsed));
    decode_percent_entities(&unescaped)
}

/// The store-layer quote strip: `s/^["'](.*)["']\z/$1/` — when the first
/// and last characters are both quote chars (either one, mixed ends
/// allowed), both go; a lone quote char (length 1) stays.
fn strip_quote_pair(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let quoted = |c: char| c == '"' || c == '\'';
    if chars.len() >= 2 && quoted(chars[0]) && quoted(chars[chars.len() - 1]) {
        chars[1..chars.len() - 1].iter().collect()
    } else {
        s.to_string()
    }
}

/// The store-layer unescape: `\X` → X for X in `# " ' ; \` — upstream
/// `s/\\([#"';\\])/$1/g`. Other backslashes stay literal.
fn store_unescape(s: &str) -> String {
    rewrite(s, |chars, i| {
        if chars[i] == '\\'
            && i + 1 < chars.len()
            && matches!(chars[i + 1], '#' | '"' | '\'' | ';' | '\\')
        {
            (chars[i + 1].to_string(), 2)
        } else {
            (chars[i].to_string(), 1)
        }
    })
}

/// Walk `s` char by char; at each index `step` returns the replacement
/// text and how many source chars it consumed (≥ 1). Shared walker for
/// the escape/percent layers.
fn rewrite(s: &str, step: impl Fn(&[char], usize) -> (String, usize)) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let (chunk, adv) = step(&chars, i);
        out.push_str(&chunk);
        i += adv;
    }
    out
}

/// Unquote escapes for the upstream special-character set: `\X` → X for X
/// in `[ ( { } ] ) # < > - . =`; other backslashes stay literal.
/// Unquote escapes for the upstream special-character set: `\X` → X for X
/// in `[ ( { } ] ) # < > - . =`; other backslashes stay literal (upstream
/// `s/\\([\[\(\{\}\]\)#<>\-\.=])/$1/g` — each backslash is judged on
/// its own successor, so `\\q` stays two literal backslashes + q).
pub fn unescape(s: &str) -> String {
    let special = |c: char| {
        matches!(
            c,
            '[' | '(' | '{' | '}' | ']' | ')' | '#' | '<' | '>' | '-' | '.' | '='
        )
    };
    rewrite(s, |chars, i| {
        if chars[i] == '\\' && i + 1 < chars.len() && special(chars[i + 1]) {
            (chars[i + 1].to_string(), 2)
        } else {
            (chars[i].to_string(), 1)
        }
    })
}

/// The store-layer %XX pass (upstream `s/%[^2-7][a-fA-F0-9]|%7f//g` then
/// `s/%([2-7][a-fA-F0-9])/sprintf("%c",hex($1))/eg`): per position, a `%`
/// followed by a non-%2X-%7X head and a hex digit — the exploit band
/// (%00-%1f, %80-%ff) plus `%7f` — is dropped, a `%` in the %20-%7f band
/// followed by a hex digit decodes to its character, and anything else
/// stays literal (e.g. `100%`, `%zz`).
fn decode_percent_entities(s: &str) -> String {
    let hex = |c: char| c.is_ascii_hexdigit();
    let exploit = |head: char, tail: char| {
        hex(tail) && (!(('2'..='7').contains(&head)) || (head == '7' && tail == 'f'))
    };
    rewrite(s, |chars, i| {
        if chars[i] == '%' && i + 2 < chars.len() {
            let (head, tail) = (chars[i + 1], chars[i + 2]);
            if exploit(head, tail) {
                return (String::new(), 3); // drop the escape entirely
            }
            if hex(tail) && ('2'..='7').contains(&head) {
                let v = head.to_digit(16).unwrap() * 16 + tail.to_digit(16).unwrap();
                let c = char::from_u32(v).expect("%20-%7f is a valid scalar");
                return (c.to_string(), 3);
            }
        }
        (chars[i].to_string(), 1)
    })
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
