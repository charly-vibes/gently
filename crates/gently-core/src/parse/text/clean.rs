//! Purpose: the c7 comment/escape pre-pass of the ge.text_parser grammar.
//! Responsibilities: per input line, truncate at an unescaped `#` (everywhere
//! — mid-line and inside quoted values), keep `\#` escapes intact for the
//! later unquote pass, accept `#rgb`/`#rrggbb` tokens as an auto-escape when
//! they immediately follow an attribute separator, and drop lines that are
//! only a comment. Rationale: upstream `_clean_line` (Graph::Easy 0.69
//! Parser.pm) is quote-unaware for both rules, so this pass runs before any
//! structural parsing (probe: tests/repro/claims/sharp-label.observed).

/// Clean every line of `input` (c7) and keep the line structure so the
/// cursor's 1-based line numbers stay aligned.
pub fn strip_comments(input: &str) -> String {
    let lines: Vec<String> = input.split('\n').map(clean_line).collect();
    lines.join("\n")
}

/// One line's comment/escape handling (c7): truncate at the first unescaped
/// `#` unless it opens a hex colour token after an attribute separator;
/// collapse `\#` escapes afterwards; a fully commented line cleans to "".
fn clean_line(line: &str) -> String {
    let trimmed = line.strip_suffix('\r').unwrap_or(line);
    let chars: Vec<char> = trimmed.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && chars.get(i + 1) == Some(&'#') {
            out.push_str("\\#");
            i += 2;
            continue;
        }
        if c == '#' && !hex_after_separator(&chars, i) {
            break; // comment: the rest of the line is dropped
        }
        out.push(c);
        i += 1;
    }
    out.replace("\\#", "#")
}

/// True when `chars[i]` opens a 3- or 6-digit hex token (`#rgb`/`#rrggbb`)
/// that immediately follows an attribute separator — optionally separated
/// by whitespace and one opening quote (upstream `_clean_line`'s hex
/// auto-escape regex: `$sep\s*("?)#hex("?)`).
fn hex_after_separator(chars: &[char], i: usize) -> bool {
    if hex_token_len(chars, i) == 0 {
        return false;
    }
    let prefix = &chars[..i];
    let before_quote = prefix.iter().rev().find(|c| !c.is_whitespace()).copied();
    let sep = if before_quote == Some('"') {
        // one opening quote may sit between the separator and the hex
        let quote_pos = prefix.iter().rposition(|c| *c == '"');
        let before = &prefix[..quote_pos.unwrap_or(0)];
        before.iter().rev().find(|c| !c.is_whitespace()).copied()
    } else {
        before_quote
    };
    sep == Some(':')
}

/// The length of the hex digit run after the `#` at `i` — 3 or 6 exactly
/// make a token (any other run length is a comment starter, not a colour).
fn hex_token_len(chars: &[char], i: usize) -> usize {
    let run: usize = chars[i + 1..]
        .iter()
        .take_while(|c| c.is_ascii_hexdigit())
        .count();
    if run == 3 || run == 6 {
        run
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::{clean_line, strip_comments};

    #[test]
    fn comments_truncate_and_escape() {
        assert_eq!(clean_line("[ a ] # junk"), "[ a ] ");
        assert_eq!(clean_line("# all junk"), "");
        assert_eq!(clean_line("x \\# y"), "x # y");
        assert_eq!(clean_line("\\# # junk"), "# ");
    }

    #[test]
    fn hex_tokens_survive_only_after_a_separator() {
        assert_eq!(clean_line("{ color: #ff0000; }"), "{ color: #ff0000; }");
        assert_eq!(clean_line("{ color: #f00; }"), "{ color: #f00; }");
        assert_eq!(clean_line("{ color:#f00; }"), "{ color:#f00; }");
        assert_eq!(clean_line("{ color: red #f00; }"), "{ color: red ");
        assert_eq!(clean_line("{ color: #f000; }"), "{ color: ");
        assert_eq!(clean_line("[ #1 ]"), "[ ");
    }

    #[test]
    fn line_structure_is_preserved() {
        assert_eq!(strip_comments("a # c\nb"), "a \nb");
        assert_eq!(strip_comments("a\r\nb # c"), "a\nb ");
    }
}
