# graph-easy fixture corpus

Differential fixtures: each `<name>.txt` is an input text graph; its
companions are the recorded outputs of the **pinned oracle** —
Graph::Easy **v0.69** @ `ededa3d787ad89ac532c578c06390e8a7b270499`
(specs/ge-oracle.md c1) — committed alongside it:

| companion              | oracle output |
|------------------------|---------------|
| `<name>.txt.expected`   | `$graph->as_txt`   |
| `<name>.ascii.expected` | `$graph->as_ascii` |

## Pin headers (ge.oracle.c1)

Every companion **starts with a single `# oracle: ` pin header line**:

```
# oracle: Graph::Easy v0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499
```

The header is recorder metadata, not oracle output: all verifiers
(`just oracle-verify`, the `tb::oracle` scenario tests) strip leading
`# oracle: ` lines before the byte-identical comparison. A companion
whose header is missing or names a different version/commit is stale and
rejected with a typed error (ge.oracle.c4).

## Recording (ge.oracle.c5)

`just oracle-record` runs `tools/oracle.pl record tests/fixtures/graph-easy`
in one pass: new inputs get companions, stale-pinned companions are
regenerated, and companions whose pin already matches are refused
(re-recording without a pin change is a non-deliberate change).

The pin check requires Graph::Easy **exactly 0.69** loaded from the
**pinned source checkout** on `PERL5LIB` (never a cpan/cpanm-installed
copy), perl **v5.44.0**, and an explicit `PERL_HASH_SEED` (spec
ge.oracle.c1/c4 — a missing seed pin is a typed error, because hash
randomization makes recordings unreproducible). A dist tarball cannot be
verified against the pin commit; the remediation verifies it by
construction:

```sh
git clone https://github.com/shlomif/Graph-Easy /var/tmp/ge0.69/Graph-Easy-0.69
git -C /var/tmp/ge0.69/Graph-Easy-0.69 checkout ededa3d787ad89ac532c578c06390e8a7b270499
PERL_HASH_SEED=0 PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib just oracle-record
```

(The backpan tarball at the same path works if already unpacked, but its
bytes are only SIGNATURE-verified, not commit-verified.)

## Determinism (ge.oracle.c1/c3/c7)

Recordings run under a pinned `PERL_HASH_SEED` (the current corpus is
byte-consistent with seed 0); byte comparison is only meaningful at the
recording seed. Upstream renders some inputs differently across seeds —
`txt-diamond` is hash-dependent in this corpus — and those classes are
out of scope for byte-compat claims (ge.oracle.c7); the computed gate is
`tests/repro/admission.tsv` (`just repro`).

## Verifying (ge.oracle.c3)

`just oracle-verify` needs no perl: it re-renders every fixture input
through gently (the built `gently` binary for `--format ascii`; the
`tb::oracle` scenario tests drive the txt pipeline through gently-core)
and compares byte-identically to the recorded companions after header
stripping. Any mismatch fails naming the input, the format, and the
first differing line.