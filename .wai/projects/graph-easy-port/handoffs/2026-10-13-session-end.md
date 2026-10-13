---
date: 2026-10-13
project: graph-easy-port
phase: research
---

# Session Handoff

## What Was Done

<!-- Summary of completed work -->

## Key Decisions

<!-- Decisions made and rationale -->

## Gotchas & Surprises

<!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

## What Took Longer Than Expected

<!-- Steps that needed multiple attempts. Commands that failed before the right one. -->

## Open Questions

<!-- Unresolved questions -->

## Next Steps

<!-- Prioritized list of what to do next -->

## Context

### git_status

```
 M crates/gently-cli/src/doctor.rs
 M crates/gently-cli/src/suggest.rs
 M crates/gently-cli/tests/cli.rs
 M crates/gently-cli/tests/common/mod.rs
 M crates/gently-core/examples/gallery.rs
 M crates/gently-core/src/graph/attributes.rs
 M crates/gently-core/src/graph/mod.rs
 M crates/gently-core/src/layout.rs
 M crates/gently-core/src/layout/chains.rs
 M crates/gently-core/src/layout/flow.rs
 M crates/gently-core/src/layout/paths.rs
 M crates/gently-core/src/layout/placement.rs
 M crates/gently-core/src/lib.rs
 M crates/gently-core/src/parse.rs
 M crates/gently-core/src/parse/dot/attrs.rs
 M crates/gently-core/src/parse/dot/mod.rs
 M crates/gently-core/src/parse/dot/record.rs
 M crates/gently-core/src/parse/dot/scope.rs
 M crates/gently-core/src/parse/dot/stmt.rs
 M crates/gently-core/src/parse/text/attrs.rs
 M crates/gently-core/src/parse/text/edge.rs
 M crates/gently-core/src/parse/text/mod.rs
 M crates/gently-core/src/parse/text/node.rs
 M crates/gently-core/src/render/ascii/canvas.rs
 M crates/gently-core/src/render/ascii/edges.rs
 M crates/gently-core/src/render/ascii/geometry.rs
 M crates/gently-core/src/render/ascii/groups.rs
 M crates/gently-core/src/render/ascii/labels.rs
 M crates/gently-core/src/render/ascii/mod.rs
 M crates/gently-core/src/render/ascii/polyline.rs
 M crates/gently-core/src/render/ascii/styles.rs
 M crates/gently-core/src/render/boxart/canvas.rs
 M crates/gently-core/src/render/boxart/columns.rs
 M crates/gently-core/src/render/boxart/mod.rs
 M crates/gently-core/src/render/boxart/polyline.rs
 M crates/gently-core/src/render/boxart/styles.rs
 M crates/gently-core/src/render/graphviz/mod.rs
 M crates/gently-core/src/render/graphviz/quote.rs
 M crates/gently-core/src/render/html/mod.rs
 M crates/gently-core/src/render/html/styles.rs
 M crates/gently-core/src/render/txt/attributes.rs
 M crates/gently-core/src/render/txt/mod.rs
 M crates/gently-core/tests/scenarios.rs
 M crates/gently-core/tests/scenarios/ge_ascii_render.rs
 M crates/gently-core/tests/scenarios/ge_ascii_render_edge_labels.rs
 M crates/gently-core/tests/scenarios/ge_ascii_render_edge_node.rs
 M crates/gently-core/tests/scenarios/ge_ascii_render_klf.rs
 M crates/gently-core/tests/scenarios/ge_ascii_render_node_labels.rs
 M crates/gently-core/tests/scenarios/ge_boxart_render.rs
 M crates/gently-core/tests/scenarios/ge_dot_parser.rs
 M crates/gently-core/tests/scenarios/ge_graph_model.rs
 M crates/gently-core/tests/scenarios/ge_graphviz_render.rs
 M crates/gently-core/tests/scenarios/ge_html_render.rs
 M crates/gently-core/tests/scenarios/ge_layout.rs
 M crates/gently-core/tests/scenarios/ge_layout_ports.rs
 M crates/gently-core/tests/scenarios/ge_oracle.rs
 M crates/gently-core/tests/scenarios/ge_perf.rs
 M crates/gently-core/tests/scenarios/ge_text_parser.rs
 M crates/gently-core/tests/scenarios/ge_text_parser_p11.rs
 M crates/gently-core/tests/scenarios/ge_text_parser_p12.rs
 M crates/gently-core/tests/scenarios/ge_text_parser_p13.rs
 M crates/gently-core/tests/scenarios/ge_txt_render.rs
 M crates/gently-core/tests/tb/corpus.rs
 M justfile
 M lefthook.yml
 M specodelic/ge-ascii_render.check.json
 M specodelic/ge-boxart_render.check.json
 M specodelic/ge-dot_parser.check.json
 M specodelic/ge-graph_model.check.json
 M specodelic/ge-graphviz_render.check.json
 M specodelic/ge-html_render.check.json
 M specodelic/ge-layout.check.json
 M specodelic/ge-oracle.check.json
 M specodelic/ge-perf.check.json
 M specodelic/ge-text_parser.check.json
 M specodelic/ge-txt_render.check.json
?? pretender.toml
```

### open_issues

```
○ gently-4ga P2 [bug] gently: comma-chain fan layouts hit the render adjacency error (edge path cells not orthogonally adjacent)
○ gently-58m P2 [bug] gently cannot parse anonymous-group syntax '( [ Bonn ] ) -> [ Berlin ]' from Parser.pm POD
○ gently-awd P2 [bug] so-flowchart-size layout remainder: { size: 5,10; } node attrs ignored and the 2x2 layout diverges (SO 47323345)
○ gently-byq P2 [bug] so-start-port-index: indexed start/end port attrs {start: front,N} ignored (SO 47323345 community workaround)
○ gently-ca0 P2 lefthook: chain bd hooks and add pretender.toml
○ gently-fb1 P2 [bug] dot_parser: bare scope nested in a named subgraph joins the enclosing group in the oracle; gently keeps its nodes ungrouped
○ gently-cpc P3 README: ddl-standard badge set (CI, crates.io, license, docs)
○ gently-lmu P3 CI: add release.yml and smoke.yml workflows (ddl parity)
○ gently-skn P3 Add CHANGELOG.md (Keep a Changelog format)
○ gently-hny P4 mdbook docs (book.toml + docs/src) with docs/just recipes — deferred until docs warrant it

--------------------------------------------------------------------------------
Total: 10 issues (10 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```

