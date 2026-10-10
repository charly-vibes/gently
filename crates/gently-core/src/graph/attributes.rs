//! Purpose: the attribute machinery of the graph model (ge.graph_model c2,
//! gently-4ht). Responsibilities: ordered verbatim attribute tables and the
//! upstream-faithful border splitter that derives style/width/color at
//! assignment time. Rationale: kept apart from the graph structure so the
//! supported-attribute-set contract has one readable home (see mod.rs for
//! the full supported set).

/// An ordered attribute table: verbatim `(key, value)` pairs in insertion
/// order, last-set wins on duplicates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttributeTable {
    entries: Vec<(String, String)>,
}

impl AttributeTable {
    /// Store `value` verbatim under `key`; overwrites an earlier value.
    /// The key `border` additionally derives `border_style`, `border_width`
    /// and `border_color` at assignment time (upstream
    /// `split_border_attributes` semantics — see the module header).
    pub fn set(&mut self, key: &str, value: &str) {
        set_entry(&mut self.entries, key, value);
        if key == "border" {
            let (style, width, color) = split_border_attributes(value);
            set_entry(&mut self.entries, "border_style", &style);
            set_entry(&mut self.entries, "border_width", &width);
            set_entry(&mut self.entries, "border_color", &color);
        }
    }

    /// The verbatim value stored under `key`, if any.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Drop the value stored under `key` (last occurrence); returns whether
    /// anything was removed. Parser-serving surface (ge.dot_parser c5: a
    /// split node's `label` attribute becomes its name and must not linger).
    pub fn remove(&mut self, key: &str) -> bool {
        match self.entries.iter().rposition(|(k, _)| k == key) {
            Some(i) => {
                self.entries.remove(i);
                true
            }
            None => false,
        }
    }

    /// True when no attribute is stored.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All `(key, value)` entries sorted by key (keys are unique — last-set
    /// wins — so the order is total). Consumers that must emit attributes in
    /// sorted order (the txt renderer) read through this view.
    pub fn sorted_entries(&self) -> Vec<(&str, &str)> {
        let mut v: Vec<(&str, &str)> = self
            .entries
            .iter()
            .map(|(k, val)| (k.as_str(), val.as_str()))
            .collect();
        v.sort_unstable_by(|x, y| x.0.cmp(y.0));
        v
    }
}

/// Overwrite or append `(key, value)` preserving first-seen position.
fn set_entry(entries: &mut Vec<(String, String)>, key: &str, value: &str) {
    match entries.iter_mut().find(|(k, _)| k == key) {
        Some(slot) => slot.1 = value.to_string(),
        None => entries.push((key.to_string(), value.to_string())),
    }
}

/// Split a border value into `(style, width, color)` exactly as upstream
/// Graph::Easy 0.69 `split_border_attributes` does: the style is the first
/// (leftmost-position, alternation-order) occurrence of a style word, and
/// defaults to `solid`; every `\d+(px|em|%)` token is removed with the last
/// one remembered digits-only as the width; the whitespace-free remainder
/// is the color.
fn split_border_attributes(border: &str) -> (String, String, String) {
    // upstream special case: `border: 0` → none
    if border == "0" {
        return ("none".into(), String::new(), String::new());
    }
    let (mut rest, matched) = strip_first_style_word(border);
    let style = matched.unwrap_or("solid");
    // width: remove every `\d+(px|em|%)` token, remember the last, digits only
    let ranges = width_ranges(&rest);
    let mut width = String::new();
    if let Some(&(start, end)) = ranges.last() {
        width = rest[start..end].chars().filter(|c| c.is_ascii_digit()).collect();
    }
    for &(start, end) in ranges.iter().rev() {
        rest.replace_range(start..end, "");
    }
    let color: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    (style.to_string(), width, color)
}

/// The upstream style words in alternation order — leftmost position wins,
/// then this order decides.
const STYLES: [&str; 13] = [
    "solid", "dotted", "dot-dot-dash", "dot-dash", "dashed", "double-dash", "double",
    "bold-dash", "bold", "broad", "wide", "wave", "none",
];

/// Remove the first style word from `s` (leftmost position, upstream
/// alternation order — no word boundaries, exactly as upstream's regex
/// substitution does), returning the remainder and the matched style.
fn strip_first_style_word(s: &str) -> (String, Option<&'static str>) {
    for pos in 0..s.len() {
        for candidate in STYLES {
            if s[pos..].starts_with(candidate) {
                let mut rest = s.to_string();
                rest.replace_range(pos..pos + candidate.len(), "");
                return (rest, Some(candidate));
            }
        }
    }
    (s.to_string(), None)
}

/// All `\d+(px|em|%)` token ranges in `s`, left to right (upstream's global
/// width substitution: the unit is required, bare digits are left alone).
fn width_ranges(s: &str) -> Vec<(usize, usize)> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let end = if b[i].is_ascii_digit() { digit_token_end(s, i) } else { i };
        if end > i {
            out.push((i, end));
        }
        i = end.max(i + 1);
    }
    out
}

/// End of the `\d+(px|em|%)` token starting at `i`, or `i` when the digits
/// are not followed by a unit (upstream requires the unit).
fn digit_token_end(s: &str, i: usize) -> usize {
    let digits = s[i..].bytes().take_while(u8::is_ascii_digit).count();
    let after = i + digits;
    for unit in ["px", "em", "%"] {
        if s[after..].starts_with(unit) {
            return after + unit.len();
        }
    }
    i
}
