//! ge.graph_model (gently-4ht): the shared model contract — each property
//! row of specs/ge-graph_model.md is one test below, named for its id.

use gently_core::graph::{Arrows, Edge, Graph, ObjectKind, Scope};

/// ge.graph_model.p1 (c1): named nodes are unique per graph; duplicate
/// name insertion merges into the existing node (upstream add_node
/// semantics: the existing node is returned, no second node is created).
/// Anonymous nodes are unnamed and cannot be referenced again by name.
#[test]
fn p1() {
    let mut g = Graph::default();
    let a1 = g.add_node("a");
    let a2 = g.add_node("a");
    assert_eq!(a1, a2, "duplicate-name insertion merges into the existing node");
    assert_eq!(g.nodes.len(), 1, "no second node is created");
    let b = g.add_node("b");
    assert_ne!(a1, b);
    assert_eq!(g.node_by_name("a"), Some(a1));
    assert_eq!(g.node_by_name("b"), Some(b));
    // the model never exposes two distinct nodes with the same name
    let mut g2 = Graph::default();
    for name in ["x", "y", "x", "z", "y"] {
        g2.add_node(name);
    }
    let named: Vec<&str> = g2.nodes.iter().map(|n| n.name.as_str()).collect();
    let mut unique: Vec<&str> = named.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(named.len(), unique.len(), "distinct same-name nodes exposed");
    // anonymous nodes: distinct, unnamed, unreferencable by name
    let n1 = g.add_anonymous_node();
    let n2 = g.add_anonymous_node();
    assert_ne!(n1, n2, "each anonymous node is a distinct node");
    assert!(g.nodes[n1].name.is_empty() && g.nodes[n2].name.is_empty());
    assert_eq!(g.node_by_name(""), None, "anonymous nodes cannot be referenced by name");
    assert_eq!(g.node_by_name("never-added"), None);
}

/// ge.graph_model.p2 (c2): every attribute assignment applies to exactly
/// the object or class scope it targets, values stored verbatim (no
/// loss); reads on other scopes are unchanged. Derived border components
/// (style, width, color) are computed at assignment time exactly as
/// upstream Graph::Easy does.
#[test]
fn p2() {
    let mut g = Graph::default();
    let n = g.add_node("a");
    let e = g.add_edge(n, n, true).expect("self-loops are legal");
    let grp = g.add_group("grp");
    // verbatim storage, per scope
    g.set_attr(Scope::Node(n), "label", "A1");
    g.set_attr(Scope::Edge(e), "label", "  bold  ");
    g.set_attr(Scope::Graph, "title", "T");
    g.set_attr(Scope::Group(grp), "label", "G");
    g.set_attr(Scope::Class(ObjectKind::Node, "important".into()), "color", "red");
    assert_eq!(g.get_attr(Scope::Node(n), "label"), Some("A1"), "value stored verbatim");
    assert_eq!(g.get_attr(Scope::Edge(e), "label"), Some("  bold  "), "whitespace kept");
    assert_eq!(g.get_attr(Scope::Graph, "title"), Some("T"));
    assert_eq!(g.get_attr(Scope::Group(grp), "label"), Some("G"));
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Node, "important".into()), "color"),
        Some("red")
    );
    // reads on other scopes are unchanged
    assert_eq!(g.get_attr(Scope::Graph, "label"), None);
    assert_eq!(g.get_attr(Scope::Edge(e), "label"), Some("  bold  "));
    assert_eq!(g.get_attr(Scope::Group(grp), "label"), Some("G"));
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Node, "".into()), "color"),
        None,
        "class scopes are distinct from the base class"
    );
    assert_eq!(g.get_attr(Scope::Class(ObjectKind::Edge, "important".into()), "color"), None);
    // derived border components computed AT ASSIGNMENT TIME, exactly as
    // upstream split_border_attributes (Graph::Easy 0.69)
    g.set_attr(Scope::Node(n), "border", "2px dotted #f0f");
    assert_eq!(g.get_attr(Scope::Node(n), "border"), Some("2px dotted #f0f"), "verbatim");
    assert_eq!(g.get_attr(Scope::Node(n), "border_style"), Some("dotted"));
    assert_eq!(g.get_attr(Scope::Node(n), "border_width"), Some("2"), "digits-only");
    assert_eq!(g.get_attr(Scope::Node(n), "border_color"), Some("#f0f"));
    // defaults and the `0` special case (upstream-faithful)
    let m = g.add_node("m");
    g.set_attr(Scope::Node(m), "border", "red");
    assert_eq!(g.get_attr(Scope::Node(m), "border_style"), Some("solid"), "default style");
    assert_eq!(g.get_attr(Scope::Node(m), "border_width"), Some(""), "no width");
    assert_eq!(g.get_attr(Scope::Node(m), "border_color"), Some("red"));
    g.set_attr(Scope::Node(m), "border", "0");
    assert_eq!(g.get_attr(Scope::Node(m), "border_style"), Some("none"), "0 → none");
    // assignment to one scope leaves reads on other scopes unchanged
    g.set_attr(Scope::Class(ObjectKind::Node, "".into()), "border", "bold blue");
    assert_eq!(g.get_attr(Scope::Node(n), "border_style"), Some("dotted"), "node untouched");
    assert_eq!(
        g.get_attr(Scope::Class(ObjectKind::Node, "".into()), "border_style"),
        Some("bold"),
        "class scope derives its own components"
    );
}

