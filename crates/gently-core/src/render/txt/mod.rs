//! Purpose: canonical txt serialization for the graph model — capability
//! ge.txt_render (gently-3hv), gently's port of Graph::Easy 0.69
//! `lib/Graph/Easy/As_txt.pm`.
//! Responsibilities: orchestrate the upstream output passes — class
//! attribute sections first (sorted class order), attribute-bearing node
//! declaration lines, group sections carrying their member node
//! declarations inside the section (so membership round-trips), then every
//! edge exactly once as an operator chain plus edge-less isolated nodes, in
//! upstream
//! `(abs rank, name)` order driven by `_assign_ranks`; attribute-text
//! rules live in the `attributes` submodule.
//! Rationale: the txt form is the golden-test lingua franca — the recorded
//! upstream fixture corpus (tests/fixtures/graph-easy/, pinned to
//! Graph::Easy 0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499) must render
//! byte-identically, so every ordering decision mirrors As_txt.pm exactly;
//! the tracer slice (gently-2po.7) stays green because a graph without
//! attributes has no class sections and the same edge chains.

mod attributes;
mod ranks;
#[cfg(test)]
mod tests;

use crate::graph::{AttributeTable, Graph, ObjectKind};

use attributes::{class_section, instance_attributes};
use ranks::{assign_ranks, has_predecessors, ordered_nodes, sorted_successors, successors};

/// Serialize the graph to the canonical Graph::Easy txt form.
pub fn render(graph: &Graph) -> String {
    let ranks = assign_ranks(graph);
    let (declarations, mut declared) = declaration_pass(graph);
    let mut out = class_pass(graph);
    out.push_str(&declarations);
    out.push_str(&group_pass(graph, &mut declared));
    out.push_str(&edge_pass(graph, &ranks, &declared));
    out
}

/// Pass 1 (ge.txt_render.c1): the class attribute sections, plus the blank
/// line that separates them from the body (upstream `$txt .= "\n" if ...`).
fn class_pass(graph: &Graph) -> String {
    let mut sections: Vec<(String, &AttributeTable, ObjectKind)> =
        std::iter::once(("graph".to_string(), &graph.attributes, ObjectKind::Node))
            .filter(|(_, table, _)| !table.is_empty())
            .chain(
                graph
                    .class_attributes
                    .iter()
                    .filter(|(_, _, table)| !table.is_empty())
                    .map(|(kind, class, table)| (class_name(*kind, class), table, *kind)),
            )
            .collect();
    sections.sort_by(|x, y| x.0.cmp(&y.0));

    let out: String = sections
        .iter()
        .map(|(name, table, kind)| class_section(name, table, *kind))
        .collect();
    if !out.is_empty() {
        return format!("{out}\n");
    }
    out
}

/// The upstream class name for a class scope: `node`, `node.<class>`, …
fn class_name(kind: ObjectKind, class: &str) -> String {
    match class {
        "" => kind_name(kind).to_string(),
        c => format!("{}.{}", kind_name(kind), c),
    }
}

fn kind_name(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Node => "node",
        ObjectKind::Edge => "edge",
        ObjectKind::Group => "group",
    }
}

/// Pass 2: attribute-bearing nodes as declaration lines, sorted by name,
/// followed by a blank line when any were emitted (upstream marks them
/// `_p` so they are not re-emitted as isolated nodes). Group members are
/// excluded: a node belongs to exactly one group, so its declaration —
/// attributes included — lives inside that group's section (pass 3).
fn declaration_pass(graph: &Graph) -> (String, std::collections::HashSet<usize>) {
    let members: std::collections::HashSet<usize> = graph
        .groups
        .iter()
        .flat_map(|g| g.members.iter().copied())
        .collect();
    let declared = declared_nodes(graph, &members);
    let out: String = declared
        .iter()
        .map(|&i| {
            let node = &graph.nodes[i];
            format!(
                "{}{}\n",
                node_line(&node.name),
                instance_attributes(&node.attributes, ObjectKind::Node)
            )
        })
        .collect();
    let body = if declared.is_empty() {
        out
    } else {
        format!("{out}\n")
    };
    (body, declared.into_iter().collect())
}

