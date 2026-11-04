use super::*;

/// Case 1: perl missing (fake PATH without perl) — the justfile recipe types this first.
pub(crate) fn p4_missing_perl(tmp: &Path) {
    let bin = tmp.join("bin");
    std::fs::create_dir_all(&bin).must("temp bin dir");
    #[cfg(unix)]
    std::os::unix::fs::symlink("/bin/bash", bin.join("bash")).must("symlink bash");
    let (ok, out) = run(Command::new("/usr/bin/just")
        .arg("oracle-record")
        .current_dir(repo_root())
        .env("PATH", &bin));
    typed_failure(ok, &out, "perl not found", "missing perl");
}

/// Case 2: version drift — a fake checkout exposing Graph::Easy 0.99.
pub(crate) fn p4_version_drift(tmp: &Path) {
    let fake = tmp.join("fake-lib");
    fake_checkout(&fake, "0.99");
    let (ok, out) = run(&mut record_cmd(tmp, &[("PERL5LIB", fake.as_path())]));
    typed_failure(ok, &out, "0.99", "version drift");
}

/// Case 3: off the pinned checkout — right version from the wrong path
/// (isolates the source-checkout clause; a fully missing checkout surfaces
/// as cpan-copy drift or the p3/p5 skip guard in CI).
pub(crate) fn p4_off_checkout(tmp: &Path) {
    let off = tmp.join("off-path-lib");
    fake_checkout(&off, PIN_VERSION);
    let (ok, out) = run(&mut record_cmd(tmp, &[("PERL5LIB", off.as_path())]));
    typed_failure(
        ok,
        &out,
        "pin requires the source checkout",
        "off-checkout load path",
    );
}

/// Case 4: record run without an explicit PERL_HASH_SEED.
pub(crate) fn p4_missing_seed(tmp: &Path) {
    let pin = PathBuf::from(format!(
        "/var/tmp/ge{PIN_VERSION}/Graph-Easy-{PIN_VERSION}/lib"
    ));
    let mut cmd = record_cmd(tmp, &[("PERL5LIB", pin.as_path())]);
    cmd.env_remove("PERL_HASH_SEED");
    let (ok, out) = run(&mut cmd);
    typed_failure(ok, &out, "PERL_HASH_SEED", "missing seed pin");
}

/// A minimal git repo at `root` with one empty commit; returns its HEAD sha.
pub(crate) fn git_repo_at_wrong_commit(root: &Path) -> String {
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "probe")
            .env("GIT_AUTHOR_EMAIL", "probe@example.invalid")
            .env("GIT_COMMITTER_NAME", "probe")
            .env("GIT_COMMITTER_EMAIL", "probe@example.invalid")
            .output()
            .must("spawn git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    git(&["init", "-q"]);
    git(&[
        "commit",
        "-q",
        "--allow-empty",
        "-m",
        "drifted checkout probe",
    ]);
    git(&["rev-parse", "HEAD"])
}

/// Case 6 (gently-4sq): commit drift — the pinned version loaded from a
/// checkout whose git HEAD is not the pin commit. GE_PIN_LIB points the
/// pin at the fake checkout so the load-path clause passes and the commit
/// clause is exercised alone; a real drifted checkout is the same shape.
pub(crate) fn p4_commit_drift(tmp: &Path) {
    let root = tmp.join("drifted-checkout");
    fake_checkout(&root.join("lib"), PIN_VERSION);
    let head = git_repo_at_wrong_commit(&root);
    let lib = root.join("lib");
    let (ok, out) = run(&mut record_cmd(
        tmp,
        &[("PERL5LIB", lib.as_path()), ("GE_PIN_LIB", lib.as_path())],
    ));
    typed_failure(ok, &out, "pin commit is", "commit drift");
    assert!(
        out.contains(PIN_COMMIT) && out.contains(&head),
        "commit drift must name both shas (ge.oracle.c4), got: {out}"
    );
}

