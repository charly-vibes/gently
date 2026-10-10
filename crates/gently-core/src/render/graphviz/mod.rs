//! Purpose: the Graphviz DOT output renderer — gently's port of upstream
//! Graph::Easy's `as_graphviz` (gently-b4v, specs/ge-graphviz_render.md).
//! Responsibilities: consume the graph model directly (dot lays out its
//! own graph — the layout grid is not involved) and emit DOT source whose
//! elements correspond one-to-one with the model: one node statement per
//! node with safely quoted names and mapped attributes (c1), one edge
//! statement per model edge with the arrow mirroring the MODEL direction
//! — `->` directed, `--` undirected — and mapped style attributes (c2),
//! and one subgraph cluster per group containing exactly its members with
//! the group label as cluster label; named groups keep their name,
//! anonymous groups (empty name) emit `cluster<N>` in internal-id order
//! (c3). Feeding the emission through ge.dot_parser yields an isomorphic
//! model (c4).
//! Rationale: shaped by the probed pinned-oracle bytes (tests/repro/
//! claims/graphviz-round-trip.observed, Graph::Easy v0.69 @ ededa3d7):
//! `digraph GRAPH_0` header, generated-by comment, global `edge/graph/node`
//! default blocks, then per-object statements. Where the oracle is lossy
//! the spec row wins, as documented divergences:
//! - arrows mirror the model edge direction (the oracle always emits
//!   `digraph` + `->`, losing undirected edges);
//! - named groups keep their name as the cluster id (`cluster_<name>`;
//!   the oracle renumbers every cluster `cluster0..N`), anonymous groups
//!   use `cluster<N>` after their internal id;
//! - anonymous nodes (empty model name) emit the `#N` id after their
//!   node index (upstream names them `#<odd global id>`).

mod quote;

use crate::graph::{AttributeTable, Graph};
use quote::quote;

/// The DOT key for a model attribute key (`fill` maps to `fillcolor`,
/// the rest pass through verbatim).
fn dot_key(key: &str) -> &str {
    match key {
        "fill" => "fillcolor",
        other => other,
    }
}

/// `k=v` pairs for `table`, sorted by the model's key order, keys mapped
/// to their DOT counterparts, values safely quoted.
fn attr_list(table: &AttributeTable) -> String {
    table
        .sorted_entries()
        .iter()
        .map(|(k, v)| format!("{}={}", dot_key(k), quote(v)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// One node statement: the quoted display name (anonymous nodes emit
/// `#<index>`) plus the mapped attribute list.
fn node_statement(graph: &Graph, index: usize) -> String {
    let node = &graph.nodes[index];
    let name = if node.name.is_empty() { format!("#{index}") } else { node.name.clone() };
    let attrs = attr_list(&node.attributes);
    if attrs.is_empty() {
        quote(&name)
    } else {
        format!("{} [ {} ]", quote(&name), attrs)
    }
}

/// One edge statement: quoted endpoints, the arrow mirroring the MODEL
/// edge direction, and the mapped attribute list (`dir=both` for
/// bidirectional edges).
fn edge_statement(graph: &Graph, index: usize) -> String {
    let edge = &graph.edges[index];
    let from = quote(&graph.nodes[edge.from].name);
    let to = quote(&graph.nodes[edge.to].name);
    let arrow = if edge.directed { "->" } else { "--" };
    let mut attrs = attr_list(&edge.attributes);
    if edge.arrows.start && edge.directed {
        attrs = if attrs.is_empty() {
            "dir=both".to_string()
        } else {
            format!("{attrs}, dir=both")
        };
    }
    if attrs.is_empty() {
        format!("{from} {arrow} {to}")
    } else {
        format!("{from} {arrow} {to} [ {attrs} ]")
    }
}

/// One subgraph cluster for `group`: named groups keep their name as
/// `cluster_<name>`, anonymous groups (empty name) emit `cluster<N>` in
/// internal-id order; the cluster label is the group's `label` attribute,
/// falling back to the group name; exactly the member nodes are declared
/// inside.
fn cluster(graph: &Graph, index: usize) -> String {
    let group = &graph.groups[index];
    let id = if group.name.is_empty() {
        format!("cluster{index}")
    } else {
        format!("cluster_{}", group.name)
    };
    // c3: the group label as cluster label — emitted only for an explicit
    // `label` attribute. A name-fallback label is NOT emitted: the
    // ge.dot_parser grammar has no graph-attribute statement inside a
    // scope, so emitting one would break the c4 round trip for every
    // named group (documented deviation).
    let label_line = match group.attributes.get("label") {
        Some(label) => format!("  label={};\n\n", quote(label)),
        None => String::from("\n"),
    };
    let members: String = group
        .members
        .iter()
        .map(|&n| format!("  {}\n", node_statement(graph, n)))
        .collect();
    format!("  subgraph {} {{\n{label_line}{}}}\n", id, members)
}

/// Render `graph` as DOT source (upstream `as_graphviz`): the oracle
/// skeleton (header) followed by top-level nodes, then clusters, then
/// edge statements — the order the pinned oracle uses. Deviations from
/// the oracle skeleton, forced by the c4 round trip (the ge.dot_parser
/// grammar has no graph-attribute statement): the global `edge [ … ]` /
/// `graph [ … ]` / `node [ … ]` default blocks are omitted — the parser
/// would ingest them as spurious `edge`/`graph`/`node` NODE statements —
/// and there are no `//` comments (the lexer rejects them).
pub fn render(graph: &Graph) -> String {
    let grouped: Vec<usize> = graph.groups.iter().flat_map(|grp| grp.members.iter().copied()).collect();
    let mut out = String::from(
        "digraph GRAPH_0 {\n\n",
    );
    for i in 0..graph.nodes.len() {
        if grouped.contains(&i) {
            continue;
        }
        out.push_str(&format!("  {}\n", node_statement(graph, i)));
    }
    for i in 0..graph.groups.len() {
        out.push_str(&cluster(graph, i));
        out.push('\n');
    }
    for i in 0..graph.edges.len() {
        out.push_str(&format!("  {}\n", edge_statement(graph, i)));
    }
    out.push_str("}\n");
    out
}