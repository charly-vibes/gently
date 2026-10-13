//! Integration tests for the wired `--as` format surface (gently-4ig) —
//! the `scenarios::cli::p2_formats` routing pin and the extended
//! `scenarios::cli::p4` exit-255 scope-offs, kept in their own binary so
//! tests/cli.rs holds the pretender file limit.
//!
//! Binding constraints: `specs/cli.md` c2 (the `--as` format map and
//! extension inference) and c4 (unknown/unwired formats exit 255 with a
//! diagnostic naming the requested and valid formats). Oracle evidence:
//! `tests/repro/claims/cli-formats.observed` (gently-4ig).

mod common;


mod scenarios {
    mod cli {
        use crate::common::{fixture_bytes, run_gently, run_gently_in, TempDir};

        /// cli.p2 (c2, gently-4ig): the wired `--as` formats route to their
        /// renderers, and extension inference resolves real renders.
        #[test]
        fn p2_formats() {
            // --as dot / graphviz route to the graphviz renderer. Scope-off
            // (ge.graphviz_render c4): gently's DOT omits the upstream
            // Generated-by comment and the hash-ordered edge/graph/node
            // default blocks (tests/repro/claims/cli-formats.observed) —
            // the pin is gently's own deterministic renderer bytes, not
            // the seed-unstable oracle bytes.
            let want = fixture_bytes("graphviz");
            assert!(want.starts_with("digraph GRAPH_0 {"), "sanity: {want}");
            for fmt in ["dot", "graphviz"] {
                let r = run_gently(&["--as", fmt], b"[ a ] --> [ b ]\n");
                assert_eq!(Some(0), r.code, "--as {fmt} must exit 0: {}", String::from_utf8_lossy(&r.stderr));
                assert_eq!(want.as_bytes(), &r.stdout[..], "--as {fmt} must route to the graphviz renderer");
            }

            // --as html routes to the html renderer (document form).
            // Scope-off (ge.html_render): gently's document embeds the CSS
            // rules without the upstream DOCTYPE/html/head page wrapper.
            let want = fixture_bytes("html");
            assert!(want.starts_with("<style"), "sanity: {want}");
            let r = run_gently(&["--as", "html"], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(0), r.code, "--as html must exit 0: {}", String::from_utf8_lossy(&r.stderr));
            assert_eq!(want.as_bytes(), &r.stdout[..], "--as html must route to the html renderer");

            // --as txt routes to the canonical text serializer — for the
            // plain corpus fixture the bytes match the oracle exactly
            // (tests/repro/claims/cli-formats.observed).
            let want = fixture_bytes("txt");
            assert_eq!("[ a ] --> [ b ]\n", want, "plain fixture serializes in the oracle form");
            let r = run_gently(&["--as", "txt"], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(0), r.code, "--as txt must exit 0: {}", String::from_utf8_lossy(&r.stderr));
            assert_eq!(want.as_bytes(), &r.stdout[..], "--as txt must route to the txt renderer");

            // extension inference (upstream map): with `--as` absent the
            // output filename's extension selects the format; each wired
            // extension now resolves to a real render, not exit 255.
            let dir = TempDir::new("p2-ext");
            let input = dir.0.join("in.txt");
            std::fs::write(&input, b"[ a ] --> [ b ]\n").expect("write fixture");
            for (ext, fmt) in [("html", "html"), ("txt", "txt"), ("dot", "graphviz")] {
                let out = dir.0.join(format!("out.{ext}"));
                let r = run_gently_in(&dir.0, &[input.to_str().unwrap(), out.to_str().unwrap()], b"");
                assert_eq!(Some(0), r.code, ".{ext} must resolve {fmt}: {}", String::from_utf8_lossy(&r.stderr));
                assert!(r.stdout.is_empty(), "output goes to the file, not stdout");
                assert_eq!(
                    fixture_bytes(fmt).as_bytes(),
                    std::fs::read(&out).expect("read output file").as_slice(),
                    ".{ext} output must be {fmt} bytes"
                );
            }
        }

        /// cli.p4 (c4): an unknown output format exits 255 with a diagnostic
        /// on stderr naming both the requested format and the valid formats;
        /// stdout stays empty. Exit 255 is bug-for-bug with the upstream
        /// script, whose unknown `--as` dies calling the missing
        /// `as_<fmt>` method (re-derived against upstream v0.69, gently-0h9).
        #[test]
        fn p4() {
            // A truly unknown format name.
            let r = run_gently(&["--as", "nosuch"], b"[ a ] --> [ b ]\n");
            assert_eq!(Some(255), r.code, "unknown format must exit 255");
            assert!(r.stdout.is_empty(), "diagnostics must not touch stdout");
            let err = String::from_utf8_lossy(&r.stderr);
            assert!(err.contains("nosuch"), "diagnostic must name the requested format: {err}");
            assert!(err.contains("ascii"), "diagnostic must name the valid formats: {err}");

            // gently-4ig: the known-but-unwired formats stay exit 255 with
            // the same message shape (scope-offs, evidence in
            // tests/repro/claims/cli-formats.observed): svg dies under the
            // pinned oracle's shebang perl (no in-dist As_svg), and
            // boxart/vcg/gdl/graphml are upstream-renderable but unwired
            // here — no gently renderer yet.
            for fmt in ["boxart", "svg", "vcg", "gdl", "graphml"] {
                let r = run_gently(&["--as", fmt], b"[ a ] --> [ b ]\n");
                assert_eq!(Some(255), r.code, "--as {fmt} must exit 255");
                assert!(r.stdout.is_empty(), "--as {fmt}: diagnostics must not touch stdout");
                let err = String::from_utf8_lossy(&r.stderr);
                assert!(err.contains(fmt), "--as {fmt}: diagnostic must name the requested format: {err}");
                assert!(err.contains("ascii"), "--as {fmt}: diagnostic must name the valid formats: {err}");
            }
        }

    }
}
