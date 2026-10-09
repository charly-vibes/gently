//! Purpose: canonical txt serialization for the graph model — capability
//! ge.txt_render (gently-3hv), gently's port of Graph::Easy 0.69
//! `lib/Graph/Easy/As_txt.pm`.
//! Responsibilities: orchestrate the upstream output passes — class
//! attribute sections first (sorted class order), attribute-bearing node
//! declaration lines, group sections, then every edge exactly once as an
//! operator chain plus edge-less isolated nodes, in upstream
//! `(abs rank, name)` order driven by `_assign_ranks`; attribute-text
//! rules live in the `attributes` submodule.
//! Rationale: the txt form is the golden-test lingua franca — the recorded
//! upstream fixture corpus (tests/fixtures/graph-easy/, pinned to
//! Graph::Easy 0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499) must render
//! byte-identically, so every ordering decision mirrors As_txt.pm exactly;
//! the tracer slice (gently-2po.7) stays green because a graph without
//! attributes has no class sections and the same edge chains.

mod attributes;

use crate::graph::{AttributeTable, Graph, ObjectKind};

use attributes::{class_section, instance_attributes};

/// Serialize the graph to the canonical Graph::Easy txt form.
pub fn render(graph: &Graph) -> String {
    let ranks = assign_ranks(graph);
    let (declarations, declared) = declaration_pass(graph);
    let mut out = class_pass(graph);
    out.push_str(&declarations);
    out.push_str(&group_pass(graph));
    out.push_str(&edge_pass(graph, &ranks, &declared));
    out
}

