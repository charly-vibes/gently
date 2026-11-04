use super::*;

/// ge.dot_parser.p3 (c3): a graph-scope attribute statement `key=value`
/// (quoted or bare value) applies to the model graph, with `rankdir`
/// remapped to `flow` per the oracle's direction map (gently-a0k).
pub(crate) fn graph_scope_attr_statements() {
    for (dir, flow) in [
        ("LR", "east"),
        ("RL", "west"),
        ("TB", "south"),
        ("BT", "north"),
    ] {
        let src = format!("digraph G {{ rankdir={dir} a -> b }}\n");
        let g = dot::parse(&src).unwrap_or_else(|e| panic!("{src:?} must parse: {e}"));
        assert_eq!(g.attributes.get("flow"), Some(flow), "{src:?}");
    }
    // any unrecognized rankdir value falls back to east (oracle `$rankdir->{$dir} || 'east'`)
    for dir in ["XX", "lr"] {
        let src = format!("digraph G {{ rankdir={dir} a -> b }}\n");
        let g = dot::parse(&src).must("must parse");
        assert_eq!(g.attributes.get("flow"), Some("east"), "{src:?}");
    }
    // quoted values unescape exactly once
    let g = dot::parse("digraph G { rankdir=\"LR\" a -> b }\n").must("must parse");
    assert_eq!(g.attributes.get("flow"), Some("east"), "quoted value");
    // applies from inside a bare scope too, and the last statement wins
    let g = dot::parse("digraph G { { rankdir=RL } a -> b }\n").must("must parse");
    assert_eq!(
        g.attributes.get("flow"),
        Some("west"),
        "inside a bare scope"
    );
    let g = dot::parse("digraph G { rankdir=TB; rankdir=BT a }\n").must("must parse");
    assert_eq!(
        g.attributes.get("flow"),
        Some("north"),
        "last statement wins"
    );
}

/// ge.dot_parser.p4 (c4): bare scopes as edge-chain endpoints (gently-2n5,
/// gh-4) under the oracle's node-stack semantics — the stack survives
/// scope boundaries, an edge operator links EVERY node on it, and the
/// statement's edge attributes land on every linked edge.
pub(crate) fn scope_stack_endpoints() {
    // {a b} -> c links BOTH scope nodes (gh-4 input b)
    let g = dot::parse("digraph G { {foo bar} -> baz }\n").must("must parse");
    assert_eq!(edge_pairs(&g), vec![("foo", "baz"), ("bar", "baz")]);

    // gh-4 input a: the pre-declared nodes leak into the scope's stack, so
    // the pinned spurious self-edge fires FIRST (baz->baz), then both scope
    // nodes — and rankdir/title land per c3/c1 (gh-4-nodes-first.observed)
    let g = dot::parse(
        "digraph {\n  rankdir=\"LR\"\n\n  foo\n  bar [label=\"bar-label!\"]\n  baz\n\n  {foo bar} -> baz\n}\n",
    )
    .must("gh-4 input a must parse");
    assert_eq!(g.attributes.get("flow"), Some("east"));
    assert_eq!(g.attributes.get("title"), Some("unnamed"));
    assert_eq!(
        edge_pairs(&g),
        vec![("baz", "baz"), ("foo", "baz"), ("bar", "baz")],
        "gh-4 input a: leaked stack first"
    );
    assert_eq!(
        g.nodes[node(&g, "bar")].attributes.get("label"),
        Some("bar-label!")
    );

    // the endpoint form links every scope node from the left side, and the
    // chain continues through the scope (observed add_edge order)
    let g = dot::parse("digraph G { a -> { b c } -> d }\n").must("must parse");
    assert_eq!(
        edge_pairs(&g),
        vec![("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")]
    );
    // chains continue after a statement-position scope too
    let g = dot::parse("digraph G { {a b} -> c -> d }\n").must("must parse");
    assert_eq!(edge_pairs(&g), vec![("a", "c"), ("b", "c"), ("c", "d")]);
    // a named subgraph as an endpoint links its nodes — no node for its name
    let g = dot::parse("digraph G { subgraph g { x } -> y }\n").must("must parse");
    assert_eq!(edge_pairs(&g), vec![("x", "y")]);
    assert!(
        g.node_by_name("g").is_none(),
        "no node for the subgraph name"
    );
    // statement-end attributes land on every edge the statement linked
    let g = dot::parse("digraph G { {a b} -> c [color=red] }\n").must("must parse");
    assert_eq!(g.edges.len(), 2);
    assert_eq!(g.edges[0].attributes.get("color"), Some("red"));
    assert_eq!(g.edges[1].attributes.get("color"), Some("red"));
}

