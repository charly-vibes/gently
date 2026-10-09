//! ge.html_render (gently-eyo): the table-based HTML renderer contract —
//! one test per property row of specs/ge-html_render.md. Expected bytes
//! are the probed pinned oracle (Graph::Easy v0.69 @ ededa3d7,
//! tests/repro/claims/html-*.observed et al), adapted to gently's
//! one-cell-per-node layout grid (the oracle's colspan/rowspan=4
//! subcell machinery has no counterpart — see render/html).

use gently_core::graph::{Graph, Scope};
use gently_core::render::html;
use gently_core::{layout, parse::text};

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

/// `[ x ] { label: A\nB; }` — a real newline label via the model API.
fn multiline_label() -> Graph {
    let mut g = Graph::default();
    let n = g.add_node("x");
    g.set_attr(Scope::Node(n), "label", "A\nB");
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
    let table = render(&parse("[ a ] --> [ b ] --> [ c ]\n"));
    assert_eq!(table.matches("<td").count(), 5, "chain3: w=5 h=1");

    // the bend cycle: w=3 h=2 — corners (eb), plain horizontals (lh)
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
    assert!(table.contains("<td class='node'>a&amp;b&lt;c&gt;</td>"), "{table}");

    // the observed href-escaping bytes: raw & in the href
    let table = render(&parse("[ A ] { link: 'http://x/?a=1&b=2'; }\n"));
    assert!(table.contains("<td class='node'><a href='http://x/?a=1&b=2'>A</a></td>"), "{table}");

    // href encoding: space → +, ' → %27
    let table = render(&parse("[ A ] { link: 'http://x/a b\\'c'; }\n"));
    assert!(table.contains("<a href='http://x/a+b%27c'>A</a>"), "{table}");

    // multiline labels join with <br> (observed td-colspan bytes);
    // escaped labels too
    let table = render(&multiline_label());
    assert!(table.contains("<td class='node'>A<br>B</td>"), "{table}");
    let table = render(&parse("[ x ] { link: 'u'; label: x&y; }\n"));
    assert!(table.contains("<td class='node'><a href='u'>x&amp;y</a></td>"), "{table}");

    // an empty label is not linkable (upstream) — no a href
    let table = render(&parse("[ ] { link: 'u'; }\n"));
    assert!(table.contains("<td class='node'></td>"), "{table}");
    assert!(!table.contains("<a href"), "{table}");
}

/// p3 (c3): edge styles map to the documented border-image CSS values
/// (the observed remap) on the edge cells.
#[test]
fn p3() {
    assert_edge_style_remap();
    assert_edge_cells();
    assert!(!all_tds(&render(&parse("[ a ] --> [ b ]\n"))).is_empty(), "tracer renders");
}

/// The observed border remap per edge style on the bend-over cells.
fn assert_edge_style_remap() {
    for (style, border) in [
        ("solid", "solid 2px #000000"),
        ("dotted", "dotted 2px #000000"),
        ("dashed", "dashed 2px #000000"),
        ("double", "double #000000"),
        ("wave", "wave 2px #000000"),
        ("bold", "solid 4px #000000"),
        ("wide", "solid 1em #000000"),
        ("broad", "solid 0.5em #000000"),
        ("dot-dash", "dot-dash 2px #000000"),
        ("dot-dot-dash", "dot-dot-dash 2px #000000"),
        ("double-dash", "double #000000"),
        ("bold-dash", "dashed 4px #000000"),
    ] {
        let src = format!(
            "[ a ] --> {{ style: {style}; }} [ b ] --> {{ style: {style}; }} [ c ]\n\
             [ a ] --> {{ style: {style}; }} [ c ]\n"
        );
        let tds = all_tds(&render(&parse(&src)));
        for col in [1usize, 2, 3] {
            assert_eq!(
                tds[col],
                format!("<td class=\"edge lh\" style=\"border-bottom: {border};\">&nbsp;</td>"),
                "style {style} horizontal cell {col}"
            );
        }
        assert_eq!(
            tds[0],
            format!(
                "<td class=\"edge eb\" style=\"border-bottom: {border}; border-left: {border};\">&nbsp;</td>"
            ),
            "style {style} corner cell"
        );
    }
}

