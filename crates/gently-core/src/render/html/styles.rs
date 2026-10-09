//! Purpose: the attribute remap tables of the HTML renderer
//! (ge.html_render c2/c3/c5) — the W3C color-name scheme, the
//! border-to-CSS converter, and the text/href escaping rules.
//! Responsibilities: convert color attribute values to hex exactly as
//! upstream Graph::Easy 0.69 `color_as_hex` (lib/Graph/Easy/Attributes.pm,
//! the `w3c` scheme: named colors to #rrggbb, rgb(r,g,b) to hex, hex
//! passthrough); compose `border_attribute_as_html(style, width, color)`
//! exactly as upstream (bold→solid 4px, wide→solid 1em, broad→solid
//! 0.5em, bold-dash→dashed, double-dash→double, width dropped for
//! double*); escape label text (&, >, <) and href values (space→+,
//! '→%27 — the observed raw `&` in href). Rationale: these are the
//! probed pinned-oracle tables (tests/repro/claims/html-*.observed).

/// The upstream W3C color-name scheme (lib/Graph/Easy/Attributes.pm,
/// scheme `w3c`): color name → #rrggbb, verbatim, sorted by name.
const W3C_COLORS: &[(&str, &str)] = &[
        ("aliceblue", "#f0f8ff"),
        ("antiquewhite", "#faebd7"),
        ("aqua", "#00ffff"),
        ("aquamarine", "#7fffd4"),
        ("azure", "#f0ffff"),
        ("beige", "#f5f5dc"),
        ("bisque", "#ffe4c4"),
        ("black", "#000000"),
        ("blanchedalmond", "#ffebcd"),
        ("blue", "#0000ff"),
        ("blueviolet", "#8a2be2"),
        ("brown", "#a52a2a"),
        ("burlywood", "#deb887"),
        ("cadetblue", "#5f9ea0"),
        ("chartreuse", "#7fff00"),
        ("chocolate", "#d2691e"),
        ("coral", "#ff7f50"),
        ("cornflowerblue", "#6495ed"),
        ("cornsilk", "#fff8dc"),
        ("crimson", "#dc143c"),
        ("cyan", "#00ffff"),
        ("darkblue", "#00008b"),
        ("darkcyan", "#008b8b"),
        ("darkgoldenrod", "#b8860b"),
        ("darkgray", "#a9a9a9"),
        ("darkgreen", "#006400"),
        ("darkgrey", "#a9a9a9"),
        ("darkkhaki", "#bdb76b"),
        ("darkmagenta", "#8b008b"),
        ("darkolivegreen", "#556b2f"),
        ("darkorange", "#ff8c00"),
        ("darkorchid", "#9932cc"),
        ("darkred", "#8b0000"),
        ("darksalmon", "#e9967a"),
        ("darkseagreen", "#8fbc8f"),
        ("darkslateblue", "#483d8b"),
        ("darkslategray", "#2f4f4f"),
        ("darkslategrey", "#2f4f4f"),
        ("darkturquoise", "#00ced1"),
        ("darkviolet", "#9400d3"),
        ("deeppink", "#ff1493"),
        ("deepskyblue", "#00bfff"),
        ("dimgray", "#696969"),
        ("dodgerblue", "#1e90ff"),
        ("firebrick", "#b22222"),
        ("floralwhite", "#fffaf0"),
        ("forestgreen", "#228b22"),
        ("fuchsia", "#ff00ff"),
        ("gainsboro", "#dcdcdc"),
        ("ghostwhite", "#f8f8ff"),
        ("gold", "#ffd700"),
        ("goldenrod", "#daa520"),
        ("gray", "#808080"),
        ("green", "#008000"),
        ("greenyellow", "#adff2f"),
        ("grey", "#808080"),
        ("honeydew", "#f0fff0"),
        ("hotpink", "#ff69b4"),
        ("indianred", "#cd5c5c"),
        ("indigo", "#4b0082"),
        ("inherit", "inherit"),
        ("ivory", "#fffff0"),
        ("khaki", "#f0e68c"),
        ("lavender", "#e6e6fa"),
        ("lavenderblush", "#fff0f5"),
        ("lawngreen", "#7cfc00"),
        ("lemonchiffon", "#fffacd"),
        ("lightblue", "#add8e6"),
        ("lightcoral", "#f08080"),
        ("lightcyan", "#e0ffff"),
        ("lightgoldenrodyellow", "#fafad2"),
        ("lightgray", "#d3d3d3"),
        ("lightgreen", "#90ee90"),
        ("lightgrey", "#d3d3d3"),
        ("lightpink", "#ffb6c1"),
        ("lightsalmon", "#ffa07a"),
        ("lightseagreen", "#20b2aa"),
        ("lightskyblue", "#87cefa"),
        ("lightslategray", "#778899"),
        ("lightslategrey", "#778899"),
        ("lightsteelblue", "#b0c4de"),
        ("lightyellow", "#ffffe0"),
        ("lime", "#00ff00"),
        ("limegreen", "#32cd32"),
        ("linen", "#faf0e6"),
        ("magenta", "#ff00ff"),
        ("maroon", "#800000"),
        ("mediumaquamarine", "#66cdaa"),
        ("mediumblue", "#0000cd"),
        ("mediumorchid", "#ba55d3"),
        ("mediumpurple", "#9370db"),
        ("mediumseagreen", "#3cb371"),
        ("mediumslateblue", "#7b68ee"),
        ("mediumspringgreen", "#00fa9a"),
        ("mediumturquoise", "#48d1cc"),
        ("mediumvioletred", "#c71585"),
        ("midnightblue", "#191970"),
        ("mintcream", "#f5fffa"),
        ("mistyrose", "#ffe4e1"),
        ("moccasin", "#ffe4b5"),
        ("navajowhite", "#ffdead"),
        ("navy", "#000080"),
        ("oldlace", "#fdf5e6"),
        ("olive", "#808000"),
        ("olivedrab", "#6b8e23"),
        ("orange", "#ffa500"),
        ("orangered", "#ff4500"),
        ("orchid", "#da70d6"),
        ("palegoldenrod", "#eee8aa"),
        ("palegreen", "#98fb98"),
        ("paleturquoise", "#afeeee"),
        ("palevioletred", "#db7093"),
        ("papayawhip", "#ffefd5"),
        ("peachpuff", "#ffdab9"),
        ("peru", "#cd853f"),
        ("pink", "#ffc0cb"),
        ("plum", "#dda0dd"),
        ("powderblue", "#b0e0e6"),
        ("purple", "#800080"),
        ("red", "#ff0000"),
        ("rosybrown", "#bc8f8f"),
        ("royalblue", "#4169e1"),
        ("saddlebrown", "#8b4513"),
        ("salmon", "#fa8072"),
        ("sandybrown", "#f4a460"),
        ("seagreen", "#2e8b57"),
        ("seashell", "#fff5ee"),
        ("sienna", "#a0522d"),
        ("silver", "#c0c0c0"),
        ("skyblue", "#87ceeb"),
        ("slateblue", "#6a5acd"),
        ("slategray", "#708090"),
        ("slategrey", "#708090"),
        ("snow", "#fffafa"),
        ("springgreen", "#00ff7f"),
        ("steelblue", "#4682b4"),
        ("tan", "#d2b48c"),
        ("teal", "#008080"),
        ("thistle", "#d8bfd8"),
        ("tomato", "#ff6347"),
        ("turquoise", "#40e0d0"),
        ("violet", "#ee82ee"),
        ("wheat", "#f5deb3"),
        ("white", "#ffffff"),
        ("whitesmoke", "#f5f5f5"),
        ("yellow", "#ffff00"),
        ("yellowgreen", "#9acd32"),
];

