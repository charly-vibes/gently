//! ge.html_render (gently-eyo): the table-based HTML renderer contract —
//! one test per property row of specs/ge-html_render.md. Expected bytes
//! are the probed pinned oracle (Graph::Easy v0.69 @ ededa3d7,
//! tests/repro/claims/html-*.observed et al), adapted to gently's
//! one-cell-per-node layout grid (the oracle's subcell machinery has no
//! counterpart — see render/html).

use gently_core::graph::{Graph, Scope};
use gently_core::render::html;
use gently_core::{layout, parse::text};

#[path = "ge_html_render/asserts.rs"]
mod asserts;
use asserts::*;

fn parse(src: &str) -> Graph {
    text::parse(src).expect("fixture must parse")
}

fn render(g: &Graph) -> String {
    html::render(g, &layout::layout(g)).expect("must render")
}

fn document(g: &Graph) -> String {
    html::document(g, &layout::layout(g)).expect("must render")
}

/// The tds of one table row, by column index (exact td bytes).
fn row_tds(table: &str, row: usize) -> Vec<String> {
    let marker = format!("<!-- row {row} line 0 -->\n<tr>\n");
    let rest = table
        .split(marker.as_str())
        .nth(1)
        .unwrap_or_else(|| panic!("no row {row}"));
    let row_html = rest.split("</tr>").next().unwrap_or(rest);
    row_html
        .lines()
        .filter(|l| l.starts_with(" <td"))
        .map(|l| l[1..].to_string())
        .collect()
}

/// All td bytes of the table, in grid order.
fn all_tds(table: &str) -> Vec<String> {
    let mut tds = Vec::new();
    let mut row = 0;
    while table.contains(&format!("<!-- row {row} line 0 -->")) {
        tds.extend(row_tds(table, row));
        row += 1;
    }
    tds
}

/// A one-node graph via the model API — for the values the text parser
/// does not deliver (newline labels, quoted URLs; upstream strips
/// attribute-value quotes, gently-bzx tracks that divergence).
fn node_attrs(name: &str, attrs: &[(&str, &str)]) -> Graph {
    let mut g = Graph::default();
    let n = g.add_node(name);
    for (k, v) in attrs {
        g.set_attr(Scope::Node(n), k, v);
    }
    g
}

/// p1 (c1): every grid cell maps to exactly one td — td count equals the
/// grid size, the skeleton carries the observed oracle attributes, and
/// each td carries the right content-kind class (node / edge / empty).
#[test]
fn p1() {
    // the tracer table: exact skeleton + content classes
    let g = parse("[ a ] --> [ b ]\n");
    assert_eq!(
        render(&g),
        "\n\n<table class=\"graph\" cellpadding=0 cellspacing=0>\n\
         <!-- row 0 line 0 -->\n<tr>\n\
         \x20<td class='node'>a</td>\n\
         \x20<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;color: #000000;\"><span class=\"sh\">></span></td>\n\
         \x20<td class='node'>b</td>\n\
         </tr>\n\n</table>\n"
    );

    // the empty graph: the skeleton, no rows
    assert_eq!(
        render(&Graph::default()),
        "\n\n<table class=\"graph\" cellpadding=0 cellspacing=0>\n</table>\n"
    );

    // td count equals grid size (w=5 h=1)
    assert_eq!(
        render(&parse("[ a ] --> [ b ] --> [ c ]\n"))
            .matches("<td")
            .count(),
        5
    );

    // the bend cycle: w=3 h=2 — corners (eb), horizontals (lh)
    let table = render(&parse("[ a ] --> [ b ] --> [ a ]\n"));
    assert_eq!(table.matches("<td").count(), 6, "cycle: w=3 h=2");
    assert_eq!(row_tds(&table, 0), vec![
        "<td class=\"edge eb\" style=\"border-bottom: solid 2px #000000; border-left: solid 2px #000000;color: #000000;\"><span class=\"sv\">∨</span></td>".to_string(),
        "<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;\">&nbsp;</td>".to_string(),
        "<td class=\"edge eb\" style=\"border-bottom: solid 2px #000000; border-left: solid 2px #000000;\">&nbsp;</td>".to_string(),
    ]);
    assert_eq!(row_tds(&table, 1), vec![
        "<td class='node'>a</td>".to_string(),
        "<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;color: #000000;\"><span class=\"sh\">></span></td>".to_string(),
        "<td class='node'>b</td>".to_string(),
    ]);
}

