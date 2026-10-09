//! Scenario tests, grouped per capability.
//!
//! Filters used by espectacular contracts (`ah check --run-tests`) match
//! these module paths: `scenarios::<cap>::<pid>`.

mod tb {
    /// Walking-skeleton smoke test (epic gently-2po): the workspace links,
    /// gently-core is loadable, and the tracer shape plumbing is reachable.
    /// Deepened by gently-2po.7/.8/.9/.10.
    #[test]
    fn workspace_smoke() {
        assert!(gently_core::ready());
    }

    /// tb.txt-render (gently-2po.7): minimal canonical txt serialization of
    /// the tracer graph shape, byte-identical to the pinned oracle.
    ///
    /// Oracle (Graph::Easy v0.69 @ ededa3d7, recorded 2026-10-08):
    ///   perl -IGraph-Easy-0.69/lib -MGraph::Easy \
    ///     -e 'my $g = Graph::Easy->new; $g->add_edge("a","b"); print $g->as_txt'
    ///   → "[ a ] --> [ b ]\n"
    /// Deepened by gently-3hv (capability contracts p1–p4).
    mod txt_render {
        use gently_core::{graph::Graph, render::txt};

        #[test]
        fn tracer_shape_matches_oracle() {
            let g = Graph::tracer();
            assert_eq!(txt::render(&g), "[ a ] --> [ b ]\n");
        }
    }

    /// tb.layout (gently-2po.8): one deterministic layout path for the
    /// tracer graph — ranks grow eastward (a west of b, same row), node
    /// cells never overlap and stay inside the grid, and the edge routes
    /// orthogonally from a's east side to b's west side without entering a
    /// node cell. Oracle shape (Graph::Easy v0.69 @ ededa3d7, as_ascii):
    /// `+---+     +---+` / `| a | --> | b |` / `+---+     +---+` —
    /// a west of b, same row, one-cell gap. Deepened by gently-4ht /
    /// gently-4nx (capability contracts p1–p4).
    mod layout {
        use gently_core::{graph::Graph, layout};

        /// ge.layout.c1: the layout is a pure function — identical inputs
        /// produce identical output.
        #[test]
        fn tracer_layout_is_deterministic() {
            let g = Graph::tracer();
            assert_eq!(layout::layout(&g), layout::layout(&g));
        }

        /// Tracer semantics: rank(a) = 0 < rank(b) = 1, ranks growing
        /// eastward, both nodes on the same row.
        #[test]
        fn ranks_grow_eastward_on_one_row() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            let a = l.node_cells[0];
            let b = l.node_cells[1];
            assert!(a.0 < b.0, "a must sit west of b (rank(a)=0 < rank(b)=1)");
            assert_eq!(a.1, b.1, "tracer nodes share one row");
        }

