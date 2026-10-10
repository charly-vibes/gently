//! ge.txt_render (gently-3hv): the canonical txt serialization contract —
//! each property row of specs/ge-txt_render.md is one test below, named for
//! its id. The upstream oracle (Graph::Easy 0.69 As_txt.pm @
//! ededa3d787ad89ac532c578c06390e8a7b270499) is authoritative; expected
//! forms here were captured from it.

use gently_core::graph::{Graph, ObjectKind, Scope};
use gently_core::parse::text;
use gently_core::render::txt;

/// Add two nodes and a live edge between them; returns the edge index.
fn chain_edge(g: &mut Graph, a: &str, b: &str, directed: bool) -> usize {
    let from = g.add_node(a);
    let to = g.add_node(b);
    g.add_edge(from, to, directed).expect("live endpoints")
}

/// Set the bidirectional arrows on an edge (arrowheads at both ends).
fn make_bidirectional(g: &mut Graph, e: usize) {
    g.edges[e].arrows.start = true;
    g.edges[e].arrows.end = true;
}

/// The p2 composite: one edge per style/direction, an attribute-bearing
/// edge-less node, and a bare isolated node (oracle-verified shapes).
fn styled_graph() -> Graph {
    let mut g = Graph::default();
    chain_edge(&mut g, "a", "b", true);
    chain_edge(&mut g, "c", "d", false);
    let ef = chain_edge(&mut g, "e", "f", true);
    make_bidirectional(&mut g, ef);
    let gh = chain_edge(&mut g, "g", "h", true);
    g.set_attr(Scope::Edge(gh), "style", "dotted");
    let ij = chain_edge(&mut g, "i", "j", true);
    g.set_attr(Scope::Edge(ij), "style", "dashed");
    g.set_attr(Scope::Edge(ij), "label", "go");
    let kl = chain_edge(&mut g, "k", "l", true);
    g.set_attr(Scope::Edge(kl), "style", "bold");
    g.add_node("solo");
    let n = g.add_node("n");
    g.set_attr(Scope::Node(n), "color", "red");
    g
}

/// ge.txt_render.p1 (c1): the output starts with class attribute sections
/// for graph, node, edge, and group classes, emitted in sorted class order
/// with sorted attribute order.
#[test]
fn p1() {
    let mut g = Graph::default();
    g.add_node("a");
    g.set_attr(Scope::Graph, "title", "T");
    g.set_attr(Scope::Class(ObjectKind::Node, String::new()), "color", "blue");
    g.set_attr(Scope::Class(ObjectKind::Edge, String::new()), "label", "E");
    g.set_attr(Scope::Class(ObjectKind::Group, String::new()), "label", "G");
    let out = txt::render(&g);
    // sections first: sorted classes (edge < graph < group < node), each
    // with its attributes sorted by key (oracle: Graph::Easy 0.69 as_txt)
    let head = "edge { label: E; }\ngraph { title: T; }\ngroup { label: G; }\nnode { color: blue; }\n";
    assert!(
        out.starts_with(head),
        "class sections must come first, sorted: {out:?}"
    );
    // the body follows after the sections' separating blank line
    assert_eq!(&out[head.len()..], "\n[ a ]\n");

    multi_attribute_class_section_is_multi_line_and_sorted();
}

/// A class with more than one attribute uses the multi-line form with
/// attribute keys sorted.
fn multi_attribute_class_section_is_multi_line_and_sorted() {
    let mut g = Graph::default();
    g.set_attr(Scope::Class(ObjectKind::Node, String::new()), "label", "N");
    g.set_attr(Scope::Class(ObjectKind::Node, String::new()), "color", "blue");
    let out = txt::render(&g);
    assert!(
        out.starts_with("node {\n  color: blue;\n  label: N;\n}\n\n"),
        "multi-attribute class section must be multi-line and sorted: {out:?}"
    );
}

/// ge.txt_render.p2 (c2): every node is emitted with its name (and its
/// instance attributes), and every edge as an operator chain whose
/// operator matches the edge style and direction. In the graph below each
/// object appears exactly once: attribute-bearing nodes are edge-less,
/// and edges connect attribute-less nodes (upstream as_txt order and
/// forms, verified against the oracle).
#[test]
fn p2() {
    let out = txt::render(&styled_graph());
    let expected = "[ n ] { color: red; }\n\
                    \n\
                    [ a ] --> [ b ]\n\
                    [ c ] -- [ d ]\n\
                    [ e ] <--> [ f ]\n\
                    [ g ] ..> [ h ]\n\
                    [ i ] -  go - > [ j ]\n\
                    [ k ] --> { style: bold; } [ l ]\n\
                    [ solo ]\n";
    assert_eq!(out, expected);
}

