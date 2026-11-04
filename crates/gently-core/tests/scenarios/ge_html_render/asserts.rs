use super::*;

/// The observed border remap per edge style on the bend-over cells.
pub(crate) fn assert_edge_style_remap() {
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
pub(crate) fn assert_edge_cells() {
    let arrow = |flow: &str, class: &str, side: &str, span: &str| {
        let tds = all_tds(&render(&parse(&format!(
            "graph {{ flow: {flow}; }}\n[ a ] --> [ b ]\n"
        ))));
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

    // arrowless edges emit no span (a lone `--` edge) and keep the
    // border cells (the parallel bend of directed+undirected)
    assert!(
        !render(&parse("[ a ] -- [ b ]\n")).contains("<span"),
        "arrowless: no arrow spans"
    );
    let table = render(&parse("[ a ] --> [ b ]\n[ a ] -- [ b ]\n"));
    let tds = all_tds(&table);
    assert!(
        tds.iter().any(|t| *t
            == "<td class=\"edge lh\" style=\"border-bottom: solid 2px #000000;\">&nbsp;</td>"),
        "arrowless horizontal cell"
    );
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge eb\" style=\"border-bottom: solid 2px #000000; border-left: solid 2px #000000;\">&nbsp;</td>"
    ), "arrowless corner cell");

    // a multi-cell vertical run carries the plain lv cell
    let tds = all_tds(&render(&parse(
        "[ a ] --> [ b ]\n[ b ] --> [ f ]\n[ a ] --> [ c ]\n[ b ] --> [ d ]\n[ c ] --> [ e ]\n[ a ] --> [ e ]\n",
    )));
    assert!(
        tds.iter()
            .any(|t| *t
                == "<td class=\"edge lv\" style=\"border-left: solid 2px #000000;\">&nbsp;</td>"),
        "plain vertical cell"
    );
    assert!(tds.iter().any(|t| *t ==
        "<td class=\"edge lv\" style=\"border-left: solid 2px #000000;color: #000000;\"><span class=\"sv\">∨</span></td>"
    ), "vertical arrow cell");
}

/// The document shape: the `<style>` block ahead of the table with the
/// observed upstream css() rule blocks.
pub(crate) fn assert_document_wrapper() {
    let g = parse("[ a ] --> [ b ]\n");
    let doc = document(&g);
    assert!(
        doc.starts_with("<style type=\"text/css\">\n<!--\n"),
        "{doc}"
    );
    assert!(
        doc.contains("-->\n</style>\n\n\n<table class=\"graph\" cellpadding=0 cellspacing=0>\n"),
        "{doc}"
    );
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
    assert!(
        !document(&parse("[ A ]\n")).contains(".lh"),
        "edge rules stay out"
    );
}

/// Every class token the table references has a matching CSS rule,
/// across node/edge/arrow/rounded-shape graph shapes.
pub(crate) fn assert_css_rule_coverage() {
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
pub(crate) fn used_class_tokens(table: &str) -> Vec<String> {
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