/// ge.dot_parser.p4 (c4): the stack resets at a plain `}` — a scope with no
/// inner edge links nothing; a nested scope's `}` cuts the leak; an
/// operator with nothing on the stack is the oracle's not-recognized error;
/// an attribute list after a scope is dropped (no operator) or fatal.
pub(crate) fn scope_stack_resets() {
    // no inner edge: nothing links, the stack does not leak out
    let g = dot::parse("digraph G { a -> b; { c } }\n").must("must parse");
    assert_eq!(edge_pairs(&g), vec![("a", "b")]);
    let g = dot::parse("digraph G { a; { c } b -> d }\n").must("must parse");
    assert_eq!(edge_pairs(&g), vec![("b", "d")]);
    // a nested bare scope's plain `}` cuts the leak at its boundary
    let g = dot::parse("digraph G { { a { b } c -> d } }\n").must("must parse");
    assert_eq!(edge_pairs(&g), vec![("c", "d")]);
    // an operator with an empty stack is the oracle's not-recognized error
    for src in [
        "digraph G { { } -> b }\n",
        "digraph G { { a b } [x=1] -> c }\n",
    ] {
        let err = dot::parse(src).expect_err("empty-stack operator must fail");
        assert!(err.line >= 1, "{src:?} must name a line");
        assert!(
            err.message.contains("not recognized"),
            "{src:?}: {}",
            err.message
        );
    }
    // an attribute list after a scope with no operator is dropped
    let g = dot::parse("digraph G { a -> { b } [x=1] }\n").must("must parse");
    assert_eq!(edge_pairs(&g), vec![("a", "b")]);
}

/// HTML-like table label autosplits into one part per <TD> cell.
pub(crate) fn html_table_autosplits() {
    let g = dot::parse(
        "digraph G { a [label=<<TABLE><TR><TD>one</TD><TD>two</TD></TR></TABLE>>] a -> b }\n",
    )
    .must("must parse");
    node(&g, "a.0");
    node(&g, "a.1");
    assert_eq!(g.edges[0].from, node(&g, "a.0"));
}

/// The failing constructs: typed errors naming a line and quoting the
/// offending input (tokenizing) or the `base:port` and edge id (ports).
pub(crate) fn failing_constructs_are_typed_errors() {
    // malformed HTML-like label (non-tag text inside the label)
    let err = dot::parse("digraph G { a [label=<<table>...</table>>] }\n")
        .expect_err("malformed HTML-like label must fail");
    assert!(err.line >= 1);
    assert!(
        err.message.contains("<<table>...</table>>"),
        "error quotes the input: {}",
        err.message
    );
    // unresolvable port reference names base:port and the edge id
    for src in [
        "digraph G { b:p1 -> c }\n",
        "digraph G { a [shape=record label=\"A|B\"] a:zz -> b }\n",
        "digraph G { b:nw -> c }\n",
    ] {
        let err = dot::parse(src).expect_err("port without a matching part must fail");
        let head = err.message.split(" on edge").next().unwrap_or("");
        assert!(
            head.starts_with("Cannot find autosplit node for ") && head.contains(':'),
            "oracle error text for {src:?}, got: {}",
            err.message
        );
        assert!(
            err.message.contains(" on edge "),
            "the edge id is named: {}",
            err.message
        );
    }
}