/// p2 (c2): node cells carry class + label; a `link` attribute becomes an
/// `a href` wrapping the label; label text is HTML-escaped and the href
/// follows the observed escaping rule (space→+, '→%27, raw `&`).
#[test]
fn p2() {
    // label text escapes &, <, > (upstream _label_as_html order)
    let table = render(&parse("[ a&b<c> ]\n"));
    assert!(
        table.contains("<td class='node'>a&amp;b&lt;c&gt;</td>"),
        "{table}"
    );

    // the observed href-escaping bytes: raw & in the href (built
    // through the model API — href-escaping.observed)
    let table = render(&node_attrs("A", &[("link", "http://x/?a=1&b=2")]));
    assert!(
        table.contains("<td class='node'><a href='http://x/?a=1&b=2'>A</a></td>"),
        "{table}"
    );

    // href encoding: space → +, ' → %27; multiline labels join with
    // <br> (observed td-colspan bytes); escaped labels; empty labels are
    // not linkable (upstream)
    let table = render(&node_attrs("A", &[("link", "http://x/a b'c")]));
    assert!(
        table.contains("<a href='http://x/a+b%27c'>A</a>"),
        "{table}"
    );
    let table = render(&node_attrs("x", &[("label", "A\nB")]));
    assert!(table.contains("<td class='node'>A<br>B</td>"), "{table}");
    let table = render(&node_attrs("x", &[("link", "u"), ("label", "x&y")]));
    assert!(
        table.contains("<td class='node'><a href='u'>x&amp;y</a></td>"),
        "{table}"
    );
    let table = render(&node_attrs("", &[("link", "u")]));
    assert!(
        table.contains("<td class='node'></td>") && !table.contains("<a href"),
        "{table}"
    );
}

/// p3 (c3): edge styles map to the documented border-image CSS values
/// (the observed remap) on the edge cells.
#[test]
fn p3() {
    assert_edge_style_remap();
    assert_edge_cells();
    assert!(
        !all_tds(&render(&parse("[ a ] --> [ b ]\n"))).is_empty(),
        "tracer renders"
    );
}

/// p4 (c4): the emitted document embeds the CSS rules for every class it
/// uses — the document is self-contained (upstream `css()` shape).
#[test]
fn p4() {
    assert!(
        !document(&Graph::default()).is_empty(),
        "document is emitted"
    );
    assert_document_wrapper();
    assert_css_rule_coverage();
}

/// p5 (c5): `fill`/`background`/`color` map to CSS color declarations via
/// the upstream W3C color-name scheme, and `shape` maps to its documented
/// CSS classes (observed bytes, without the colspan/rowspan machinery).
#[test]
fn p5() {
    assert!(
        !render(&Graph::default()).contains("style="),
        "no styles on empty graphs"
    );
    assert_color_declarations();
    assert_shape_classes();
    assert_border_components();
}