/// Arrow spans per direction (observed glyphs), arrowless edges,
/// selfloops, bidirectional edges, edge labels as text, and the plain
/// vertical border cell of a multi-cell vertical run.
fn assert_edge_cells() {
    let arrow = |flow: &str, class: &str, side: &str, span: &str| {
        let tds = all_tds(&render(&parse(&format!("graph {{ flow: {flow}; }}\n[ a ] --> [ b ]\n"))));
        assert!(tds.iter().any(|t| *t == format!(
            "<td class=\"edge {class}\" style=\"border-{side}: solid 2px #000000;color: #000000;\">{span}</td>"
        )), "flow {flow}");
    };
    arrow("east", "lh", "bottom", "<span class=\"sh\">></span>");
    arrow("west", "lh", "bottom", "<span class=\"shl\"><</span>");
    arrow("south", "lv", "left", "<span class=\"sv\">∨</span>");
    arrow("north", "lv", "left", "<span class=\"su\">∧</span>");

    // selfloop: one gap cell above the node, arrow pointing south
    let tds = all_tds(&render(&parse("[ a ] --> [ a ]\n")));
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lv\" style=\"border-left: solid 2px #000000;color: #000000;\"><span class=\"sv\">∨</span></td>"
    ), "selfloop arrow");

    // bidirectional: both arrowheads on the one gap cell
    let tds = all_tds(&render(&parse("[ a ] <--> [ b ]\n")));
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;color: #000000;\"><span class=\"shl\"><</span><span class=\"sh\">></span></td>"
    ), "bidirectional arrows");

    // edge labels render as text on their edge cell (then the arrow);
    // label text is escaped like node labels
    let tds = all_tds(&render(&parse("[ a ] --> { label: go; } [ b ]\n")));
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;color: #000000;\">go<span class=\"sh\">></span></td>"
    ), "labelled edge");
    let tds = all_tds(&render(&parse("[ a ] --> { label: a&b; } [ b ]\n")));
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;color: #000000;\">a&amp;b<span class=\"sh\">></span></td>"
    ), "escaped edge label");

    // arrowless edges emit no span and keep the border cells
    let table = render(&parse("[ a ] --> [ b ]\n[ a ] -- [ b ]\n"));
    assert!(!table.contains("<span"), "arrowless: no arrow spans");
    let tds = all_tds(&table);
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;\">&nbsp;</td>"
    ), "arrowless horizontal cell");
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge eb\" style=\"border-bottom: solid 2px #000000; border-left: solid 2px #000000;\">&nbsp;</td>"
    ), "arrowless corner cell");

    // a multi-cell vertical run carries the plain lv cell
    let tds = all_tds(&render(&parse(
        "[ a ] --> [ b ]\n[ b ] --> [ f ]\n[ a ] --> [ c ]\n[ b ] --> [ d ]\n[ c ] --> [ e ]\n[ a ] --> [ e ]\n",
    )));
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lv\" style=\"border-left: solid 2px #000000;\">&nbsp;</td>"
    ), "plain vertical cell");
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lv\" style=\"border-left: solid 2px #000000;color: #000000;\"><span class=\"sv\">∨</span></td>"
    ), "vertical arrow cell");
}

/// p4 (c4): the emitted document embeds the CSS rules for every class it
/// uses — the document is self-contained (upstream `css()` block shape).
#[test]
fn p4() {
    assert!(!document(&Graph::default()).is_empty(), "document is emitted");
    assert_document_wrapper();
    assert_css_rule_coverage();
}