/// Upstream As_txt marks a node for declaration only when its
/// POST-canonicalization attribute text is non-empty ($att ne '' after
/// attributes_as_txt) — a table whose every attribute collapses (e.g.
/// size: 1,1 dropping to the default) leaves the node undeclared
/// (gently-awd). Sorted by name (name ties by model order).
fn declared_nodes(graph: &Graph, members: &std::collections::HashSet<usize>) -> Vec<usize> {
    let mut declared: Vec<usize> = graph
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, n)| {
            !members.contains(i)
                && !instance_attributes(&n.attributes, ObjectKind::Node)
                    .trim()
                    .is_empty()
        })
        .map(|(i, _)| i)
        .collect();
    declared.sort_by(|x, y| {
        graph.nodes[*x]
            .name
            .cmp(&graph.nodes[*y].name)
            .then(x.cmp(y))
    });
    declared
}

/// Pass 3: group sections, sorted by name (upstream Group::as_txt). A
/// member-bearing group emits the upstream section shape — the header
/// `( name`, then each member's node declaration indented two spaces, then
/// `)` (tests/repro/claims/group-edge-graphviz.observed) — so membership
/// round-trips through ge.text_parser's group-scope rule (a node line
/// inside a group scope joins that group, c6). Member nodes are marked
/// declared so pass 4 does not re-emit them as isolated nodes. An empty
/// group keeps the parser's `( name )` empty-section form.
fn group_pass(graph: &Graph, declared: &mut std::collections::HashSet<usize>) -> String {
    let mut groups: Vec<usize> = (0..graph.groups.len()).collect();
    groups.sort_by(|x, y| graph.groups[*x].name.cmp(&graph.groups[*y].name));
    groups
        .iter()
        .map(|&i| group_section(graph, i, declared))
        .collect()
}

/// An empty group keeps the parser's `( name )` empty-section form.
fn empty_group_section(group: &crate::graph::Group) -> String {
    format!(
        "{}{}\n\n",
        group_line(&group.name),
        instance_attributes(&group.attributes, ObjectKind::Group)
    )
}

/// One group's section: a member-bearing group emits the upstream shape —
/// header `( name`, each member's node declaration indented two spaces,
/// then `)` (tests/repro/claims/group-edge-graphviz.observed) — so
/// membership round-trips through ge.text_parser's group-scope rule (a
/// node line inside a group scope joins that group, c6). Member nodes are
/// marked declared so pass 4 does not re-emit them as isolated nodes.
fn group_section(
    graph: &Graph,
    i: usize,
    declared: &mut std::collections::HashSet<usize>,
) -> String {
    let group = &graph.groups[i];
    if group.members.is_empty() {
        return empty_group_section(group);
    }
    let mut out = String::new();
    out.push_str("( ");
    out.push_str(&escape_chars(
        &group.name,
        &['[', ']', '(', ')', '{', '}', '#'],
    ));
    out.push('\n');
    for &m in &group.members {
        let node = &graph.nodes[m];
        out.push_str("  ");
        out.push_str(&node_line(&node.name));
        out.push_str(&instance_attributes(&node.attributes, ObjectKind::Node));
        out.push('\n');
        declared.insert(m);
    }
    out.push(')');
    out.push_str(&instance_attributes(&group.attributes, ObjectKind::Group));
    out.push_str("\n\n");
    out
}

/// Pass 4: edges as operator chains + isolated nodes, in upstream
/// `(abs rank, name)` order of their from-node.
fn edge_pass(
    graph: &Graph,
    ranks: &[isize],
    declared: &std::collections::HashSet<usize>,
) -> String {
    // successor lists computed once, shared by ordering and emission
    let succs: Vec<Vec<usize>> = (0..graph.nodes.len())
        .map(|i| successors(graph, i))
        .collect();
    ordered_nodes(graph, ranks)
        .iter()
        .map(|&i| node_pass(graph, i, &succs, declared))
        .collect()
}

