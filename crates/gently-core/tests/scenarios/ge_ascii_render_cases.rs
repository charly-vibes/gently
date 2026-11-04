use super::*;

/// The p2 styled-edge table (style, arrowed, oracle bytes).
pub(crate) fn p2_cases() -> Vec<(&'static str, bool, &'static str)> {
    [p2_cases_a(), p2_cases_b()].concat()
}

fn p2_cases_a() -> Vec<(&'static str, bool, &'static str)> {
    vec![
        (
            "solid",
            true,
            "+---+     +---+\n| a | --> | b |\n+---+     +---+\n",
        ),
        (
            "solid",
            false,
            "+---+     +---+\n| a | --- | b |\n+---+     +---+\n",
        ),
        (
            "double",
            true,
            "+---+     +---+\n| a | ==> | b |\n+---+     +---+\n",
        ),
        (
            "double",
            false,
            "+---+     +---+\n| a | === | b |\n+---+     +---+\n",
        ),
        (
            "dotted",
            true,
            "+---+     +---+\n| a | ..> | b |\n+---+     +---+\n",
        ),
        (
            "dotted",
            false,
            "+---+     +---+\n| a | ... | b |\n+---+     +---+\n",
        ),
        (
            "wave",
            true,
            "+---+     +---+\n| a | ~~> | b |\n+---+     +---+\n",
        ),
        (
            "wave",
            false,
            "+---+     +---+\n| a | ~~~ | b |\n+---+     +---+\n",
        ),
        (
            "dashed",
            true,
            "+---+     +---+\n| a | - > | b |\n+---+     +---+\n",
        ),
        (
            "dashed",
            false,
            "+---+     +---+\n| a |  -  | b |\n+---+     +---+\n",
        ),
    ]
}

fn p2_cases_b() -> Vec<(&'static str, bool, &'static str)> {
    vec![
        (
            "bold",
            true,
            "+---+     +---+\n| a | ##> | b |\n+---+     +---+\n",
        ),
        (
            "bold",
            false,
            "+---+     +---+\n| a | ### | b |\n+---+     +---+\n",
        ),
        (
            "wide",
            true,
            "+---+     +---+\n| a | ##> | b |\n+---+     +---+\n",
        ),
        (
            "broad",
            true,
            "+---+     +---+\n| a | ##> | b |\n+---+     +---+\n",
        ),
        (
            "wide",
            false,
            "+---+     +---+\n| a | ### | b |\n+---+     +---+\n",
        ),
        (
            "broad",
            false,
            "+---+     +---+\n| a | ### | b |\n+---+     +---+\n",
        ),
        (
            "dot-dash",
            true,
            "+---+     +---+\n| a | .-> | b |\n+---+     +---+\n",
        ),
        (
            "dot-dot-dash",
            true,
            "+---+      +---+\n| a | ..-> | b |\n+---+      +---+\n",
        ),
        (
            "double-dash",
            true,
            "+---+     +---+\n| a | = > | b |\n+---+     +---+\n",
        ),
        (
            "dot-dash",
            false,
            "+---+     +---+\n| a | -.- | b |\n+---+     +---+\n",
        ),
        (
            "dot-dot-dash",
            false,
            "+---+      +---+\n| a | -..- | b |\n+---+      +---+\n",
        ),
    ]
}

/// ge.ascii_render.p2 (c2): each edge's glyph run matches the style table
/// byte-for-byte, arrowed and arrow-less (probed oracle bytes).
#[test]
pub(crate) fn p2() {
    let cases = p2_cases();
    assert!(
        cases.len() >= 10,
        "every styled-edge oracle case is covered"
    );
    assert_p2_edge_styles(&cases);
}
