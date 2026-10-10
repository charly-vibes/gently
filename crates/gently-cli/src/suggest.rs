//! Purpose: genesis suggestion surface for unknown names (specs/cli.md
//! c7, gently-ef5).
//! Responsibilities: pre-scan argv for unknown flags and subcommand
//! near-misses, resolve suggestions through genesis `suggest_typo` over
//! the known-name registry, and emit `DidYouMean` / `Fix` messages on
//! stderr before the caller exits nonzero (exit 2, clap's invalid-usage
//! code).
//! Rationale: bare tokens carrying path separators or extensions are
//! pipeline input files, never subcommand candidates — this keeps upstream
//! `[inputfile]` behavior intact (c1) while still catching `gently initt`
//! style misses; flag values are skipped via the [`VALUE_FLAGS`] ledger so
//! `--as ascii` never suggests against its own value.

use genesis::suggestions::{CommandRegistry, Suggestion, SuggestionEngine};

/// Known subcommands (c5/c7): everything else is pipeline input or a
/// near-miss suggestion.
pub const SUBCOMMANDS: &[&str] = &["init", "doctor"];

/// Flags that consume a following value — skipped when scanning for the
/// subcommand position.
pub const VALUE_FLAGS: &[&str] = &["--as", "--output"];

/// Known long flags (c7 registry), dashes stripped for typo matching.
pub const KNOWN_FLAG_NAMES: &[&str] = &[
    "as", "output", "json", "human", "verbose", "quiet", "fix", "help", "version",
];

/// Known single-dash flags (clap's derived shorts: `-v`/`-q` verbosity,
/// `-j` json, `-h`/`-V` help/version).
const KNOWN_SHORTS: &[&str] = &["v", "q", "j", "h", "V"];

/// Peel a leading subcommand off the argv (c5 dispatch): skip flags and
/// their values, and treat the first bare token as a subcommand only when
/// it is one — everything else is pipeline input. Returns the subcommand
/// name and removes it from argv on match.
pub fn extract_subcommand(argv: &mut Vec<String>) -> Option<String> {
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        if VALUE_FLAGS.contains(&arg) {
            i += 2;
            continue;
        }
        if arg.starts_with('-') {
            i += 1;
            continue;
        }
        if arg.contains('/') || arg.contains('.') {
            return None;
        }
        let found = SUBCOMMANDS.iter().find(|s| **s == arg).map(|s| s.to_string());
        if found.is_some() {
            argv.remove(i);
        }
        return found;
    }
    None
}

/// cli.c7 precheck: any unknown flag, or a first bare token that is a
/// near-miss of a known subcommand, gets a genesis `suggest_typo` (or a
/// `Fix`) suggestion naming the closest known name on stderr, then the
/// caller exits 2. Returns the exit code when a suggestion was emitted.
pub fn precheck_known_names(argv: &[String]) -> Option<i32> {
    let engine = SuggestionEngine::new();
    let is_known = |name: &str| {
        KNOWN_FLAG_NAMES.contains(&name) || KNOWN_SHORTS.contains(&name)
    };

    // Flags: every `--name[=value]` or `-x` token must be known; unknown
    // ones get a DidYouMean when a close name exists, a Fix otherwise.
    let flag_registry = flag_names_registry();
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        if VALUE_FLAGS.contains(&arg) {
            i += 2;
            continue;
        }
        if !is_flag(arg) {
            i += 1;
            continue;
        }
        if let Some(exit_code) = flag_suggestion(&engine, &flag_registry, is_known, arg) {
            return Some(exit_code);
        }
        i += 1;
    }

    // Subcommand near-miss: first bare token without path separators, that
    // is not itself a subcommand (files are exempt via the separator
    // guard). Suggestions come only from the subcommand registry so
    // pipeline filenames never cross-match flags.
    if let Some(arg) = first_bare_token(argv) {
        if !SUBCOMMANDS.contains(&arg.as_str()) {
            if let Some(suggestion) = engine.suggest_typo(&arg, &subcommand_registry()) {
                report(&suggestion);
                return Some(2);
            }
        }
    }
    None
}

/// Resolve an unknown flag token: DidYouMean on a close registry match,
/// otherwise a Fix naming the valid flags. Returns the exit code (2).
fn flag_suggestion(
    engine: &SuggestionEngine,
    registry: &CommandRegistry,
    is_known: impl Fn(&str) -> bool,
    arg: &str,
) -> Option<i32> {
    let name = arg.trim_start_matches('-');
    let name = name.split('=').next().unwrap_or(name);
    if is_known(name) {
        return None;
    }
    if let Some(suggestion) = engine.suggest_typo(name, registry) {
        report(&suggestion);
        return Some(2);
    }
    let fix = Suggestion::fix(format!(
        "Unknown flag '{arg}' (valid flags: {})",
        KNOWN_FLAG_NAMES
            .iter()
            .map(|f| format!("--{f}"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    report(&fix);
    Some(2)
}

/// The first bare token that is neither a flag nor a flag value, or `None`
/// when argv holds only flags (or the first bare token is path-like).
fn first_bare_token(argv: &[String]) -> Option<String> {
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        if VALUE_FLAGS.contains(&arg) {
            i += 2;
            continue;
        }
        if arg.starts_with('-') {
            i += 1;
            continue;
        }
        if arg.contains('/') || arg.contains('.') {
            return None;
        }
        return Some(arg.to_string());
    }
    None
}

/// `--name` / `-x` token shape (not the bare `-` stdin placeholder).
fn is_flag(arg: &str) -> bool {
    (arg.starts_with("--") && arg.len() > 2)
        || (arg.starts_with('-') && arg != "-" && arg.len() > 1 && !arg.starts_with("--"))
}

fn subcommand_registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register("gently", SUBCOMMANDS.iter().map(|s| s.to_string()).collect());
    registry
}

fn flag_names_registry() -> CommandRegistry {
    let mut registry = subcommand_registry();
    registry.register("gently", KNOWN_FLAG_NAMES.iter().map(|s| s.to_string()).collect());
    registry
}

/// Print a genesis suggestion to stderr — nowhere else (c7).
fn report(suggestion: &Suggestion) {
    eprintln!("gently: {}", suggestion.message());
}
