use super::*;

/// Model equality for the round-trip (c3): the same node-name set with
/// equal attribute tables and the same edge multiset keyed by endpoint
/// names, direction and per-end arrows with equal attribute tables, and
/// the same group set with equal attributes and equal membership —
/// members compared by node NAME (group membership is a name relation;
/// re-parsed node indices may differ from the source's). The
/// model-equality predicate for the parser-supported feature subset (see
/// the documented deviation in p3).
pub(crate) fn model_equivalent(a: &Graph, b: &Graph) -> bool {
    let node_names =
        |g: &Graph| -> Vec<String> { sorted(g.nodes.iter().map(|n| n.name.clone()).collect()) };
    let node_attrs = |g: &Graph| -> Vec<(String, gently_core::graph::AttributeTable)> {
        sorted_by(
            g.nodes
                .iter()
                .map(|n| (n.name.clone(), n.attributes.clone()))
                .collect(),
            |x, y| x.0.cmp(&y.0),
        )
    };
    let edge_keys = |g: &Graph| -> Vec<(String, String, bool, bool, bool)> {
        sorted(
            g.edges
                .iter()
                .map(|e| {
                    (
                        g.nodes[e.from].name.clone(),
                        g.nodes[e.to].name.clone(),
                        e.directed,
                        e.arrows.start,
                        e.arrows.end,
                    )
                })
                .collect(),
        )
    };
    let group_keys = |g: &Graph| -> Vec<(String, Vec<String>, gently_core::graph::AttributeTable)> {
        sorted_by(
            g.groups
                .iter()
                .map(|grp| {
                    (
                        grp.name.clone(),
                        grp.members
                            .iter()
                            .map(|&m| g.nodes[m].name.clone())
                            .collect(),
                        grp.attributes.clone(),
                    )
                })
                .collect(),
            |x, y| x.0.cmp(&y.0),
        )
    };
    node_names(a) == node_names(b)
        && node_attrs(a) == node_attrs(b)
        && edge_keys(a) == edge_keys(b)
        && a.attributes == b.attributes
        && a.class_attributes == b.class_attributes
        && group_keys(a) == group_keys(b)
}

/// The round-trip sources: shapes within the parser-supported feature set.
pub(crate) fn round_trip_sources() -> Vec<Graph> {
    let mut sources = connected_shapes();
    sources.push(diamond_shape());
    sources.extend(group_shapes());
    sources
}

/// Group shapes: a named group whose member sits on an edge, a group whose
/// members are isolated (one bearing node attributes), a group with
/// group-level attributes, and an empty group — the membership-carrier
/// forms for the c3 round-trip.
pub(crate) fn group_shapes() -> Vec<Graph> {
    let mut edged_member = Graph::default();
    let a = edged_member.add_node("a");
    let b = edged_member.add_node("b");
    edged_member.add_edge(a, b, true);
    let g = edged_member.add_group("G");
    edged_member.set_node_group(b, g);

    let mut isolated_members = Graph::default();
    let x = isolated_members.add_node("x");
    let y = isolated_members.add_node("y");
    let ga = isolated_members.add_group("A");
    isolated_members.set_node_group(x, ga);
    isolated_members.set_node_group(y, ga);
    isolated_members.set_attr(Scope::Node(x), "fill", "red");

    let mut attributed_group = Graph::default();
    let c = attributed_group.add_node("c");
    let h = attributed_group.add_group("H");
    attributed_group.set_node_group(c, h);
    attributed_group.set_attr(Scope::Group(h), "fill", "#ffccaa");

    let mut empty = Graph::default();
    empty.add_group("E");

    vec![edged_member, isolated_members, attributed_group, empty]
}

/// Chain-derived shapes: a chain, an isolated node appended, parallel
/// duplicate edges, a self-loop, and a two-node cycle.
pub(crate) fn connected_shapes() -> Vec<Graph> {
    let mut chain = Graph::default();
    let a = chain.add_node("a");
    let b = chain.add_node("b");
    let c = chain.add_node("c");
    chain.add_edge(a, b, true);
    chain.add_edge(b, c, true);
    let mut mixed = chain.clone();
    mixed.add_node("d");

    let mut parallel = Graph::default();
    let a = parallel.add_node("a");
    let b = parallel.add_node("b");
    parallel.add_edge(a, b, true);
    parallel.add_edge(a, b, true);

    let mut looped = Graph::default();
    let a = looped.add_node("a");
    looped.add_edge(a, a, true);

    let mut cycle = Graph::default();
    let a = cycle.add_node("a");
    let b = cycle.add_node("b");
    cycle.add_edge(a, b, true);
    cycle.add_edge(b, a, true);

    vec![Graph::tracer(), chain, mixed, parallel, looped, cycle]
}

/// A diamond: two parallel branches merging into one target.
pub(crate) fn diamond_shape() -> Graph {
    let mut diamond = Graph::default();
    let a = diamond.add_node("a");
    let b = diamond.add_node("b");
    let c = diamond.add_node("c");
    let d = diamond.add_node("d");
    diamond.add_edge(a, b, true);
    diamond.add_edge(a, c, true);
    diamond.add_edge(b, d, true);
    diamond.add_edge(c, d, true);
    diamond
}

/// Labels render in the upstream form (`-- go -->`) and round-trip through
/// the full ge.text_parser grammar (gently-bzx implemented c4 — the
/// earlier tracer-era deviation, "labels cannot be read back", is gone).
pub(crate) fn labels_do_not_survive_the_supported_subset() {
    let mut g = Graph::default();
    let e = chain_edge(&mut g, "a", "b", true);
    g.set_attr(Scope::Edge(e), "label", "go");
    let txt = txt::render(&g);
    assert_eq!(txt, "[ a ] -- go --> [ b ]\n");
    let reparsed =
        text::parse(&txt).unwrap_or_else(|err| panic!("labeled edges must re-parse: {err}\n{txt}"));
    assert!(model_equivalent(&g, &reparsed), "the model must round-trip");
    assert_eq!(
        reparsed.get_attr(Scope::Edge(0), "label"),
        Some("go"),
        "the label must survive the round-trip"
    );
}

/// Strip every leading `# oracle: ` pin-header line so only the oracle
/// payload bytes remain for byte-identical comparison (same convention as
/// the tb.oracle companions).
pub(crate) fn strip_pin_header(bytes: &[u8]) -> &[u8] {
    let s = std::str::from_utf8(bytes).must("companion must be utf-8 text");
    // Byte offset of the first line that is not an oracle pin-header.
    let first_body = s.lines().position(|l| !l.starts_with("# oracle: "));
    let off: usize = s
        .lines()
        .take(first_body.unwrap_or(usize::MAX))
        .map(|l| l.len() + 1)
        .sum();
    &s.as_bytes()[off.min(s.len())..]
}

/// A collection sorted by its natural order.
fn sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort();
    v
}

/// A collection sorted by `cmp`.
fn sorted_by<T>(mut v: Vec<T>, cmp: impl Fn(&T, &T) -> std::cmp::Ordering) -> Vec<T> {
    v.sort_by(cmp);
    v
}

/// Build a `Graph` by applying `f` to a fresh default graph — keeps the
/// call sites' bindings immutable.
fn built(f: impl FnOnce(&mut Graph)) -> Graph {
    let mut g = Graph::default();
    f(&mut g);
    g
}
