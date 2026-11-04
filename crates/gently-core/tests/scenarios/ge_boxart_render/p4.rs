use super::*;

/// p4 (c4): a node's shape attribute changes the outline per the upstream
/// Unicode shape table — `rounded` overlays the ╭╮╯╰ corner set for the
/// solid|dotted|dashed|dot-dash|dot-dot-dash border styles, blanks the
/// corners for bold|wide|broad|double|double-dash|bold-dash, and KEEPS
/// the corners for wave (upstream quirk); `point` swaps the label for a
/// centered ★; `invisible` draws nothing; every other vocabulary name
/// collapses to the plain box.
#[test]
pub(crate) fn p4() {
    for (shape, want) in [
        ("rect", "┌───┐\n│ x │\n└───┘\n"),
        ("rounded", "╭───╮\n│ x │\n╰───╯\n"),
        ("point", "\n  ★\n\n"),
        ("circle", "┌───┐\n│ x │\n└───┘\n"),
        ("ellipse", "┌───┐\n│ x │\n└───┘\n"),
        ("diamond", "┌───┐\n│ x │\n└───┘\n"),
        ("triangle", "┌───┐\n│ x │\n└───┘\n"),
        ("pentagon", "┌───┐\n│ x │\n└───┘\n"),
        ("hexagon", "┌───┐\n│ x │\n└───┘\n"),
        ("octagon", "┌───┐\n│ x │\n└───┘\n"),
        ("parallelogram", "┌───┐\n│ x │\n└───┘\n"),
        ("house", "┌───┐\n│ x │\n└───┘\n"),
        ("invisible", "\n\n\n"),
        ("img", "┌───┐\n│ x │\n└───┘\n"),
    ] {
        assert_eq!(render(&shaped_node(shape)), want, "shape {shape}");
    }
    // rounded × border style: overlay, blank, or keep (wave quirk).
    for (border, want) in [
        ("double", " ═══\n║ x ║\n ═══\n"),
        ("dotted", "╭⋯⋯⋯╮\n⋮ x ⋮\n╰⋯⋯⋯╯\n"),
        ("wave", "┌∼∼∼┐\n≀ x ≀\n└∼∼∼┘\n"),
        ("bold", " ━━━\n┃ x ┃\n ━━━\n"),
        ("dot-dot-dash", "╭·-·╮\n│ x │\n╰·-·╯\n"),
    ] {
        let mut g = bordered_node(border);
        let n = g.nodes.len() - 1;
        g.set_attr(Scope::Node(n), "shape", "rounded");
        assert_eq!(render(&g), want, "rounded + border {border}");
    }
}
