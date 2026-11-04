use super::*;

/// ge.ascii_render.p2 (c2), rx-dependence: the undirected run shifts with
/// the gap's absolute x — chain gap 2 renders `..-.`, gap 1 renders
/// `-..-` (edge-style-short-runs.observed, upstream _draw_hor ofs rule).
#[test]
fn p2_undirected_dot_dot_dash_shifts_with_rx() {
    let mut g = Graph::default();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    let e1 = g.add_edge(a, b, true).must("live endpoints");
    let e2 = g.add_edge(b, c, false).must("live endpoints");
    g.set_attr(Scope::Edge(e1), "style", "solid");
    g.set_attr(Scope::Edge(e2), "style", "dot-dot-dash");
    g.edges[e2].arrows.end = false;
    let art = ascii::render(&g, &layout::layout(&g)).must("styled edges must render");
    assert_eq!(
        art,
        "+---+     +---+      +---+\n| a | --> | b | ..-. | c |\n+---+     +---+      +---+\n"
    );
}

/// ge.ascii_render.p2 (c2), gently-7qz: the straight bidirectional run —
/// pod-bidir.observed: the '<->' edge renders ' <--> ' (BOTH arrowheads,
/// upstream _draw_hor END_W/END_E) in the widened 6-column gap (upstream
/// _correct_size w++ for BD_HOR cells).
#[test]
fn p2_bidirectional_straight_run() {
    let g = text::parse("[ Bonn ] <--> [ Berlin ]\n").must("bidirectional edge must parse");
    let art = ascii::render(&g, &layout::layout(&g)).must("bidirectional edge must render");
    assert_eq!(
        art,
        "+------+      +--------+\n| Bonn | <--> | Berlin |\n+------+      +--------+\n"
    );
}

/// ge.ascii_render.p2 (c2), gently-7qz: the start arrowhead of a BENT
/// bidirectional edge — pod-bidir-parallel.observed: the straight edge
/// renders ' <--> ' inline while the bent edge carries a 'v' start
/// arrowhead at the source's north attachment mirroring the 'v' end
/// arrowhead at the target (upstream Node _draw_hor END_* arrow rule).
#[test]
fn p2_bidirectional_parallel_bend_start_arrow() {
    let g = text::parse("[ A ] <--> [ B ]\n[ A ] <--> [ B ]\n").must("parallel edges must parse");
    let art =
        ascii::render(&g, &layout::layout(&g)).must("parallel bidirectional edges must render");
    assert_eq!(
        art,
        "\n  +----------+\n  v          v\n+---+      +---+\n| A | <--> | B |\n+---+      +---+\n"
    );
}

/// ge.ascii_render.p2/p4 (c2/c4), gently-7qz: the labeled bidirectional
/// straight run — pod-bidir-label-straight.observed: the gap widens to 11
/// (label 5 + 5 + 1 for the BD cell, upstream _correct_size), the run is
/// ' <-------> ' with both heads, and the label hosts at cell-x 2
/// (centered in [2, w-3), upstream _insert_label).
#[test]
fn p2_bidirectional_labelled_straight() {
    let g =
        text::parse("[ A ] <-- label --> [ B ]\n").must("labeled bidirectional edge must parse");
    let art = ascii::render(&g, &layout::layout(&g)).must("labeled bidirectional edge must render");
    assert_eq!(
        art,
        "+---+  label    +---+\n| A | <-------> | B |\n+---+           +---+\n"
    );
}

/// ge.ascii_render.p2/p4 (c2/c4), gently-bpq: the undirected labeled run
/// carries NO arrowhead — pod-label-attr.observed: the oracle renders the
/// fill pattern only ('------------'); gently's spurious '>' at the
/// target end is the bug.
#[test]
fn p2_undirected_labelled_run_has_no_head() {
    let g = text::parse("[ AB ] -- { label: edgelabel; } [ CD ]\n")
        .must("labeled undirected edge must parse");
    let art = ascii::render(&g, &layout::layout(&g)).must("labeled undirected edge must render");
    assert_eq!(
        art,
        "+----+  edgelabel   +----+\n| AB | ------------ | CD |\n+----+              +----+\n"
    );
}
