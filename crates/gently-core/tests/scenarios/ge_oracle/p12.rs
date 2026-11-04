use super::p4family::*;
use super::*;

/// ge.oracle.p1 (c1): every pin header names version + commit, every
/// metadata line names perl + seed, no companion carries a foreign pin.
#[test]
fn p1() {
    let cs = companions();
    assert!(!cs.is_empty(), "corpus must not be empty");
    for c in &cs {
        let mut lines = c.raw.lines();
        let header = lines.next().unwrap_or("");
        assert_eq!(
            format!("{header}\n"),
            pin_header(),
            "{}:{}: pin header must name the pinned oracle (c1)",
            c.stem,
            c.format
        );
        let meta = lines.next().unwrap_or("");
        assert!(
            meta.starts_with("# oracle: perl ") && meta.contains("PERL_HASH_SEED="),
            "{}:{}: metadata must name perl version + seed (c1), got {meta:?}",
            c.stem,
            c.format
        );
        for stray in lines {
            assert!(
                !stray.starts_with("# oracle:"),
                "{}:{}: foreign pin {stray:?} (c1)",
                c.stem,
                c.format
            );
        }
    }
}

/// ge.oracle.p2 (c2): every corpus input has exactly one recorded expected
/// output per supported format (txt, ascii); no orphans.
#[test]
fn p2() {
    let inputs = corpus_inputs();
    assert!(!inputs.is_empty(), "corpus must not be empty");
    let cs = companions();
    for input in &inputs {
        for format in ["txt", "ascii"] {
            assert_eq!(companion_count(&cs, input, format), 1, "input {input}.txt must have exactly one {format}.expected companion (ge.oracle.c2)");
        }
    }
    assert_no_orphans(&cs, &inputs);
    assert!(
        cs.len() >= inputs.len(),
        "each input contributes at least one companion (c2)"
    );
}

/// ge.oracle.p3 (c3): the full verify pass — every fixture re-rendered and
/// compared byte-identically to the recorded companions; the runner owns
/// the diff report, this arm asserts its exit status.
#[test]
fn p3() {
    if !toolchain() {
        skip("p3 needs perl + the pinned Graph::Easy checkout (ge.oracle.c4)");
        return;
    }
    let (ok, out) = run(Command::new("just")
        .arg("oracle-verify")
        .current_dir(repo_root()));
    assert!(ok, "just oracle-verify failed — see its input/format/first-differing-line report (ge.oracle.c3): {out}");
}

/// Companion count for one input/format pair.
fn companion_count(cs: &[Companion], input: &str, format: &str) -> usize {
    cs.iter()
        .filter(|c| c.stem == input && c.format == format)
        .count()
}

/// No companion without a matching corpus input (ge.oracle.c2).
fn assert_no_orphans(cs: &[Companion], inputs: &[String]) {
    for c in cs {
        assert!(
            inputs.iter().any(|i| i == &c.stem),
            "orphan companion for unknown input {}.{} (ge.oracle.c2)",
            c.stem,
            c.format
        );
    }
}
