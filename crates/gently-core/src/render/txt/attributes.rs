//! Purpose: the attribute-text builders of the canonical txt serializer
//! (ge.txt_render, gently-3hv) — gently's port of the attribute-facing half
//! of Graph::Easy 0.69 `As_txt.pm`.
//! Responsibilities: render one class attribute section (`class { ... }`
//! with sorted keys, border normalized from the derived components and
//! class defaults dropped, upstream's 40-char single-line compaction), the
//! instance attribute text ` { k: v; ... }` for nodes/edges/groups, the
//! `border: X` reconstruction from border_style/border_width/border_color,
//! the class default borders, and the percent-encoding of critical
//! characters in values.
//! Rationale: kept apart from the graph-ordering logic in the parent
//! module so every upstream format decision has one readable home; the
//! recorded oracle forms (t/edge.t, live oracle captures) pin each rule
//! asserted in the tests below.

use crate::graph::{AttributeTable, ObjectKind};

/// One `class { ... }` section: sorted attributes (border components
/// collapsed into a normalized `border:` line), compacted to one line when
/// short (upstream's "short, single definitions" rule).
pub fn class_section(class: &str, table: &AttributeTable, kind: ObjectKind) -> String {
    let mut att = String::new();
    for (key, value) in table.sorted_entries() {
        if key.starts_with("border") || value.is_empty() {
            continue;
        }
        att.push_str(&format!("  {key}: {};\n", encode_value(value)));
    }
    // edges do not have a border
    if kind != ObjectKind::Edge {
        let border = border_attribute(table, kind);
        let default = default_border_str(kind);
        if !border.is_empty() && !default.starts_with(&border) {
            att.push_str(&format!("  border: {border};\n"));
        }
    }
    if att.is_empty() {
        return String::new();
    }
    // short, single definitions fit on one line
    let single = att.matches('\n').count() < 2 && att.len() < 40;
    if single {
        let one = att.replace('\n', " ");
        let one = one.replacen("  ", " ", 1);
        format!("{class} {{{one}}}\n")
    } else {
        format!("{class} {{\n{att}}}\n")
    }
}

/// The instance attribute text ` { k: v; ... }` (empty when no attributes):
/// sorted keys with border collapsed into a normalized trailing `border:`
/// entry (upstream Node::attributes_as_txt; the literal `class` key is
/// emitted last, matching the upstream subclass position).
pub fn instance_attributes(table: &AttributeTable, kind: ObjectKind) -> String {
    let mut att = String::new();
    let mut class_value = "";
    for (key, value) in table.sorted_entries() {
        if key.starts_with("border") || value.is_empty() {
            continue;
        }
        if key == "class" {
            class_value = value;
            continue;
        }
        if key == "label" && kind == ObjectKind::Edge {
            // the label is shown in the operator chain, not here
            continue;
        }
        if key == "style" && kind == ObjectKind::Edge && !style_visible_in_operator(table) {
            // the operator already shows the style
            continue;
        }
        att.push_str(&format!("{key}: {}; ", encode_value(value)));
    }
    let border = border_attribute(table, kind);
    // upstream: don't include the default border (`border == ''` when it
    // collapses to the class default, `none` for edge classes)
    if !border.is_empty() && border != default_border_str(kind) {
        att.push_str(&format!("border: {border}; "));
    }
    if !class_value.is_empty() {
        att.push_str(&format!("class: {}; ", encode_value(class_value)));
    }
    if att.is_empty() {
        String::new()
    } else {
        format!(" {{ {att}}}")
    }
}

/// Edge styles whose operator stays `--` keep a visible `style:` attribute;
/// the operator-shown styles are suppressed from the attribute text
/// (upstream Edge::_as_txt's `$suppress`).
fn style_visible_in_operator(table: &AttributeTable) -> bool {
    matches!(
        table.get("style").unwrap_or("solid"),
        "bold" | "bold-dash" | "broad" | "wide" | "invisible"
    )
}

/// The `border: X` reconstruction from the derived components (upstream
/// `border_attribute` + `_border_attribute`): default width/color dropped,
/// digits-only width gets `px`, default style for the class with no
/// width/color left is dropped entirely; `none` passes through.
pub fn border_attribute(table: &AttributeTable, kind: ObjectKind) -> String {
    let style = table.get("border_style").unwrap_or(default_border_style(kind));
    if style == "none" {
        return "none".to_string();
    }
    let mut width = table.get("border_width").unwrap_or("");
    if width == "1" {
        width = "";
    }
    let mut color = table.get("border_color").unwrap_or("");
    if color == "#000000" {
        color = "";
    }
    if default_border_style(kind) == style && width.is_empty() && color.is_empty() {
        return String::new();
    }
    join_border(style, width, color)
}

fn join_border(style: &str, width: &str, color: &str) -> String {
    let width = if !width.is_empty() && width.bytes().all(|b| b.is_ascii_digit()) {
        format!("{width}px")
    } else {
        width.to_string()
    };
    [style, &width, color]
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The class default border style (upstream `borderstyle` defaults:
/// node `solid`, group `dashed`, everything else `none`).
fn default_border_style(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Node => "solid",
        ObjectKind::Group => "dashed",
        ObjectKind::Edge => "none",
    }
}

/// The class default border string (upstream `border` attribute defaults).
pub fn default_border_str(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Node => "solid 1px #000000",
        ObjectKind::Group => "dashed 1px #000000",
        ObjectKind::Edge => "none",
    }
}

/// Percent-encode the critical characters (`;`, `"`, `%` and control
/// characters) — but only when the value actually contains anything other
/// than plain text, so `rgb(10%,0,0)` stays as it is (upstream
/// `_remap_attributes` with 'encode').
pub fn encode_value(value: &str) -> String {
    let critical = value.bytes().any(|b| b == b';' || b == b'"' || b < 0x20);
    if !critical {
        return value.to_string();
    }
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b == b';' || b == b'"' || b == b'%' || b < 0x20 {
            out.push_str(&format!("%{b:02x}"));
        } else {
            out.push(b as char);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Graph, Scope};

    /// Oracle: a class border that is a prefix of (or equal to) the class
    /// default is suppressed entirely — the section vanishes.
    #[test]
    fn class_border_default_prefix_is_suppressed() {
        let mut g = Graph::default();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.add_edge(a, b, true);
        g.set_attr(Scope::Class(ObjectKind::Node, String::new()), "border", "solid");
        assert_eq!(super::super::render(&g), "[ a ] --> [ b ]\n");
    }

    /// Oracle: values containing `;`, `"` or control characters are
    /// percent-encoded (lowercase hex); `rgb(10%,0,0)` stays as it is.
    #[test]
    fn values_are_percent_encoded_only_when_critical() {
        assert_eq!(encode_value("rgb(10%,0,0)"), "rgb(10%,0,0)");
        assert_eq!(encode_value("a;b"), "a%3bb");
        assert_eq!(encode_value("say \"hi\""), "say %22hi%22");
    }

    /// Oracle: `border: blue solid 2px` on the node class normalizes to
    /// `border: solid 2px blue;` with the default components dropped.
    #[test]
    fn border_normalizes_style_width_color() {
        let mut g = Graph::default();
        g.set_attr(
            Scope::Class(ObjectKind::Node, String::new()),
            "border",
            "blue solid 2px",
        );
        assert_eq!(
            super::super::render(&g),
            "node { border: solid 2px blue; }\n\n"
        );
    }
}