/// The document shape: the `<style>` block ahead of the table with the
/// observed upstream css() rule blocks.
fn assert_document_wrapper() {
    let g = parse("[ a ] --> [ b ]\n");
    let doc = document(&g);
    assert!(doc.starts_with("<style type=\"text/css\">\n<!--\n"), "{doc}");
    assert!(doc.ends_with("-->\n</style>\n\n\n<table class=\"graph\" cellpadding=0 cellspacing=0>\n"), "{doc}");
    assert!(doc.contains(&render(&g)), "the document embeds the table");
    for rule in [
        "table.graph .edge {\n  font-family: monospaced, courier-new, courier, sans-serif;\n  margin: 0.1em;\n  padding: 0.2em;\n  vertical-align: bottom;\n}",
        "table.graph {\n  empty-cells: show;\n  margin: 0.5em;\n  padding: 0.5em;\n}",
        "table.graph .node,table.graph .node_anon {\n  text-align: center;\n  border-color: #000000;\n  border-style: solid;\n  border-width: 1px;\n  background: #ffffff;\n  margin: 0.1em;\n  padding: 0.2em;\n  padding-left: 0.3em;\n  padding-right: 0.3em;\n}",
        "table.graph td {\n  padding: 2px;\n  background: inherit;\n  white-space: nowrap;\n  }",
        "table.graph .lh, table.graph .lv {\n  font-size: 0.8em;\n  padding-left: 0.4em;\n  }",
        "table.graph .eb { max-height: 0; line-height: 0; height: 0; }",
    ] {
        assert!(doc.contains(rule), "rule block missing: {rule}");
    }
    assert!(!document(&parse("[ A ]\n")).contains(".lh"), "edge rules stay out");
}

/// Every class token referenced by the table has a matching CSS rule,
/// for graph shapes exercising node, edge, arrow, and rounded classes.
fn assert_css_rule_coverage() {
    for src in [
        "[ a ] --> [ b ]\n",
        "graph { flow: north; }\n[ a ] <--> { style: dashed; } [ b ] --> [ a ]\n",
        "[ a ] { shape: rounded; fill: red; } --> { label: go; } [ b ]\n[ a ] --> [ a ]\n",
    ] {
        let table = render(&parse(src));
        let doc = document(&parse(src));
        for token in used_class_tokens(&table) {
            assert!(
                doc.contains(&format!(".{token}")),
                "class '{token}' has no CSS rule: {src:?}"
            );
        }
    }
}

/// Every class token used by any td's class attribute in the table.
fn used_class_tokens(table: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    for td in all_tds(table) {
        let Some(i) = td.find("class=") else { continue };
        let quote = td.as_bytes()[i + 6] as char;
        let class = td[i + 7..].split(quote).next().unwrap_or("");
        for token in class.split_whitespace() {
            tokens.push(token.to_string());
        }
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

/// p5 (c5): `fill`/`background`/`color` map to CSS color declarations via
/// the upstream W3C color-name scheme, and `shape` maps to its documented
/// CSS classes (observed bytes, without the colspan/rowspan machinery).
#[test]
fn p5() {
    assert!(!render(&Graph::default()).contains("style="), "no styles on empty graphs");
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
            render(&g).contains(&format!("<td class='node' style=\"background: {hex}\">A</td>")),
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
    assert!(render(&g).contains("<td class='node' style=\"background: #ff0000; color: #0000ff\">A</td>"));

    // `background` on a plain node is suppressed (observed) — only fill
    let g = parse("[ A ] { background: #00ff00; }\n");
    assert!(render(&g).contains("<td class='node'>A</td>"), "background suppressed");
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
    assert!(render(&g).contains("<td class='node' style=\"background: inherit; border: none\">★</td>"));
    let g = parse("[ A ] { shape: invisible; }\n");
    assert!(render(&g).contains("<td style=\"border: none; background: inherit;\"></td>"));
    for shape in ["rect", "diamond", "house", "parallelogram", "triangle"] {
        let g = parse(&format!("[ A ] {{ shape: {shape}; }}\n"));
        assert!(render(&g).contains("<td class='node'>A</td>"), "shape {shape}");
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
            "border {border}: {}", render(&g)
        );
    }
}