/// The W3C color-name scheme: named → hex, hex passthrough, rgb()
/// conversion, sorted combined declarations, `background` suppression.
fn assert_color_declarations() {
    for (name, hex) in [
        ("red", "#ff0000"),
        ("blue", "#0000ff"),
        ("white", "#ffffff"),
        ("black", "#000000"),
        ("lime", "#00ff00"),
        ("fuchsia", "#ff00ff"),
        ("aqua", "#00ffff"),
        ("silver", "#c0c0c0"),
        ("gray", "#808080"),
        ("olive", "#808000"),
        ("purple", "#800080"),
        ("teal", "#008080"),
        ("navy", "#000080"),
    ] {
        let g = parse(&format!("[ A ] {{ fill: {name}; }}\n"));
        assert!(
            render(&g).contains(&format!(
                "<td class='node' style=\"background: {hex}\">A</td>"
            )),
            "fill={name}"
        );
    }
    // hex passthrough, rgb() → hex
    let g = parse("[ A ] { fill: #112233; }\n");
    assert!(render(&g).contains("<td class='node' style=\"background: #112233\">A</td>"));
    let g = parse("[ A ] { fill: rgb(10,20,30); }\n");
    assert!(render(&g).contains("<td class='node' style=\"background: #0a141e\">A</td>"));

    // color → color declaration; combined, sorted (background before color)
    let g = parse("[ A ] { color: blue; }\n");
    assert!(render(&g).contains("<td class='node' style=\"color: #0000ff\">A</td>"));
    let g = parse("[ A ] { fill: red; color: blue; }\n");
    assert!(render(&g)
        .contains("<td class='node' style=\"background: #ff0000; color: #0000ff\">A</td>"));

    // `background` on a plain node is suppressed (observed) — only fill
    let g = parse("[ A ] { background: #00ff00; }\n");
    assert!(
        render(&g).contains("<td class='node'>A</td>"),
        "background suppressed"
    );
    assert!(!render(&g).contains("style="), "{:?}", render(&g));

    // edge color: the border value and the color declaration both carry
    // the converted hex (observed edge-color bytes)
    let tds = all_tds(&render(&parse("[ a ] --> { color: red; } [ b ]\n")));
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lh\" style=\"border-bottom: solid 2px #ff0000;color: #ff0000;\"><span class=\"sh\">></span></td>"
    ), "edge color=red");
}

/// The shape vocabulary (observed bytes): rounded/circle/ellipse wrap
/// the label in the div.r/div.c template, point emits the star glyph,
/// invisible drops the class, other shapes leave the plain node td.
fn assert_shape_classes() {
    for (shape, div_class) in [("rounded", "r"), ("circle", "c"), ("ellipse", "c")] {
        let g = parse(&format!("[ A ] {{ shape: {shape}; }}\n"));
        assert!(render(&g).contains(&format!(
            "<td class='node' style=\"border: none;background: inherit\"><div class='{div_class}' style='background:#ffffff;border:solid 1px #000000;width: 3em; height: 3em'><span class='c' style=\"top: 1em\">A</span></div></td>"
        )), "shape {shape}: {}", render(&g));
    }
    let g = parse("[ A ] { shape: point; }\n");
    assert!(
        render(&g).contains("<td class='node' style=\"background: inherit; border: none\">★</td>")
    );
    let g = parse("[ A ] { shape: invisible; }\n");
    assert!(render(&g).contains("<td style=\"border: none; background: inherit;\"></td>"));
    for shape in ["rect", "diamond", "house", "parallelogram", "triangle"] {
        let g = parse(&format!("[ A ] {{ shape: {shape}; }}\n"));
        assert!(
            render(&g).contains("<td class='node'>A</td>"),
            "shape {shape}"
        );
    }
}

/// Node border overrides via the border components (observed width
/// units: bold 4px, wide 1em, broad 0.5em; double no-width; hex colors).
fn assert_border_components() {
    for (border, want) in [
        ("bold", "solid 4px #000000"),
        ("wide", "solid 1em #000000"),
        ("broad", "solid 0.5em #000000"),
        ("double", "double #000000"),
        ("solid red", "solid 1px #ff0000"),
    ] {
        let g = parse(&format!("[ A ] {{ border: {border}; }}\n"));
        assert!(
            render(&g).contains(&format!("<td class='node' style=\"border: {want}\">A</td>")),
            "border {border}: {}",
            render(&g)
        );
    }
}