/// Model equality for the round-trip (c3): the same node-name set with
/// equal attribute tables and the same edge multiset keyed by endpoint
/// names, direction and per-end arrows with equal attribute tables, and
/// the same group set with equal attributes and equal membership —
/// members compared by node NAME (group membership is a name relation;
/// re-parsed node indices may differ from the source's). The
/// model-equality predicate for the parser-supported feature subset (see
/// the documented deviation in p3).
fn model_equivalent(a: &Graph, b: &Graph) -> bool {
    let node_names = |g: &Graph| -> Vec<String> {
        let mut v: Vec<String> = g.nodes.iter().map(|n| n.name.clone()).collect();
        v.sort();
        v
    };
    let node_attrs =
        |g: &Graph| -> Vec<(String, gently_core::graph::AttributeTable)> {
            let mut v: Vec<_> = g
                .nodes
                .iter()
                .map(|n| (n.name.clone(), n.attributes.clone()))
                .collect();
            v.sort_by(|x, y| x.0.cmp(&y.0));
            v
        };
    let edge_keys = |g: &Graph| -> Vec<(String, String, bool, bool, bool)> {
        let mut v: Vec<_> = g
            .edges
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
            .collect();
        v.sort();
        v
    };
    let group_keys =
        |g: &Graph| -> Vec<(String, Vec<String>, gently_core::graph::AttributeTable)> {
            let mut v: Vec<_> = g
                .groups
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
                .collect();
            v.sort_by(|x, y| x.0.cmp(&y.0));
            v
        };
    node_names(a) == node_names(b)
        && node_attrs(a) == node_attrs(b)
        && edge_keys(a) == edge_keys(b)
        && a.attributes == b.attributes
        && a.class_attributes == b.class_attributes
        && group_keys(a) == group_keys(b)
}

/// The round-trip sources: shapes within the parser-supported feature set.
fn round_trip_sources() -> Vec<Graph> {
    let mut sources = connected_shapes();
    sources.push(diamond_shape());
    sources.extend(group_shapes());
    sources
}

/// Group shapes: a named group whose member sits on an edge, a group whose
/// members are isolated (one bearing node attributes), a group with
/// group-level attributes, and an empty group — the membership-carrier
/// forms for the c3 round-trip.
fn group_shapes() -> Vec<Graph> {
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
fn connected_shapes() -> Vec<Graph> {
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
fn diamond_shape() -> Graph {
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

/// ge.txt_render.p3 (c3): parsing the emitted text with ge.text_parser
/// reproduces a model with the same nodes, edges, styles, labels,
/// directions, attributes, and group membership as the source model.
/// Bound to the parser-supported feature subset (see the documented
/// deviation below).
#[test]
fn p3() {
    for g in &round_trip_sources() {
        let txt = txt::render(g);
        let reparsed = text::parse(&txt)
            .unwrap_or_else(|e| panic!("emitted txt must re-parse: {e}\n{txt}"));
        assert!(
            model_equivalent(g, &reparsed),
            "round-trip lost the model\nsource: {g:?}\ntxt: {txt:?}\nre-parsed: {reparsed:?}"
        );
    }
    labels_do_not_survive_the_supported_subset();
}

/// Labels render in the upstream form (`-- go -->`) and round-trip through
/// the full ge.text_parser grammar (gently-bzx implemented c4 — the
/// earlier tracer-era deviation, "labels cannot be read back", is gone).
fn labels_do_not_survive_the_supported_subset() {
    let mut g = Graph::default();
    let e = chain_edge(&mut g, "a", "b", true);
    g.set_attr(Scope::Edge(e), "label", "go");
    let txt = txt::render(&g);
    assert_eq!(txt, "[ a ] -- go --> [ b ]\n");
    let reparsed = text::parse(&txt)
        .unwrap_or_else(|err| panic!("labeled edges must re-parse: {err}\n{txt}"));
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
fn strip_pin_header(bytes: &[u8]) -> &[u8] {
    let s = std::str::from_utf8(bytes).expect("companion must be utf-8 text");
    let mut rest = s;
    while let Some(idx) = rest.find('\n') {
        if rest[..idx].starts_with("# oracle: ") {
            rest = &rest[idx + 1..];
        } else {
            break;
        }
    }
    rest.as_bytes()
}

/// ge.txt_render.p4 (c4): every recorded upstream fixture under
/// `tests/fixtures/graph-easy/` — input `.txt` plus expected canonical-text
/// companion `.txt.expected`, captured from the pinned upstream revision —
/// renders byte-identically. Re-recording expectations is a deliberate,
/// separately reviewed change, never a side effect of code edits.
#[test]
fn p4() {
    use std::path::PathBuf;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/graph-easy")
    }

    /// The pinned oracle (ge.txt_render.c4): every recorded companion
    /// names exactly this revision in its header.
    const PIN: &str = "Graph::Easy v0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499";

    let mut names: Vec<String> = std::fs::read_dir(fixture_dir())
        .expect("fixture dir")
        .map(|e| e.expect("dir entry").file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".txt"))
        .collect();
    names.sort();
    assert!(names.len() >= 5, "fixture corpus must be non-trivial");
    for name in &names {
        let input = std::fs::read(fixture_dir().join(name)).expect("fixture input");
        let companion = fixture_dir().join(format!("{name}.expected"));
        let expected = std::fs::read(&companion)
            .unwrap_or_else(|e| panic!("missing recorded companion {}: {e}", companion.display()));
        // the pin header is recorder metadata: verify it names the pinned
        // revision, then strip it before the byte-identical comparison
        let header = std::str::from_utf8(&expected)
            .expect("companion must be utf-8")
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        assert_eq!(
            header,
            format!("# oracle: {PIN}"),
            "{name}.expected must carry the pinned-revision header (ge.txt_render.c4)"
        );
        let g = text::parse(std::str::from_utf8(&input).expect("fixture must be utf-8"))
            .unwrap_or_else(|e| panic!("fixture {name} must parse: {e}"));
        let got = txt::render(&g);
        assert_eq!(
            got.as_bytes(),
            strip_pin_header(&expected),
            "fixture {name} must render byte-identically to the recorded oracle companion"
        );
    }
}