        /// ge.layout.c2: no two node cells overlap; every node fully
        /// inside the grid.
        #[test]
        fn node_cells_do_not_overlap_and_stay_in_grid() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            assert_ne!(l.node_cells[0], l.node_cells[1]);
            for &(x, y) in &l.node_cells {
                assert!(x < l.width && y < l.height, "node ({x},{y}) outside grid");
            }
        }

        /// ge.layout.c4 (tracer slice): the edge path is orthogonal and
        /// grid-connected, touches neither node cell, and runs from a's
        /// east side to b's west side through the gap column(s).
        #[test]
        fn edge_path_is_orthogonal_and_avoids_node_cells() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            let (a, b) = (l.node_cells[0], l.node_cells[1]);
            let path = &l.edge_paths[0];
            assert!(!path.is_empty(), "edge must be routed");
            for w in path.windows(2) {
                let (p, q) = (w[0], w[1]);
                assert!(p.0 == q.0 || p.1 == q.1, "segment {p:?}->{q:?} not orthogonal");
                let dx = (p.0 as isize - q.0 as isize).abs();
                let dy = (p.1 as isize - q.1 as isize).abs();
                assert!(dx + dy == 1, "cells {p:?}->{q:?} not grid-adjacent");
            }
            for &c in path {
                assert_ne!(c, a, "path enters a's cell");
                assert_ne!(c, b, "path enters b's cell");
            }
            assert_eq!(path[0], (a.0 + 1, a.1), "path must start at a's east side");
            assert_eq!(*path.last().unwrap(), (b.0 - 1, b.1), "path must end at b's west side");
        }
    }

    /// tb.cli (gently-2po.9): the minimal ge.text_parser slice — the tracer
    /// text form `[ a ] --> [ b ]` (or `->`, flexible whitespace) parses to
    /// two interned nodes and one edge; junk lines are typed parse errors,
    /// never panics. Deepened by gently-bzx (full ge.text_parser).
    mod parse_text {
        use gently_core::parse::text::{self, ParseError};

        #[test]
        fn tracer_edge_parses_to_interned_shape() {
            let g = text::parse("[ a ] --> [ b ]\n").expect("tracer line must parse");
            assert_eq!(g.nodes.len(), 2);
            assert_eq!(g.nodes[0].name, "a");
            assert_eq!(g.nodes[1].name, "b");
            assert_eq!(
                g.edges,
                vec![gently_core::graph::Edge::directed(0, 1)],
                "tracer edges are directed"
            );
        }

        /// Flexible whitespace and the `->` spelling are accepted.
        #[test]
        fn arrow_spelling_and_whitespace_flexibility() {
            let g = text::parse("[a]->[b]\n").expect("compact form must parse");
            assert_eq!(g.edges.len(), 1);
            let g = text::parse("  [  a  ]   -->   [  b  ]  \n").expect("spaced form must parse");
            assert_eq!(g.edges.len(), 1);
            assert_eq!(g.nodes[0].name, "a");
        }

        /// Shared names intern to one node; bare node lines add nodes.
        #[test]
        fn names_intern_and_bare_nodes_parse() {
            let g = text::parse("[ x ]\n[ x ] --> [ y ]\n").expect("must parse");
            assert_eq!(g.nodes.len(), 2, "x must intern to a single node");
            assert_eq!(g.nodes[0].name, "x");
            assert_eq!(g.nodes[1].name, "y");
            assert_eq!(g.edges.len(), 1);
        }

        /// Junk input is a typed error naming the offending line — no panic.
        #[test]
        fn junk_lines_are_typed_errors() {
            let err = text::parse("this is not graph text\n").expect_err("junk must error");
            assert_eq!(err.line, 1);
            assert!(text::parse("[ a ] -->\n").is_err());
            assert!(text::parse("[ ] --> [ b ]\n").is_err(), "empty node name errors");
            assert!(text::parse("[ a ] junk [ b ]\n").is_err());
            assert!(text::parse("]]][[[\n").is_err(), "no panic on bracket soup");
            // empty input is a valid (empty) graph
            assert_eq!(text::parse("").unwrap(), gently_core::graph::Graph::default());
        }

        /// Line numbers in errors are 1-based and point at the offender.
        #[test]
        fn error_line_numbers_are_one_based() {
            let err = text::parse("[ a ] --> [ b ]\nnope\n").expect_err("second line is junk");
            assert_eq!(err, ParseError { line: 2, message: err.message.clone() });
        }
    }

    /// tb.cli (gently-2po.9): the minimal ascii renderer — the tracer layout
    /// draws node boxes (`+---+` / `| a |` / `+---+`), a 5-char gap column
    /// carrying ` --> ` on the middle row, and one trailing newline.
    /// Oracle (Graph::Easy v0.69 @ ededa3d7, `add_edge("a","b"); as_ascii`):
    /// `+---+     +---+` / `| a | --> | b |` / `+---+     +---+`.
    /// Deepened by the ge-ascii_render capability slices.
    mod ascii_render {
        use gently_core::{graph::Graph, layout, render::ascii};

        #[test]
        fn tracer_layout_renders_oracle_bytes() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            let art = ascii::render(&g, &l).expect("tracer layout must render");
            assert_eq!(art, "+---+     +---+\n| a | --> | b |\n+---+     +---+\n");
        }

        /// Deterministic: identical inputs render identical bytes.
        #[test]
        fn rendering_is_deterministic() {
            let g = Graph::tracer();
            let l = layout::layout(&g);
            assert_eq!(ascii::render(&g, &l), ascii::render(&g, &l));
        }

        /// Rendering is a pure function of the layout — no panic on the
        /// empty graph, and unsupported geometry is a typed error.
        #[test]
        fn empty_graph_renders_empty() {
            let g = Graph::default();
            let l = layout::layout(&g);
            assert_eq!(ascii::render(&g, &l).unwrap(), "");
        }
    }

    /// tb.oracle (gently-2po.10): the first differential fixture proves the
    /// tracer end to end. `tests/fixtures/graph-easy/tracer.txt` is the
    /// input; each companion (`tracer.txt.expected` from `as_txt`,
    /// `tracer.ascii.expected` from `as_ascii`) is recorded from the pinned
    /// oracle — Graph::Easy v0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499
    /// — and carries a leading `# oracle: ` pin header naming that pin
    /// (ge.oracle.c1/c2). These tests re-render the fixture through
    /// gently's pipeline and compare byte-identically against the recorded
    /// bytes (ge.oracle.c3); `just oracle-record` / `just oracle-verify`
    /// run the same correspondence against live perl locally (c4/c5).
    mod oracle {
        use gently_core::{layout, parse::text, render::ascii, render::txt};
        use std::path::PathBuf;

        /// The pinned oracle (ge.oracle.c1): the (version, commit) pair
        /// every recorded companion's pin header must name.
        const PIN: (&str, &str) = ("0.69", "ededa3d787ad89ac532c578c06390e8a7b270499");

        fn fixture_dir() -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/graph-easy")
        }

        /// Read the leading `# oracle: ` pin-header line off a recorded
        /// companion: returns the named (version, commit) when the header
        /// is present and well-formed, None when it is missing or
        /// malformed. A companion may carry several leading header lines;
        /// only the first is the pin.
        fn pin_header_of(bytes: &[u8]) -> Option<(String, String)> {
            let first = std::str::from_utf8(bytes).ok()?.lines().next()?;
            let rest = first.strip_prefix("# oracle: Graph::Easy v")?;
            let (version, commit) = rest.split_once(" @ ")?;
            Some((version.to_string(), commit.trim().to_string()))
        }

        /// Strip every leading `# oracle: ` header line so only the oracle
        /// payload bytes remain for byte-identical comparison.
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

        /// The verifier-side pin check (ge.oracle.c4): a companion is
        /// stale when its header is missing, malformed, or names a pin
        /// other than the pinned revision.
        fn pin_is_stale(bytes: &[u8]) -> bool {
            pin_header_of(bytes) != Some((PIN.0.to_string(), PIN.1.to_string()))
        }

        /// ge.oracle.c3: compare the rendered bytes to the recorded
        /// companion byte-identically; on any difference, describe it
        /// naming the input, the format, and the first differing line.
        fn mismatch(input: &str, format: &str, got: &[u8], want: &[u8]) -> Option<String> {
            if got == want {
                return None;
            }
            let common = got.iter().zip(want).take_while(|(a, b)| a == b).count();
            let n = got[..common].iter().filter(|&&b| b == b'\n').count() + 1;
            let line = |b: &[u8]| {
                String::from_utf8_lossy(
                    b.split(|&c| c == b'\n').nth(n - 1).unwrap_or_default(),
                )
                .into_owned()
            };
            Some(format!(
                "oracle mismatch: input={input} format={format} \
                 first differing line {n}: got {:?}, recorded {:?} — spec ge.oracle.c3",
                line(got),
                line(want)
            ))
        }

        /// ge.oracle.c3: the tracer fixture re-renders through gently's
        /// parse -> render pipeline byte-identically to the recorded
        /// `as_txt` companion.
        #[test]
        fn tracer_fixture_matches_recorded_oracle_txt() {
            let input = std::fs::read(fixture_dir().join("tracer.txt")).expect("fixture input");
            let expected =
                std::fs::read(fixture_dir().join("tracer.txt.expected")).expect("recorded companion");
            let g = text::parse(std::str::from_utf8(&input).expect("fixture must be utf-8"))
                .expect("fixture input must parse");
            let m = mismatch(
                "tracer.txt",
                "txt",
                txt::render(&g).as_bytes(),
                strip_pin_header(&expected),
            );
            assert!(m.is_none(), "{m:?}");
        }

        /// ge.oracle.c3: the tracer fixture re-renders through the full
        /// pipeline (parse -> layout -> ascii render — the same path the
        /// `gently` binary drives) byte-identically to the recorded
        /// `as_ascii` companion.
        #[test]
        fn tracer_fixture_matches_recorded_oracle_ascii() {
            let input = std::fs::read(fixture_dir().join("tracer.txt")).expect("fixture input");
            let expected = std::fs::read(fixture_dir().join("tracer.ascii.expected"))
                .expect("recorded companion");
            let g = text::parse(std::str::from_utf8(&input).expect("fixture must be utf-8"))
                .expect("fixture input must parse");
            let l = layout::layout(&g);
            let art = ascii::render(&g, &l).expect("fixture layout must render");
            let m = mismatch(
                "tracer.txt",
                "ascii",
                art.as_bytes(),
                strip_pin_header(&expected),
            );
            assert!(m.is_none(), "{m:?}");
        }

        /// ge.oracle.c4: the pin-header gate — the real companions carry
        /// exactly the pinned header, and a foreign version, a foreign
        /// commit, or a missing header is rejected as stale.
        #[test]
        fn stale_pin_header_is_rejected() {
            for name in ["tracer.txt.expected", "tracer.ascii.expected"] {
                let bytes = std::fs::read(fixture_dir().join(name)).expect("recorded companion");
                assert!(!pin_is_stale(&bytes), "{name} must carry the pinned header");
            }
            let foreign_version =
                format!("# oracle: Graph::Easy v0.76 @ {}\n", PIN.1);
            let foreign_commit =
                format!("# oracle: Graph::Easy v{} @ 0123456789abcdef0123456789abcdef01234567\n", PIN.0);
            assert!(pin_is_stale(foreign_version.as_bytes()), "foreign version is stale");
            assert!(pin_is_stale(foreign_commit.as_bytes()), "foreign commit is stale");
            assert!(pin_is_stale(b"[ a ] --> [ b ]\n"), "missing header is stale");
        }
    }
}

/// ge.graph_model (gently-4ht): the shared model contract, one test per
/// property row of specs/ge-graph_model.md.
#[path = "scenarios/ge_graph_model.rs"]
mod ge_graph_model;

/// ge.txt_render (gently-3hv): the canonical txt serialization contract,
/// one test per property row of specs/ge-txt_render.md.
#[path = "scenarios/ge_txt_render.rs"]
mod ge_txt_render;
