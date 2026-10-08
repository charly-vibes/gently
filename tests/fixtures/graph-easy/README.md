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

The pin check requires Graph::Easy **exactly 0.69** on `PERL5LIB`
(system perl often carries a different version). Two remediations:

```sh
# system-wide via cpanm
cpanm Graph::Easy==0.69

# or an isolated checkout (ephemeral, recording is rare and deliberate)
mkdir -p /var/tmp/ge069 && cd /var/tmp/ge069
curl -fsSLO https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-0.69.tar.gz
tar xzf Graph-Easy-0.69.tar.gz
PERL5LIB=/var/tmp/ge069/Graph-Easy-0.69/lib just oracle-record
```

## Verifying (ge.oracle.c3)

`just oracle-verify` needs no perl: it re-renders every fixture input
through gently (the built `gently` binary for `--format ascii`; the
`tb::oracle` scenario tests drive the txt pipeline through gently-core)
and compares byte-identically to the recorded companions after header
stripping. Any mismatch fails naming the input, the format, and the
first differing line.