/// The hex value of a W3C color name, if the name is in the scheme.
fn w3c_color(name: &str) -> Option<&'static str> {
    W3C_COLORS.iter().find(|(n, _)| *n == name).map(|(_, hex)| *hex)
}

/// The value of a color attribute as a CSS hex color: W3C names convert
/// (case-insensitive), `rgb(r,g,b)` converts, `#hex` passes through,
/// anything else passes through verbatim (the oracle rejects unknown
/// names at attribute-assignment time — gently's model stores values
/// verbatim, so the renderer passes them on; divergence documented).
pub(in crate::render) fn color_as_hex(color: &str) -> String {
    let color = color.trim();
    let lower = color.to_lowercase();
    if let Some(hex) = w3c_color(&lower) {
        return hex.to_string();
    }
    if let Some(hex) = rgb_to_hex(&lower) {
        return hex;
    }
    color.to_string()
}

/// `rgb(r,g,b)` → `#rrggbb` (upstream accepts %, floats and hsl/hsv
/// forms; gently carries the plain integer rgb form).
fn rgb_to_hex(color: &str) -> Option<String> {
    let inner = color.strip_prefix("rgb(")?.strip_suffix(')')?;
    let mut parts = inner.split(',');
    let mut out = String::from("#");
    for part in (&mut parts).take(3) {
        let v: u32 = part.trim().parse().ok()?;
        out.push_str(&format!("{:02x}", v.min(255)));
    }
    parts.next().is_none().then_some(out)
}

