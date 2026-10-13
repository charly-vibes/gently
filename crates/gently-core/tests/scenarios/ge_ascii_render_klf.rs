//! ge.ascii_render p2 additions (gently-klf, gently-yhg): the POD
//! triangle's back-edge attachments and the minlen straight-gap stretch,
//! pinned from the recorded oracle bytes.

use super::{ascii, layout, text};

/// ge.ascii_render.p2 (gently-klf): the POD "Three nodes, interlinked"
/// triangle — the back-edge `Bonn -> Hamburg` rides over the band and
/// its north/south attachments sit at the cell's left+2, NOT the node
/// center (upstream `_draw_cross` draws the vertical at fixed offset 2
/// within the edge cell). Expected bytes:
/// tests/repro/claims/pod-triangle.observed (seed 0, hash-stable across
/// 4 seeds per tests/repro/admission.tsv).
#[test]
fn p2_back_edge_attachments_sit_left_plus_two() {
    let g =
        text::parse("[ Bonn ] -> [ Berlin ]\n[ Berlin ] -> [ Hamburg ]\n[ Bonn ] -> [ Hamburg ]\n")
            .expect("triangle must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("triangle must render");
    assert_eq!(
        art,
        concat!(
            "\n",
            "  +---------------------------+\n",
            "  |                           v\n",
            "+------+     +--------+     +---------+\n",
            "| Bonn | --> | Berlin | --> | Hamburg |\n",
            "+------+     +--------+     +---------+\n",
        ),
        "the back-edge's over-the-band route pins left+2 attachments"
    );
}

/// ge.ascii_render.p2 (gently-yhg): `minlen: 3` stretches the straight
/// edge one extra empty column wide — the run fills the doubled gap.
/// Probed oracle: minlen 2..5 all render identically (exactly one extra
/// GAP_WIDTH; minlen 0/1 stay adjacent), hash-stable across 4 seeds.
/// Expected bytes: tests/repro/claims/bg-minlen-stretch.observed. The
/// stretch is render-side (a gap-column width), not a placement change
/// — the bypass shape in bg-minlen-bypass.observed renders identically
/// WITHOUT minlen (probed), so that evidence pins the shared-middle
/// bypass ranking divergence, not minlen.
#[test]
fn p2_minlen_stretches_the_straight_gap_one_column() {
    let g =
        text::parse("[ Bonn ] --> { minlen: 3; } [ Berlin ]\n").expect("minlen edge must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("minlen edge must render");
    assert_eq!(
        art,
        concat!(
            "+------+          +--------+\n",
            "| Bonn | -------> | Berlin |\n",
            "+------+          +--------+\n",
        ),
        "minlen >= 2 widens the straight gap by one column"
    );
}

/// ge.ascii_render.p2 (gently-jrr): the shared-middle bypass — for
/// Bonn -> Koblenz -> {skip} Frankfurt -> Dresden plus Koblenz -> Trier
/// -> Frankfurt, the oracle ranks Trier INLINE on the row (Bonn,
/// Koblenz, Trier, Frankfurt, Dresden) and routes the skip-edge over
/// the top; gently used to attach Trier below on a fresh band. The
/// oracle render is byte-identical WITHOUT `minlen: 3` (re-derived by
/// the gently-klf run: minlen is incidental — the divergence is the
/// longest-path ranking of the east placement pass). Expected bytes:
/// tests/repro/claims/bg-minlen-bypass.observed (seed 0, hash-stable).
#[test]
fn p2_shared_middle_bypass_ranks_the_middle_inline() {
    let g = text::parse(concat!(
        "[ Bonn ] --> [ Koblenz ]\n",
        "[ Koblenz ] --> { minlen: 3; } [ Frankfurt ]\n",
        "[ Koblenz ] --> [ Trier ]\n",
        "[ Frankfurt ] --> [ Dresden ]\n",
        "[ Trier ] --> [ Frankfurt ]\n",
    ))
    .expect("bypass graph must parse");
    let art = ascii::render(&g, &layout::layout(&g)).expect("bypass graph must render");
    assert_eq!(
        art,
        concat!(
            "\n",
            "               +-----------------------------+\n",
            "               |                             v\n",
            "+------+     +---------+     +-------+     +-----------+     +---------+\n",
            "| Bonn | --> | Koblenz | --> | Trier | --> | Frankfurt | --> | Dresden |\n",
            "+------+     +---------+     +-------+     +-----------+     +---------+\n",
        ),
        "the shared-middle bypass ranks its middle node inline and bends the skip-edge over the band"
    );
}
