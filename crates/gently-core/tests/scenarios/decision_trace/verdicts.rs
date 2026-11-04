use super::*;

/// Parse a checklist verdict table — rows `| N | item | verdict | ...`
/// with N in 1..=5 — into `(item number, verdict)` pairs.
pub(crate) fn checklist_verdicts(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if !l.starts_with("| ") {
                return None;
            }
            let cells: Vec<&str> = l.split('|').collect();
            // "| a | b | ... |" splits into ["", a, b, ...]
            if cells.len() < 4 {
                return None;
            }
            let n: usize = cells[1].trim().parse().ok()?;
            if !(1..=5).contains(&n) {
                return None;
            }
            // Verdicts are **HOLDS**-style bolded; a holding verdict may
            // carry a qualifier ("HOLDS, modified form") — holding is the
            // HOLDS prefix, never the FAILS prefix.
            let verdict = cells[3].trim().trim_matches('*');
            Some((n, verdict.to_string()))
        })
        .collect()
}

/// Parse the registry's item list — rows `| N | item |` with N in 1..=5.
pub(crate) fn checklist_rows(text: &str) -> Vec<usize> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if !l.starts_with("| ") {
                return None;
            }
            let cells: Vec<&str> = l.split('|').collect();
            if cells.len() != 4 {
                return None;
            }
            let n: usize = cells[1].trim().parse().ok()?;
            (1..=5).contains(&n).then_some(n)
        })
        .collect()
}
