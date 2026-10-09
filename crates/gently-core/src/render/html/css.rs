//! Purpose: the CSS rule set of the HTML renderer (ge.html_render.c4) —
//! the upstream Graph::Easy 0.69 `css()` blocks (lib/Graph/Easy.pm),
//! emitted for every class the table uses so the document is
//! self-contained.
//! Responsibilities: emit the static per-primary-class rules (edge,
//! graph, group_anon, node, node_anon), the group padding rule when
//! groups exist, the td/span base rules, the edge-part rules (va/el/lh/
//! lv/sh/shl/sv/sa/su/eb) when the graph has edges, and the rounded-shape
//! rules (span.c, div.c, div.r) when a node is rounded/circle/ellipse.
//! Rationale: byte-shaped by the probed pinned oracle
//! (tests/repro/claims/html-css-rules.observed); upstream anchors the
//! selectors to `table.graph#ID` when the graph carries an id — gently
//! emits the idless `table.graph` form (the `##id##` substitution with an
//! empty id, which is the observed plain-graph output).

/// The CSS rules for the classes the rendered table uses.
pub(in crate::render) fn css(has_edges: bool, has_groups: bool, rounded: bool) -> String {
    let mut out = String::new();
    // the per-primary-class rules (observed: always emitted, in this order)
    out.push_str(
        "table.graph .edge {\n  font-family: monospaced, courier-new, courier, sans-serif;\n  margin: 0.1em;\n  padding: 0.2em;\n  vertical-align: bottom;\n}\n",
    );
    out.push_str("table.graph {\n  empty-cells: show;\n  margin: 0.5em;\n  padding: 0.5em;\n}\n");
    out.push_str("table.graph .group_anon {\n  border-style: none;\n}\n");
    out.push_str(
        "table.graph .node,table.graph .node_anon {\n  text-align: center;\n  border-color: #000000;\n  border-style: solid;\n  border-width: 1px;\n  background: #ffffff;\n  margin: 0.1em;\n  padding: 0.2em;\n  padding-left: 0.3em;\n  padding-right: 0.3em;\n}\n",
    );
    out.push_str("table.graph .node_anon {\n  border-style: none;\n}\n");
    if has_groups {
        out.push_str("table.graph td[class|=\"group\"] { padding: 0.2em; }\n");
    }
    out.push_str(
        "table.graph td {\n  padding: 2px;\n  background: inherit;\n  white-space: nowrap;\n  }\ntable.graph span.l { float: left; }\ntable.graph span.r { float: right; }\n",
    );
    if has_edges {
        out.push_str(EDGE_PART_RULES);
    }
    if rounded {
        out.push_str(
            "table.graph span.c { position: relative; top: 1.5em; }\ntable.graph div.c { -moz-border-radius: 100%; border-radius: 100%; }\ntable.graph div.r { -moz-border-radius: 1em; border-radius: 1em; }\n",
        );
    }
    out
}

/// The edge-part rules (upstream: emitted when the graph has edges).
const EDGE_PART_RULES: &str = "\
table.graph .va {
  vertical-align: middle;
  line-height: 1em;
  width: 0.4em;
  }
table.graph .el {
  width: 0.1em;
  max-width: 0.1em;
  min-width: 0.1em;
  }
table.graph .lh, table.graph .lv {
  font-size: 0.8em;
  padding-left: 0.4em;
  }
table.graph .sv, table.graph .sh, table.graph .shl, table.graph .sa, table.graph .su {
  max-height: 1em;
  line-height: 1em;
  position: relative;
  top: 0.55em;
  left: -0.3em;
  overflow: visible;
  }
table.graph .sv, table.graph .su {
  max-height: 0.5em;
  line-height: 0.5em;
  }
table.graph .shl { left: 0.3em; }
table.graph .sv { left: -0.5em; top: -0.4em; }
table.graph .su { left: -0.5em; top: 0.4em; }
table.graph .sa { left: -0.3em; top: 0; }
table.graph .eb { max-height: 0; line-height: 0; height: 0; }
";