/// The border style→(css style, css width) remap — the width-carrying
/// styles upstream rewrites (bold-dash before bold: the dash form keeps
/// its dashes; the 4px/1em/0.5em units are the observed bytes).
const BORDER_REMAPS: &[(&str, &str, &str)] = &[
    ("bold-dash", "dashed", "4px"),
    ("double-dash", "double", ""),
    ("bold", "solid", "4px"),
    ("wide", "solid", "1em"),
    ("broad", "solid", "0.5em"),
];

/// The border attribute as a CSS value, exactly as upstream
/// `_border_attribute_as_html` (lib/Graph/Easy/Attributes.pm): `solid 1px
/// #000000` from the border(style|width|color) components.
pub(in crate::render) fn border_attribute_as_html(
    style: &str,
    width: &str,
    color: &str,
) -> String {
    let mut style = style.trim().to_string();
    if style == "none" || style.is_empty() {
        return style;
    }
    let hex = if color.starts_with('#') {
        color.to_lowercase()
    } else {
        color_as_hex(color)
    };
    // width: 2px for double would collapse to one line
    let mut width = width.trim().to_string();
    if style.starts_with("double") {
        width = String::new();
    }
    if let Some((_, css_style, css_width)) = BORDER_REMAPS.iter().find(|(s, _, _)| **s == style) {
        style = (*css_style).to_string();
        if !css_width.is_empty() {
            width = (*css_width).to_string();
        }
    }
    if !width.is_empty() && width.chars().all(|c| c.is_ascii_digit()) {
        width.push_str("px");
    }
    if width.is_empty() && style != "double" {
        return String::new();
    }
    // upstream collapses runs of whitespace in the composed border (and
    // in the whole style attribute) before emission
    let val = format!("{style} {width} {hex}");
    val.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Escape label text for HTML: `&` first, then `>`, then `<` (upstream
/// `_label_as_html` order).
pub(in crate::render) fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('>', "&gt;").replace('<', "&lt;")
}

/// Escape a href value as upstream does: space → +, ' → %27 — the
/// ampersand stays RAW (observed href-escaping bytes; upstream does not
/// escape it in the href attribute).
pub(in crate::render) fn escape_href(link: &str) -> String {
    link.replace(' ', "+").replace('\'', "%27")
}

#[cfg(test)]
mod tests {
    use super::{border_attribute_as_html, color_as_hex, escape_href, escape_text};

    #[test]
    fn w3c_names_convert_to_hex() {
        assert_eq!(color_as_hex("red"), "#ff0000");
        assert_eq!(color_as_hex("BLUE"), "#0000ff");
        assert_eq!(color_as_hex("#112233"), "#112233");
        assert_eq!(color_as_hex("rgb(10,20,30)"), "#0a141e");
        assert_eq!(color_as_hex("notacolor"), "notacolor");
    }

    #[test]
    fn border_values_compose_upstream() {
        assert_eq!(border_attribute_as_html("solid", "1", "#000000"), "solid 1px #000000");
        assert_eq!(border_attribute_as_html("bold", "", "#000000"), "solid 4px #000000");
        assert_eq!(border_attribute_as_html("wide", "", "#000000"), "solid 1em #000000");
        assert_eq!(border_attribute_as_html("broad", "", "#000000"), "solid 0.5em #000000");
        assert_eq!(border_attribute_as_html("double", "", "#000000"), "double #000000");
        assert_eq!(border_attribute_as_html("double-dash", "", "#000000"), "double #000000");
        assert_eq!(border_attribute_as_html("bold-dash", "", "#000000"), "dashed 4px #000000");
        assert_eq!(border_attribute_as_html("solid", "2", "red"), "solid 2px #ff0000");
        assert_eq!(border_attribute_as_html("none", "1", "#000000"), "none");
    }

    #[test]
    fn escaping_matches_upstream() {
        assert_eq!(escape_text("a&b<c>"), "a&amp;b&lt;c&gt;");
        assert_eq!(escape_href("http://x/?a=1&b=2"), "http://x/?a=1&b=2");
        assert_eq!(escape_href("http://x/a b'c"), "http://x/a+b%27c");
    }
}
