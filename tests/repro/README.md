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
| `sharp-label` | gently-6j0/bzx (text_parser c7) | unescaped in-string `#` truncates the line (even quoted); hex colours after the separator auto-escape |
| `operator-patterns` | gently-6j0/bzx (text_parser c9, c2) | style follows the LAST unit token; `..-..-..>` is a valid dotted edge; `.-` valid undirected; bidirectional `<...>`; lone `<` and missing endpoints are errors |
| `group-syntax` | gently-6j0/bzx (text_parser c6) | colon joins the group NAME (`G:`); node belongs to one group (later group wins); nested inner nodes stay out of the outer group; anonymous group named `Group #0` |
| `anon-reference` | gently-6j0/bzx (text_parser c1) | anon nodes are named `#N` and REUSABLE via escaped `[ \#N ]`; bare `[ #N ]` is a parse error |
| `layout-flow-direction` | gently-89d (layout c3) | cycle under flow=east renders an against-flow edge |
| `subgraph-handling` | gently-13f (dot_parser c4) | any named subgraph becomes a group (name VERBATIM); nameless `subgraph {}` is a tokenizing error; bare `{}` keeps nodes ungrouped (but the pre-scope node is still linked by the scope's inner edge chain — a spurious `b --> d` edge); a named subgraph is a valid edge endpoint (edge to the GROUP object); nested nodes belong to the innermost group only |
| `dot-direction` | gently-13f (dot_parser c1/c2) | the EDGE OPERATOR decides direction, not the header: `->` directed / `--` undirected under either header; a `graph` header sets `type: undirected` on the model graph, a `digraph` header leaves the directed default |
| `dot-records-ports` | gently-13f (dot_parser c5) | records and HTML-like table labels PARSE into numbered part nodes `name.N` (port markers stripped, edges reattached to referenced parts); a record-style label without `shape=record` stays a verbatim attribute; unresolvable port refs error `Cannot find autosplit node for <base>:<port> on edge <id>`; malformed HTML-like labels are tokenizing errors |
| `cli-flags` | gently-0h9 (closed, upstream evidence) | `--as`/`--output`/positional roles; unknown format exits 255 |
| `node-unnamed` | gently-r22 (graph_model c1) | anonymous nodes are NAMED `#1`,`#3`; `node('#1')` finds them |
| `anon-numbering` | gently-r22 (graph_model c1) | anon name = `#` + global object id (shared with edges/groups); the parser RESETS the counter per parse (deterministic names), API `add_anon_node` continues it; anon nodes carry a default label `' '` |
| `attr-store-decompose` | gently-r22 (graph_model c2) | values pass through the store-layer unquote at set; `border` is DECOMPOSED into border-style/width/color at assignment (never stored verbatim) and recomposed on read — `dotted bold red` reads back `bold  red`; class-scope attrs stored separately, instance reads override |
| `group-edge` | gently-r22 (graph_model c3) | edges store a GROUP OBJECT as endpoint (`Graph::Easy::Group -> Graph::Easy::Group`), not rewritten to members; bare `( A ) --> ( B )` creates groups + edge, no nodes; as_txt renders the groups but drops the group-to-group edge |
| `deleted-node-add-edge` | gently-r22 (graph_model c3/p3) | `del_node` drops the node AND its incident edges; later `add_edge('A','B')` SUCCEEDS, re-creating A as a fresh bare node (no stored attrs); old edges are not revived |
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
