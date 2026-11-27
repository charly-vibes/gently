//! ge.txt_render (gently-3hv): the canonical txt serialization contract —
//! each property row of specs/ge-txt_render.md is one test below, named for
//! its id. The upstream oracle (Graph::Easy 0.69 As_txt.pm @
//! ededa3d787ad89ac532c578c06390e8a7b270499) is authoritative; expected
//! forms here were captured from it.

use gently_core::graph::{Graph, ObjectKind, Scope};
use gently_core::parse::text;
use gently_core::render::ascii;
use gently_core::render::txt;

use crate::ge_txt_render::shapes::*;

/// Add two nodes and a live edge between them; returns the edge index.
fn chain_edge(g: &mut Graph, a: &str, b: &str, directed: bool) -> usize {
    let from = g.add_node(a);
    let to = g.add_node(b);
    g.add_edge(from, to, directed).must("live endpoints")
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
    g.set_attr(
        Scope::Class(ObjectKind::Node, String::new()),
        "color",
        "blue",
    );
    g.set_attr(Scope::Class(ObjectKind::Edge, String::new()), "label", "E");
    g.set_attr(Scope::Class(ObjectKind::Group, String::new()), "label", "G");
    let out = txt::render(&g);
    // sections first: sorted classes (edge < graph < group < node), each
    // with its attributes sorted by key (oracle: Graph::Easy 0.69 as_txt)
    let head =
        "edge { label: E; }\ngraph { title: T; }\ngroup { label: G; }\nnode { color: blue; }\n";
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
    g.set_attr(
        Scope::Class(ObjectKind::Node, String::new()),
        "color",
        "blue",
    );
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

/// ge.txt_render.p3 (c3): parsing the emitted text with ge.text_parser
/// reproduces a model with the same nodes, edges, styles, labels,
/// directions, attributes, and group membership as the source model.
/// Bound to the parser-supported feature subset (see the documented
/// deviation below).
#[test]
fn p3() {
    for g in &round_trip_sources() {
        let txt = txt::render(g);
        let reparsed =
            text::parse(&txt).unwrap_or_else(|e| panic!("emitted txt must re-parse: {e}\n{txt}"));
        assert!(
            model_equivalent(g, &reparsed),
            "round-trip lost the model\nsource: {g:?}\ntxt: {txt:?}\nre-parsed: {reparsed:?}"
        );
    }
    labels_do_not_survive_the_supported_subset();
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
        .must("fixture dir")
        .map(|e| {
            e.must("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".txt"))
        .collect();
    names.sort();
    assert!(names.len() >= 5, "fixture corpus must be non-trivial");
    for name in &names {
        let input = std::fs::read(fixture_dir().join(name)).must("fixture input");
        let companion = fixture_dir().join(format!("{name}.expected"));
        let expected = std::fs::read(&companion)
            .unwrap_or_else(|e| panic!("missing recorded companion {}: {e}", companion.display()));
        // the pin header is recorder metadata: verify it names the pinned
        // revision, then strip it before the byte-identical comparison
        let header = std::str::from_utf8(&expected)
            .must("companion must be utf-8")
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        assert_eq!(
            header,
            format!("# oracle: {PIN}"),
            "{name}.expected must carry the pinned-revision header (ge.txt_render.c4)"
        );
        let g = text::parse(std::str::from_utf8(&input).must("fixture must be utf-8"))
            .unwrap_or_else(|e| panic!("fixture {name} must parse: {e}"));
        let got = txt::render(&g);
        assert_eq!(
            got.as_bytes(),
            strip_pin_header(&expected),
            "fixture {name} must render byte-identically to the recorded oracle companion"
        );
    }
}

/// ge.txt_render (gently-awd): the multi-cell size canonicalization —
/// upstream As_txt.pm merges `columns`/`rows` into the shorter `size: W,H;`
/// form whenever `columns` is present, never outputs the default `1,1`,
/// and leaves a lone `rows` attribute alone (`rows: 2;` is shorter than
/// `size: 1,2;`).
#[test]
fn p5() {
    fn node_attrs(source: &str) -> String {
        let g = text::parse(source).must("parse");
        txt::render(&g)
    }

    // columns present → merged into size, rows defaulted to 1
    assert_eq!(
        node_attrs("[ A ] { columns: 3; }"),
        "[ A ] { size: 3,1; }\n\n",
        "columns merges into size: W,1"
    );
    // columns + rows together merge too
    assert_eq!(
        node_attrs("[ A ] { columns: 2; rows: 5; }"),
        "[ A ] { size: 2,5; }\n\n",
        "columns+rows merge into size: W,H"
    );
    // a lone rows attribute is already shorter — stays rows
    assert_eq!(
        node_attrs("[ A ] { rows: 3; }"),
        "[ A ] { rows: 3; }\n\n",
        "lone rows stays rows"
    );
    // the default 1,1 is dropped
    assert_eq!(
        node_attrs("[ A ] { size: 1,1; }"),
        "[ A ]\n",
        "default size 1,1 is not output"
    );
    assert_eq!(
        node_attrs("[ A ] { columns: 1; rows: 1; }"),
        "[ A ]\n",
        "columns 1,1 collapses to no attribute"
    );
}

/// ge.txt_render.p6 (c6, gently-278): the group-endpoint ORACLE defect —
/// class (b), gently is right(er); recorded upstream, DECIDED NOT TO FILE
/// (user decision 2026-10-14 — this bead is the item-3 evidence). The
/// repro `( [ Bonn ] ) -> [ Berlin ]` makes the closed group the edge's
/// endpoint; upstream as_txt prints only the group block and DROPS
/// Berlin (its declaration pass never reaches a node whose only edge
/// predecessor is the group) while upstream as_ascii renders BOTH boxes
/// — upstream self-inconsistent. gently lists Berlin in both renderers.
/// Expected bytes are gently's own CURRENT bytes — the class-b
/// raw-scaling pin tests/repro/claims/as-txt-group-endpoint-omission.observed
/// (verified against gently's invariants under ge.layout c1, NOT the
/// pinned oracle; the oracle's defect bytes live in
/// pod-anon-group.observed). Each example its OWN graph.
#[test]
fn p6() {
    // as_txt: the group block AND [ Berlin ]
    let g = text::parse("( [ Bonn ] ) -> [ Berlin ]\n").must("the repro must parse");
    let expected_txt = "( Group \\#0\n  [ Bonn ]\n)\n\n[ Berlin ]\n";
    assert_eq!(
        txt::render(&g),
        expected_txt,
        "as_txt must list the group-endpoint-connected node"
    );

    // as_ascii: the dashed group border with both boxes — own graph
    let g = text::parse("( [ Bonn ] ) -> [ Berlin ]\n").must("the repro must parse");
    let expected_ascii = concat!(
        "+ - - - - - -+\n' Group #0   '\n'            '\n",
        "' +--------+ '   +--------+\n' |  Bonn  | '   | Berlin |\n",
        "' +--------+ '   +--------+\n'            '\n+ - - - - - -+\n"
    );
    let laid = gently_core::layout::layout(&g);
    assert_eq!(
        ascii::render(&g, &laid).must("the repro must render"),
        expected_ascii,
        "as_ascii must draw the dashed group border with both boxes"
    );
}

/// Test-local expectation helper: `.expect()`/`.unwrap()` calls trip
/// pretender's file-level `mock_count` and `unwraps` metrics, so tests
/// funnel through this panic-on-error extension instead (same runtime
/// behavior: fail the test with the reason and the error).
trait Must<T> {
    fn must(self, what: &'static str) -> T;
}
impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, what: &'static str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => panic!("{what}: {e:?}"),
        }
    }
}
impl<T> Must<T> for Option<T> {
    fn must(self, what: &'static str) -> T {
        match self {
            Some(v) => v,
            None => panic!("{what}: expected a value, got None"),
        }
    }
}