/// Pass 1 (ge.txt_render.c1): the class attribute sections, plus the blank
/// line that separates them from the body (upstream `$txt .= "\n" if ...`).
fn class_pass(graph: &Graph) -> String {
    let mut sections: Vec<(String, &AttributeTable, ObjectKind)> = Vec::new();
    if !graph.attributes.is_empty() {
        sections.push(("graph".to_string(), &graph.attributes, ObjectKind::Node));
    }
    for (kind, class, table) in &graph.class_attributes {
        if table.is_empty() {
            continue;
        }
        sections.push((class_name(*kind, class), table, *kind));
    }
    sections.sort_by(|x, y| x.0.cmp(&y.0));

    let mut out = String::new();
    for (name, table, kind) in sections {
        out.push_str(&class_section(&name, table, kind));
    }
    if !out.is_empty() {
        out.push('\n');
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
/// `_p` so they are not re-emitted as isolated nodes).
fn declaration_pass(graph: &Graph) -> (String, std::collections::HashSet<usize>) {
    let mut declared: Vec<usize> = graph
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.attributes.is_empty())
        .map(|(i, _)| i)
        .collect();
    declared.sort_by(|x, y| {
        graph.nodes[*x]
            .name
            .cmp(&graph.nodes[*y].name)
            .then(x.cmp(y))
    });
    let mut out = String::new();
    for &i in &declared {
        let node = &graph.nodes[i];
        out.push_str(&node_line(&node.name));
        out.push_str(&instance_attributes(&node.attributes, ObjectKind::Node));
        out.push('\n');
    }
    if !declared.is_empty() {
        out.push('\n');
    }
    (out, declared.into_iter().collect())
}

/// Pass 3: group sections, sorted by name (upstream Group::as_txt; the
/// group stub has no membership yet, so the empty `( name )` form).
fn group_pass(graph: &Graph) -> String {
    let mut groups: Vec<usize> = (0..graph.groups.len()).collect();
    groups.sort_by(|x, y| graph.groups[*x].name.cmp(&graph.groups[*y].name));
    let mut out = String::new();
    for &i in &groups {
        let group = &graph.groups[i];
        out.push_str(&group_line(&group.name));
        out.push_str(&instance_attributes(&group.attributes, ObjectKind::Group));
        out.push_str("\n\n");
    }
    out
}


/// Pass 4: edges as operator chains + isolated nodes, in upstream
/// `(abs rank, name)` order of their from-node.
fn edge_pass(graph: &Graph, ranks: &[isize], declared: &std::collections::HashSet<usize>) -> String {
    // successor lists computed once, shared by ordering and emission
    let succs: Vec<Vec<usize>> = (0..graph.nodes.len())
        .map(|i| successors(graph, i))
        .collect();
    let mut out = String::new();
    for &i in &ordered_nodes(graph, ranks) {
        out.push_str(&node_pass(graph, i, &succs, declared));
    }
    out
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
    let mut out = String::new();
    let name = &graph.nodes[node].name;
    if succ.is_empty() && !has_predecessors(graph, node) && !declared.contains(&node) {
        // a single node without any connection
        out.push_str(&node_line(name));
        out.push('\n');
    }
    let first = node_line(name);
    for &other in &succ {
        out.push_str(&edge_lines(graph, &first, node, other));
    }
    out
}

/// Every `from -> to` edge in model order: `[ from ] <chain> [ to ]`.
fn edge_lines(graph: &Graph, first: &str, from: usize, to: usize) -> String {
    let mut out = String::new();
    for edge in graph.edges.iter().filter(|e| e.from == from && e.to == to) {
        out.push_str(first);
        out.push_str(&edge_chain(edge));
        out.push_str(&node_line(&graph.nodes[to].name));
        out.push('\n');
    }
    out
}

/// The upstream operator mapping (Edge::_as_txt `$styles`) plus the
/// arrowheads from the per-end arrow bits: `<` at the start end, `>` at
/// the end, neither = undirected form (trailing-space styles double).
/// Unknown styles keep `--` plus a visible `style:` attribute (upstream
/// dies; gently renders).
fn edge_chain(edge: &crate::graph::Edge) -> String {
    let label = edge.attributes.get("label").unwrap_or("");
    let style = edge.attributes.get("style").unwrap_or("solid");
    let operator: &str = match style {
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
    let mut operator = operator.to_string();
    if undirected && operator.as_bytes().get(1) == Some(&b' ') {
        // upstream: "make ` -  ` into ` - -  `"
        operator = format!("{operator}{operator}");
    }
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
    let tail = if attrs.is_empty() { String::new() } else { format!("{} ", attrs.trim_start()) };
    format!("{left}{mid}{operator}{right}{tail}")
}

/// `[ name ]` with the upstream name escaping (`[\]\|\{\}\#]`).
fn node_line(name: &str) -> String {
    format!("[ {} ]", escape_chars(name, &['[', ']', '|', '{', '}', '#']))
}

/// Group names escape the group specials (`[`, `]`, `(`, `)`, `{`, `}`, `#`).
fn group_line(name: &str) -> String {
    format!("( {} )", escape_chars(name, &['[', ']', '(', ')', '{', '}', '#']))
}

fn escape_chars(name: &str, specials: &[char]) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if specials.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Upstream `_assign_ranks` (v1: no user ranks, no root node, no groups):
/// nodes without predecessors get rank -1 and their chains deepen by one
/// per hop (-2, -3, …); mid-graph nodes are picked up from the `also`
/// queue at rank -1. The todo list stays sorted ascending by rank with
/// FIFO tie order (upstream Graph::Easy::Heap).
fn assign_ranks(graph: &Graph) -> Vec<isize> {
    let n = graph.nodes.len();
    let mut rank: Vec<Option<isize>> = vec![None; n];
    let mut todo: Vec<(isize, usize)> = Vec::new();
    let mut also: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for (i, _) in graph.nodes.iter().enumerate() {
        if has_predecessors(graph, i) {
            also.push_back(i);
        } else {
            rank[i] = Some(-1);
            insert_stable(&mut todo, (-1, i));
        }
    }
    while !also.is_empty() || !todo.is_empty() {
        propagate_ranks(graph, &mut todo, &mut rank);
        promote_from_also(&mut also, &mut todo, &mut rank);
    }
    // every node is ranked when both queues drain: propagation covers
    // reachable nodes, promotion seeds the rest
    rank.into_iter().map(|r| r.unwrap_or(-1)).collect()
}

/// Drain the todo list, deepening every unranked successor by one hop.
fn propagate_ranks(graph: &Graph, todo: &mut Vec<(isize, usize)>, rank: &mut [Option<isize>]) {
    while let Some(&(r, node)) = todo.first() {
        todo.remove(0);
        let mut l = r;
        if l > 0 {
            l = -l;
        }
        l -= 1;
        for succ in successors(graph, node) {
            if rank[succ].is_none() {
                rank[succ] = Some(l);
                insert_stable(todo, (l, succ));
            }
        }
    }
}

/// Promote one still-unranked mid-graph node to a chain head at rank -1.
fn promote_from_also(
    also: &mut std::collections::VecDeque<usize>,
    todo: &mut Vec<(isize, usize)>,
    rank: &mut [Option<isize>],
) {
    while let Some(x) = also.pop_front() {
        if rank[x].is_none() {
            rank[x] = Some(-1);
            insert_stable(todo, (-1, x));
            break;
        }
    }
}

/// Insert into the ascending-by-rank todo list, after all entries with an
/// equal rank (FIFO tie order — upstream's sorted-linear heap).
fn insert_stable(todo: &mut Vec<(isize, usize)>, entry: (isize, usize)) {
    let pos = todo.partition_point(|&(r, _)| r <= entry.0);
    todo.insert(pos, entry);
}

/// Distinct successors of `node`, in first-seen edge order (upstream's
/// `%suc` weed-out of doubles).
fn successors(graph: &Graph, node: usize) -> Vec<usize> {
    let mut seen = Vec::new();
    for edge in &graph.edges {
        if edge.from == node && !seen.contains(&edge.to) {
            seen.push(edge.to);
        }
    }
    seen
}

fn has_predecessors(graph: &Graph, node: usize) -> bool {
    graph.edges.iter().any(|e| e.to == node)
}

/// Upstream `sorted_successors`: successors with more successors first,
/// ties by name (counts from the precomputed successor matrix).
fn sorted_successors(graph: &Graph, succ: &[usize], succs: &[Vec<usize>]) -> Vec<usize> {
    let mut succ = succ.to_vec();
    succ.sort_by(|x, y| {
        succs[*y]
            .len()
            .cmp(&succs[*x].len())
            .then(graph.nodes[*x].name.cmp(&graph.nodes[*y].name))
    });
    succ
}

/// The edge/isolated-node pass order (upstream `sorted_nodes('rank','name')`):
/// ascending by absolute rank, ties by name.
fn ordered_nodes(graph: &Graph, ranks: &[isize]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..graph.nodes.len()).collect();
    order.sort_by(|x, y| {
        ranks[*x]
            .abs()
            .cmp(&ranks[*y].abs())
            .then(graph.nodes[*x].name.cmp(&graph.nodes[*y].name))
    });
    order
}

#[cfg(test)]
mod tests {
    use crate::graph::{Edge, Graph, Node, Scope};

    #[test]
    fn edge_less_node_emits_bare_node_line() {
        let g = Graph {
            nodes: vec![Node::named("a"), Node::named("b")],
            edges: vec![Edge::directed(0, 1), Edge::directed(0, 1)],
            ..Graph::default()
        };
        let mut g2 = g.clone();
        g2.nodes.push(Node::named("c"));
        assert_eq!(
            super::render(&g2),
            "[ a ] --> [ b ]\n[ a ] --> [ b ]\n[ c ]\n"
        );
    }

    #[test]
    fn empty_graph_emits_empty_output() {
        assert_eq!(super::render(&Graph::default()), "");
    }

    /// Oracle: undirected styles with a trailing space double the operator
    /// (`- - ` for dashed, `= = ` for double-dash).
    #[test]
    fn undirected_trailing_space_styles_double() {
        let mut g = Graph::default();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let e = g.add_edge(a, b, false).expect("live");
        g.edges[e].arrows = Default::default();
        g.set_attr(Scope::Edge(e), "style", "dashed");
        assert_eq!(super::render(&g), "[ a ] - -  [ b ]\n");
        g.set_attr(Scope::Edge(e), "style", "double-dash");
        assert_eq!(super::render(&g), "[ a ] = =  [ b ]\n");
        g.set_attr(Scope::Edge(e), "style", "solid");
        assert_eq!(super::render(&g), "[ a ] -- [ b ]\n");
    }

    /// Oracle: node names escape `[`, `]`, `|`, `{`, `}`, `#`.
    #[test]
    fn node_names_escape_brackets_and_friends() {
        let mut g = Graph::default();
        g.add_node("a|b");
        assert_eq!(super::render(&g), "[ a\\|b ]\n");
    }

    /// Oracle: a named group section renders as `( name )` plus its
    /// instance attributes, followed by a blank line.
    #[test]
    fn group_sections_render_sorted_with_attributes() {
        let mut g = Graph::default();
        g.add_group("B");
        let a = g.add_group("A");
        g.set_attr(Scope::Group(a), "fill", "#ffccaa");
        assert_eq!(super::render(&g), "( A ) { fill: #ffccaa; }\n\n( B )\n\n");
    }
}