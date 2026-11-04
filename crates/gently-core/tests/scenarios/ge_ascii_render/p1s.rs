use super::*;

/// ge.ascii_render.p1 (c1, gently-awd): a size-attributed node's box
/// spans its `columns` grid columns — the spanned columns balance
/// smallest-first (upstream `_balance_sizes`) until their sum reaches
/// the node's own label width — while `rows` spans collapse: the empty
/// continuation rows are 0 tall, so `size: 1,2` / `rows: 3` stay
/// standard 3-line boxes. The bytes are the recorded oracle companions
/// (tests/fixtures/graph-easy/size-attr.ascii.expected, stripped of the
/// pin header).
#[test]
fn p1_size_attr_spans() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/graph-easy/size-attr.txt"
    ))
    .must("size-attr fixture");
    let g = text::parse(&src).must("size-attr must parse");
    let art = ascii::render(&g, &layout::layout(&g)).must("size-attr must render");
    let want = "+----+     +----+\n| A  | --> | B  |\n+----+     +----+\
\n+---------+      +----+\n|   A2    | ---> | B2 |\n+---------+      +----+\
\n+---------------+      +----+\n|      A3       | ---> | B3 |\
\n+---------------+      +----+\n+----+     +----+\n| A4 | --> | B4 |\
\n+----+     +----+\n+---------------+      +----+\n|      A5       | ---> | B5 |\
\n+---------------+      +----+\n";
    assert_eq!(art, want);
}

/// ge.ascii_render.p1 (c1, gently-awd): parallel channels between
/// multi-row boxes ride the boxes' row span — each channel a straight
/// run at its own height, the label riding its own channel row, the
/// boxes grown to their full row span (the so-flowchart-size class).
/// The bytes are gently's deterministic channel order (model order):
/// the pinned oracle's seed-0 render swaps the two east-going labels
/// (its channel order is hash-driven — same class as the fan
/// residuals), the recorded claim probe
/// tests/repro/claims/so-flowchart-size.observed pins both seeds.
#[test]
fn p1_parallel_channels_rows() {
    let src = "[ Client ] { size: 5,10; }\n[ Msg Server ] { size: 5,10; }\
\n\n[ Server ] -- put msg\\n via http --> [ Msg Server ]\
\n[ Msg Server ] -- 1. Number X of unread --> [ Client ]\
\n[ Msg Server ] -- 3. X Msg --> [ Client ]\
\n[ Client ] -- 2. read X --> [ Msg Server ]\n";
    let g = text::parse(src).must("flowchart must parse");
    let art = ascii::render(&g, &layout::layout(&g)).must("flowchart must render");
    let want = concat!(
        "+--------+  put msg    +------------+                          +--------+\n",
        "| Server |  via http   |            |  1. Number X of unread   |        |\n",
        "|        | ----------> |            | -----------------------> |        |\n",
        "+--------+             |            |                          |        |\n",
        "                       | Msg Server |  3. X Msg                | Client |\n",
        "                       |            | -----------------------> |        |\n",
        "                       |            |                          |        |\n",
        "                       |            |  2. read X               |        |\n",
        "                       |            | <----------------------- |        |\n",
        "                       +------------+                          +--------+\n",
    );
    assert_eq!(art, want);
}