/// One node's contribution to pass 4: an isolated bare line when the node
/// has neither edges nor a declaration, then one chain line per outgoing
/// edge in model order.
fn node_pass(
    graph: &Graph,
    node: usize,
    succs: &[Vec<usize>],
    declared: &std::collections::HashSet<usize>,
) -> String {
    let succ = sorted_successors(graph, &succs[node], succs);
    let name = &graph.nodes[node].name;
    let isolated = succ.is_empty() && !has_predecessors(graph, node) && !declared.contains(&node);
    let bare = if isolated {
        format!("{}\n", node_line(name))
    } else {
        String::new()
    };
    let first = node_line(name);
    let chains: String = succ
        .iter()
        .map(|&other| edge_lines(graph, &first, node, other))
        .collect();
    format!("{bare}{chains}")
}

/// Every `from -> to` edge in model order: `[ from ] <chain> [ to ]`.
fn edge_lines(graph: &Graph, first: &str, from: usize, to: usize) -> String {
    graph
        .edges
        .iter()
        .filter(|e| e.from == from && e.to == to)
        .map(|edge| {
            format!(
                "{first}{}{}\n",
                edge_chain(edge),
                node_line(&graph.nodes[to].name)
            )
        })
        .collect()
}

/// The upstream operator mapping (Edge::_as_txt `$styles`) plus the
/// arrowheads from the per-end arrow bits: `<` at the start end, `>` at
/// the end, neither = undirected form (trailing-space styles double).
/// Unknown styles keep `--` plus a visible `style:` attribute (upstream
/// dies; gently renders).
fn edge_chain(edge: &crate::graph::Edge) -> String {
    let label = edge.attributes.get("label").unwrap_or("");
    let operator = chain_operator(edge);
    let left = if edge.arrows.start { " <" } else { " " };
    let right = if edge.arrows.end { "> " } else { " " };
    let mid = if label.is_empty() {
        String::new()
    } else {
        format!("{operator} {label} ")
    };
    // upstream: `$a = attributes_as_txt . ' '; $a =~ s/^\s//` — the
    // leading brace-space becomes a trailing space
    let attrs = instance_attributes(&edge.attributes, ObjectKind::Edge);
    let tail = if attrs.is_empty() {
        String::new()
    } else {
        format!("{} ", attrs.trim_start())
    };
    format!("{left}{mid}{operator}{right}{tail}")
}

/// The upstream operator mapping (Edge::_as_txt `$styles`) plus the
/// arrowhead-driven form; unknown styles keep `--` plus a visible
/// `style:` attribute (upstream dies; gently renders).
fn chain_operator(edge: &crate::graph::Edge) -> String {
    let style = edge.attributes.get("style").unwrap_or("solid");
    let base: &str = match style {
        "solid" => "--",
        "dotted" => "..",
        "double" => "==",
        "double-dash" => "= ",
        "dashed" => "- ",
        "dot-dash" => ".-",
        "dot-dot-dash" => "..-",
        "wave" => "~~",
        _ => "--",
    };
    let undirected = !edge.arrows.start && !edge.arrows.end;
    if undirected && base.as_bytes().get(1) == Some(&b' ') {
        // upstream: "make ` -  ` into ` - -  `"
        format!("{base}{base}")
    } else {
        base.to_string()
    }
}

/// `[ name ]` with the upstream name escaping (`[\]\|\{\}\#]`).
fn node_line(name: &str) -> String {
    format!(
        "[ {} ]",
        escape_chars(name, &['[', ']', '|', '{', '}', '#'])
    )
}

/// Group names escape the group specials (`[`, `]`, `(`, `)`, `{`, `}`, `#`).
fn group_line(name: &str) -> String {
    format!(
        "( {} )",
        escape_chars(name, &['[', ']', '(', ')', '{', '}', '#'])
    )
}

fn escape_chars(name: &str, specials: &[char]) -> String {
    name.chars()
        .flat_map(|c| {
            if specials.contains(&c) {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}
