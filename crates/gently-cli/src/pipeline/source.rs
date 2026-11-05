//! Purpose: reading the graph source for the pipeline (specs/cli.md c1).
//! Responsibilities: stdin vs first-file-argument selection, with the
//! stdin kind and reader injected for hermetic tests.
//! Rationale: injecting the reader matters because under lefthook/git
//! push the hook's stdin is the open controlling terminal, so a
//! global-stdin read blocks the test suite forever (beads gently-ktv).

use std::io::IsTerminal;

use crate::{Cli, CliError};

/// Read the graph source: stdin when no file arguments are given, otherwise
/// the first file argument's contents (specs/cli.md c1 — a second positional
/// names the output file, further positionals are ignored, so only the
/// first is ever parsed).
pub fn read_source(cli: &Cli) -> Result<String, CliError> {
    let stdin = std::io::stdin();
    read_source_impl(cli, stdin.is_terminal(), &mut stdin.lock())
}

/// The source read with the stdin kind and the reader both injected so the
/// unit tests are hermetic: a terminal stdin with no file argument fails
/// fast (c1 guard for gently-in2 — the upstream oracle is always exercised
/// with piped stdin, so this deviates from upstream only where upstream
/// would block forever on an interactive prompt), and the piped path reads
/// the supplied reader instead of the process's global stdin. Injecting the
/// reader matters because under lefthook/git push the hook's stdin is the
/// open controlling terminal, so a global-stdin read blocks the test suite
/// forever (beads gently-ktv).
pub(crate) fn read_source_impl(
    cli: &Cli,
    stdin_is_terminal: bool,
    stdin: &mut dyn std::io::Read,
) -> Result<String, CliError> {
    let Some(file) = cli.files.first() else {
        if stdin_is_terminal {
            return Err(CliError(
                "no input file given and stdin is a terminal; pipe graph text into gently or pass an input file"
                    .to_string(),
            ));
        }
        let mut buf = String::new();
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| CliError(format!("cannot read stdin: {e}")))?;
        return Ok(buf);
    };
    let content = std::fs::read_to_string(file)
        .map_err(|e| CliError(format!("cannot read {}: {e}", file.display())))?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    fn cli_with(args: &[&str]) -> Cli {
        Cli::try_parse_from(std::iter::once("gently").chain(args.iter().cloned()))
            .expect("valid args")
    }

    /// gently-in2: with no file argument and a terminal stdin, the source
    /// read fails fast with a diagnostic instead of blocking forever —
    /// interactive invocations (and agent subprocesses inheriting an open
    /// TTY) get a usable error, piped stdin keeps the c1 oracle contract.
    #[test]
    fn terminal_stdin_without_file_fails_fast() {
        let cli = cli_with(&[]);
        let err = read_source_impl(&cli, true, &mut std::io::empty())
            .expect_err("must not block on a TTY");
        assert!(err.to_string().contains("stdin is a terminal"));
        assert!(err.to_string().contains("input file"));
    }

    /// The piped path is unchanged: non-terminal stdin is read (cli.c1) —
    /// from the injected reader, never the process's global stdin, which
    /// under lefthook/git push is the open controlling terminal and would
    /// hang the suite (beads gently-ktv).
    #[test]
    fn piped_stdin_still_read_when_not_terminal() {
        let cli = cli_with(&[]);
        let source = read_source_impl(
            &cli,
            false,
            &mut std::io::Cursor::new(b"[ a ] --> [ b ]\n".to_vec()),
        )
        .expect("piped stdin reads");
        assert_eq!(source, "[ a ] --> [ b ]\n");
    }

    /// The piped path reads the reader to EOF: an empty reader yields the
    /// empty graph (cli.c1 edge), without touching the global stdin.
    #[test]
    fn piped_empty_stdin_renders_empty() {
        let cli = cli_with(&[]);
        let source =
            read_source_impl(&cli, false, &mut std::io::empty()).expect("empty pipe reads");
        assert_eq!(source, "");
    }
}