/// Case 5: stale pin header — just oracle-verify types it with the
/// `just oracle-record` remediation before any comparison runs.
pub(crate) fn p4_stale_pin(tmp: &Path) {
    let stale = tmp.join("stale-corpus");
    std::fs::create_dir_all(&stale).must("stale corpus dir");
    std::fs::write(stale.join("x.txt"), "[ a ] --> [ b ]\n").must("input");
    std::fs::write(
        stale.join("x.txt.expected"),
        "# oracle: Graph::Easy v0.68 @ 0000000000000000000000000000000000000000\nstale\n",
    )
    .must("stale companion");
    let (ok, out) = run(Command::new("/usr/bin/just")
        .arg("oracle-verify")
        .env("FIX", &stale)
        .current_dir(repo_root()));
    typed_failure(ok, &out, "stale pin header", "stale pin header");
}

#[test]
pub(crate) fn p4() {
    if !toolchain() {
        skip("p4 needs perl + the pinned Graph::Easy checkout (ge.oracle.c4)");
        return;
    }
    let tmp = tempdir("p4");
    p4_missing_perl(&tmp);
    p4_version_drift(&tmp);
    p4_off_checkout(&tmp);
    p4_missing_seed(&tmp);
    p4_commit_drift(&tmp);
    p4_stale_pin(&tmp);
    assert!(
        std::fs::remove_dir_all(&tmp).is_ok(),
        "p4 tempdir must clean up"
    );
}

/// After a record run, tiny.txt companions carry pin header + metadata (c1/c5).
pub(crate) fn assert_recorded(dir: &Path, companion: &str) {
    let text = std::fs::read_to_string(dir.join(companion)).must("recorded companion");
    assert!(
        text.starts_with(&pin_header()),
        "{companion}: pin header required (ge.oracle.c1/c5)"
    );
    assert!(
        text.lines()
            .nth(1)
            .unwrap_or("")
            .starts_with("# oracle: perl "),
        "{companion}: metadata line required (ge.oracle.c1)"
    );
}

/// Re-record without a pin change → refused, bytes untouched (c5).
pub(crate) fn assert_rejected(dir: &Path, pin: &Path) {
    let before = std::fs::read(dir.join("tiny.txt.expected")).must("companion bytes");
    let (ok, out) = run(&mut record_cmd(dir, &[("PERL5LIB", pin)]));
    assert!(ok, "refusal is a clean no-op, not a crash");
    assert!(
        out.contains("refusing to overwrite"),
        "re-recording without a pin change must be rejected (ge.oracle.c5), got: {out}"
    );
    let after = std::fs::read(dir.join("tiny.txt.expected")).must("companion bytes");
    assert_eq!(
        before, after,
        "refused re-record must not touch companion bytes (ge.oracle.c5)"
    );
}

pub(crate) fn tempdir(tag: &str) -> PathBuf {
    let tmp = std::env::temp_dir().join(format!("gently-oracle-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).must("tempdir");
    tmp
}

/// The `oracle.pl record <dir>` command with an environment applied.
pub(crate) fn record_cmd(dir: &Path, envs: &[(&str, &Path)]) -> Command {
    let mut cmd = Command::new("perl");
    cmd.arg(repo_root().join("tools/oracle.pl"))
        .arg("record")
        .arg(dir)
        .env("PERL_HASH_SEED", "0");
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd
}

/// A fake Graph::Easy checkout at `lib` exposing the given version.
pub(crate) fn fake_checkout(lib: &Path, version: &str) {
    std::fs::create_dir_all(lib.join("Graph")).must("fake lib");
    std::fs::write(
        lib.join("Graph/Easy.pm"),
        format!("package Graph::Easy; our $VERSION = '{version}'; 1;\n"),
    )
    .must("write fake Graph::Easy");
}

pub(crate) fn skip(reason: &str) {
    println!("oracle toolchain unavailable — skipping: {reason}");
}

/// Run a command, return (success, combined stdout+stderr).
pub(crate) fn run(cmd: &mut Command) -> (bool, String) {
    let out = cmd.output().must("spawn command");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), combined)
}

/// A drifted-environment run must fail typed, naming the piece + remediation
/// (ge.oracle.c4).
pub(crate) fn typed_failure(ok: bool, out: &str, needle: &str, case: &str) {
    assert!(
        !ok && out.contains(needle) && out.contains("remediation"),
        "{case} must be a typed error + remediation (ge.oracle.c4), got: {out}"
    );
}
