# repro-probe harness (gently-mwo)

The falsifiable probe harness for the byte-compat scope contract —
`specs/ge-oracle.md` c7/p7, decision record
`.wai/projects/graph-easy-port/designs/2026-10-09-decision-byte-compat-scope.md`.
It is the enforcement arm: tier-1 membership is computed by these probes,
never hardcoded.

## Layout

| path | role |
|------|------|
| `probes.pl` | the runner — `admit`, `claim <name>`, `claims` |
| `admission.tsv` | generated + committed: per-fixture tier-1 classification, machine-checked by `scenarios::ge_oracle::p7` |
| `claims/<name>.observed` | generated + committed: observed pinned-oracle behavior per claim probe (evidence for spec amendments) |

## Pin (ge.oracle.c1)

Run everything under the pinned oracle — Graph::Easy **v0.69** @
`ededa3d787ad89ac532c578c06390e8a7b270499`. The runner refuses to run on a
drifted environment with a typed error naming the remediation:

```sh
mkdir -p /var/tmp/ge0.69 && cd /var/tmp/ge0.69
curl -fsSLO https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-0.69.tar.gz
tar xzf Graph-Easy-0.69.tar.gz
export PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib
```

Note the pin-contract gap (gently-ikm/liz): a tarball cannot be verified
against the named git commit, and the seed is only pinned for the run when
`PERL_HASH_SEED` is exported.

## Admission sweep (tier-1 gate)

```sh
PERL5LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib just repro
# or directly:
perl tests/repro/probes.pl admit tests/fixtures/graph-easy
```

For every corpus input the sweep renders `as_txt` + `as_ascii` in child
processes under 4 hash seeds and records:

- `stable` — identical digests across all seeds; `hash-dependent` —
  digests diverge across seeds (the oracle itself cannot reproduce the
  bytes); `render-error` — the oracle crashes or hangs (SIGKILL-bounded).
- `seconds` — worst observed render wall time; the tier-1 envelope is
  10 s (ge.oracle.c7).

Tier-1 admissible = `stable` **and** `seconds <= 10`. The committed
`admission.tsv` is re-derived and validated by
`cargo test -p gently-core ge_oracle::p7` — which *derives* the admissible
set from the manifest (no count is hardcoded anywhere).

If the oracle pin ever moves past 0.69, rerun the sweep: tier-1 membership
is recomputed by the probes; the contract does not change.

## Claim probes (per-amendment evidence)

```sh
perl tests/repro/probes.pl claims          # list probes + beads served
perl tests/repro/probes.pl claim node-unnamed
```

Each probe reproduces one upstream-review finding class under the pinned
oracle and records the observed bytes to `claims/<name>.observed`. A
spec-amendment bead runs its probe, shows the delta between the spec row's
claim and the observed bytes, and amends the row in the same commit.

| probe | serves | observed delta |
|-------|--------|----------------|
| `sharp-escape` | gently-6j0/bzx (text_parser c7) | `#` escapes as `\#` in as_txt and round-trips |
| `layout-flow-direction` | gently-89d (layout c3) | cycle under flow=east renders an against-flow edge |
| `subgraph-handling` | gently-13f (dot_parser c3) | any named subgraph becomes a group; anonymous `{}` is an error |
| `cli-flags` | gently-0h9 (closed, upstream evidence) | `--as`/`--output`/positional roles; unknown format exits 255 |
| `node-unnamed` | gently-r22 (graph_model c1) | anonymous nodes are NAMED `#1`,`#3`; `node('#1')` finds them |
| `href-escaping` | gently-dcp/eyo (html_render c2) | href carries the raw `&` |
| `td-colspan` | gently-dcp/eyo (html_render c4) | 1 td with colspan=4 rowspan=4 ≠ cell count; oracle may HANG on multiline labels |
| `shape-outline-collapse` | gently-dcp/css (ascii shapes) | shape names rejected outright; bold/wide/broad differ only in border-width |
| `graphviz-round-trip` | gently-0kg/b4v (graphviz c2/c4, txt_render c3) | always `digraph`+`->`; groups → `cluster0`; wall-clock timestamp in output |
| `size-envelope` | gently-k4u (perf c2/c5) | 100 nodes ≈ 4.5 s with dropped edges; 200 nodes trips the alarm |
| `perl5lib-pin` | gently-ikm/liz (pin contract) | commit unverifiable from a dist; seed pin absent |

## Known probe hazards

- The oracle **hangs** on multiline-label nodes under `as_html`/`as_ascii`
  — an infinite loop `alarm()` cannot bound (upstream's internal `eval`
  blocks swallow the die). All oracle renderings are SIGKILL-bounded
  children; a hang is evidence, never a stalled run.
- `as_graphviz` output embeds a wall-clock timestamp — never diff it
  byte-wise across runs.
