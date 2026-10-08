# devenv.sh — single source of truth for gently's toolchain (spike for DDL-irv).
# Verify: devenv shell -- just gates
{ pkgs, ... }: {
  packages = with pkgs; [
    just # task runner (justfile is the QA entry point)
    perl # oracle fixture recording (spec ge.oracle.c4)
  ];

  # Oracle fixtures pin Graph::Easy v0.69 @ ededa3d7 (spec ge.oracle.c4).
  # nixpkgs perlPackages.GraphEasy tracks upstream, not the pin — oracle-record
  # fails honestly on mismatch with remediation `cpanm Graph::Easy==0.69`.
  # specodelic/testaruda are fleet tools provisioned by ddl (brew/crates), not
  # duplicated here until DDL-irv decides the fleet flake question.
}