/// ge.graph_model.p3 (c3): every stored edge resolves both endpoints to
/// live nodes; self-loops are legal; no dangling edge ever escapes a
/// mutation — node removal drops incident edges (upstream del_node).
#[test]
fn p3() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let self_loop = g.add_edge(a, a, true).expect("self-loops are legal");
    let ab = g.add_edge(a, b, true).expect("live endpoints");
    let bc = g.add_edge(b, c, true).expect("live endpoints");
    // every stored edge resolves both endpoints to live nodes
    let live = |g: &Graph| {
        g.edges.iter().all(|e| e.from < g.nodes.len() && e.to < g.nodes.len())
    };
    assert!(live(&g));
    // node removal drops incident edges (upstream del_node), nothing dangles
    g.remove_node(b);
    assert!(live(&g), "no dangling edge escapes remove_node");
    assert_eq!(g.edges.len(), 1, "b's incident edges a–b and b–c dropped");
    let surviving = g.edges[0].clone();
    assert_ne!(surviving.from, b);
    assert_ne!(surviving.to, b);
    assert!(
        !g.remove_node(usize::MAX),
        "removing an out-of-range node is a no-op"
    );
    // attempts on dropped (or unknown) nodes never create a dangling edge
    // (after removal the remaining nodes re-index, so the only way to name
    // a dead endpoint is an out-of-range index — rejected, never stored)
    assert_eq!(
        g.add_edge(a, g.nodes.len(), true),
        None,
        "dead endpoint never yields a stored edge"
    );
    assert_eq!(g.add_edge(usize::MAX, a, true), None, "out-of-range source rejected");
    assert_eq!(g.add_edge(c, usize::MAX, true), None, "out-of-range target rejected");
    assert!(live(&g));
    assert_eq!(self_loop, 0);
    assert_ne!(ab, bc);
}

/// ge.graph_model.p4 (c4): each edge is directed or undirected and this
/// direction, together with per-end arrow-head presence, is preserved
/// bit-exactly through every model round-trip.
#[test]
fn p4() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let d = g.add_edge(a, b, true).expect("live");
    let u = g.add_edge(a, b, false).expect("live");
    // construction defaults: directed → arrowhead at the end only
    assert!(g.edges[d].directed);
    assert_eq!(g.edges[d].arrows, Arrows { start: false, end: true });
    assert!(!g.edges[u].directed);
    assert_eq!(g.edges[u].arrows, Arrows { start: false, end: false });
    // per-end arrow-head presence is settable (bidirectional)
    g.edges[d].arrows = Arrows { start: true, end: true };
    // round-trip: clone the whole graph — bit-exact preservation
    let g2 = g.clone();
    assert_eq!(g2.edges[d], g.edges[d], "edge round-trips bit-exactly through clone");
    assert!(g2.edges[d].directed);
    assert!(!g2.edges[u].directed);
    assert_eq!(g2.edges[d].arrows, Arrows { start: true, end: true });
    assert_eq!(g2.edges[u].arrows, Arrows { start: false, end: false });
    // round-trip through the published accessors matches direct inspection
    assert_eq!(g2.edges()[d], g.edges[d]);
    assert_eq!(g2.edges()[u].directed, g.edges[u].directed);
    assert_eq!(g2.edges()[d].arrows.start, g.edges[d].arrows.start);
    assert_eq!(g2.edges()[d].arrows.end, g.edges[d].arrows.end);
    // the three construction shapes are distinct
    let mut g3 = Graph::default();
    g3.add_node("a");
    g3.add_node("b");
    assert_ne!(Edge::directed(0, 1), Edge::undirected(0, 1));
    assert_ne!(Edge::directed(0, 1), Edge::bidirectional(0, 1));
    assert_eq!(Edge::bidirectional(0, 1).arrows, Arrows { start: true, end: true });
    assert_eq!(g3.edges.len(), 0);
}

/// ge.graph_model.p5 (c5): downstream consumers (layout, renderers)
/// read the model through the published accessors/iterators — every
/// consumer-visible traversal and attribute read matches direct model
/// inspection.
#[test]
fn p5() {
    let mut g = Graph::tracer();
    let c = g.add_node("c");
    let e = g.add_edge(1, 2, true).expect("live");
    g.set_attr(Scope::Node(c), "label", "C");
    g.set_attr(Scope::Edge(e), "style", "dashed");
    g.set_attr(Scope::Graph, "title", "T");
    g.set_attr(Scope::Class(ObjectKind::Node, "important".into()), "color", "red");
    // consumer-visible traversal matches direct inspection
    let via_accessors: Vec<&str> = g.nodes().iter().map(|n| n.name.as_str()).collect();
    let direct: Vec<&str> = g.nodes.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(via_accessors, direct);
    assert_eq!(g.nodes().len(), g.nodes.len());
    assert_eq!(g.edges().len(), g.edges.len());
    for (i, edge) in g.edges().iter().enumerate() {
        assert_eq!(edge, &g.edges[i], "edge {i} consumer-visible read matches inspection");
    }
    // consumer-visible attribute reads match direct inspection
    assert_eq!(
        g.get_attr(Scope::Node(c), "label"),
        g.nodes[c].attributes.get("label")
    );
    assert_eq!(g.get_attr(Scope::Edge(e), "style"), g.edges[e].attributes.get("style"));
    assert_eq!(g.get_attr(Scope::Graph, "title"), g.attributes.get("title"));
    let class = Scope::Class(ObjectKind::Node, "important".into());
    assert_eq!(g.get_attr(class, "color"), Some("red"));
    let class_table = &g.class_attributes[0];
    assert_eq!(class_table.0, ObjectKind::Node);
    assert_eq!(class_table.1, "important");
    assert_eq!(class_table.2.get("color"), Some("red"));
}

