//! Purpose: upstream Graph::Easy::Parser content sniffing (Parser.pm ~557).
//! Responsibilities: classify a source as DOT-family, VCG, or plain text
//! by scanning line heads against the upstream header regexes.
//! Rationale: the digraph-family header regexes run first over the whole
//! text, then the `graph: {` VCG sniff; replicating the regex semantics
//! by hand keeps the byte-compat oracle intact without a regex dep.

/// Which parser upstream's text-parser sniffing redirects to (Parser.pm
/// ~557): a line-start `(strict)?(di)?graph [name] {` header swaps in the
/// Graphviz parser, `graph: {` the VCG parser.
pub enum Sniffed {
    Text,
    Dot,
    Vcg,
}

/// Upstream Graph::Easy::Parser content sniffing (Parser.pm ~557): the
/// source is scanned line-wise — the digraph-family header regexes run
/// first over the whole text, then the `graph: {` VCG sniff. A line head
/// matches the DOT family when, after leading whitespace and `/*...*/`
/// comments, it reads `strict? digraph|graph [name]? {` — where a nameless
/// `graph {` matches only under `strict` (the upstream regexes require a
/// name otherwise). Whitespace between the tokens is optional upstream.
pub fn sniff(source: &str) -> Sniffed {
    if source
        .lines()
        .any(|line| line_head_sniffs_dot(&line.to_lowercase()))
    {
        return Sniffed::Dot;
    }
    // The VCG sniff is case-sensitive upstream (no /i).
    if source.lines().any(line_head_sniffs_vcg) {
        return Sniffed::Vcg;
    }
    Sniffed::Text
}

/// Strip leading whitespace and /*...*/ comment groups from a line head
/// (upstream's $o_cmt prefix alternation).
fn strip_line_head(line: &str) -> &str {
    let mut rest = line;
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix("/*") {
            match after.find("*/") {
                Some(i) => rest = &after[i + 2..],
                None => return rest,
            }
        } else {
            return rest;
        }
    }
}

/// One lowercased line head against the upstream DOT-family header regex
/// `(strict)?$o_cmt(di)?graph$o_cmt("[^"]*"|[\w_]+)?$o_cmt\{` — with the
/// upstream nuance that a nameless `graph {` only matches under `strict`.
fn line_head_sniffs_dot(line: &str) -> bool {
    let rest = strip_line_head(line);
    let (strict, rest) = match rest.strip_prefix("strict") {
        Some(r) => (true, r.trim_start()),
        None => (false, rest),
    };
    let (has_di, rest) = if let Some(r) = rest.strip_prefix("digraph") {
        (true, r)
    } else if let Some(r) = rest.strip_prefix("graph") {
        (false, r)
    } else {
        return false;
    };
    let Some((named, rest)) = scan_graph_name(rest.trim_start()) else {
        return false;
    };
    let rest = rest.trim_start();
    if !rest.starts_with('{') {
        return false;
    }
    // A nameless, non-strict `graph {` does not match the upstream regexes.
    !(!has_di && !named && !strict)
}

/// Scan the optional graph name after `graph`/`digraph`: a quoted string or
/// a word ([\w_]+ upstream). Returns whether a name was present and the
/// remaining text, or `None` when the head cannot be a valid header
/// (unterminated quote, or a non-name non-brace token).
fn scan_graph_name(rest: &str) -> Option<(bool, &str)> {
    if let Some(after) = rest.strip_prefix('"') {
        return after.find('"').map(|i| (true, &after[i + 1..]));
    }
    if rest.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
        let n = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        return Some((true, &rest[n..]));
    }
    Some((false, rest))
}

/// One line against the upstream VCG sniff regex `^[\s\n\t]*graph\s*:\s*\{`
/// (case-sensitive — no /i on that match).
fn line_head_sniffs_vcg(line: &str) -> bool {
    let rest = line.trim_start();
    let Some(rest) = rest.strip_prefix("graph") else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix(':') else {
        return false;
    };
    rest.trim_start().starts_with('{')
}
