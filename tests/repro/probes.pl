#!/usr/bin/env perl
# Purpose: the falsifiable repro-probe harness (gently-mwo) — the
# enforcement arm of the byte-compat scope contract (specs/ge-oracle.md
# c7/p7, decision record
# .wai/projects/graph-easy-port/designs/2026-10-09-decision-byte-compat-scope.md).
# Responsibilities: in `admit` mode, classify every corpus fixture by the
# two tier-1 admission probes (hash-stability across seeds under the
# pinned oracle; render time against the 10 s envelope) into a
# machine-checkable manifest (tests/repro/admission.tsv, validated by the
# scenarios::ge_oracle::p7 cargo test). In `claim` mode, run a named
# claim probe that grounds one upstream-review finding: it reproduces the
# upstream behavior, prints the observed bytes, and records them to
# tests/repro/claims/<name>.observed, so each spec-amendment bead can
# show the delta between what its spec row claims and what the pinned
# oracle actually does. In `claims` mode, list the probes and the beads
# they serve.
# Rationale: the original probe scripts were scratch files lost with
# their scratch directory; packaging them as one runner with a committed
# manifest makes tier-1 membership recomputable at any time — "the
# admissible set is defined by the probes, not by a hardcoded count".
# Like tools/oracle.pl, this lives outside the cargo workspace: the
# corpus, not the toolchain, stays the source of truth.
use strict;
use warnings;

# The pinned oracle (specs/ge-oracle.md c1) — identical to tools/oracle.pl.
my $PIN_VERSION = '0.69';
my $PIN_COMMIT  = 'ededa3d787ad89ac532c578c06390e8a7b270499';
my $PIN         = "Graph::Easy v$PIN_VERSION \@ $PIN_COMMIT";

# Tier-1 envelope (ge-oracle.c7): 10 s wall time.
my $ENVELOPE_SECONDS = 10.0;

# Seeds the hash-stability probe sweeps per fixture. The first entry is
# the env-pinned seed when PERL_HASH_SEED is set (0 otherwise); the rest
# are fixed variants so cross-seed divergence is always visible.
my @SEEDS = (0, 1, 2, 3);

my $mode = shift @ARGV // '';

my $usage = <<"EOF";
probes: usage: perl tests/repro/probes.pl <mode> [args]
  admit <fixtures-dir> [out.tsv]   tier-1 admission sweep over the corpus (ge-oracle.c7/p7)
  claim <name>                     run one claim probe (see: perl tests/repro/probes.pl claims)
  claims                           list probes and the beads they serve
Run under the pinned oracle, e.g.:
  PERL5LIB=/var/tmp/ge$PIN_VERSION/Graph-Easy-$PIN_VERSION/lib perl tests/repro/probes.pl admit tests/fixtures/graph-easy
EOF

# The child probe source: parses one fixture under one seed, renders
# as_txt + as_ascii, prints the digest pair on stdout and the render
# wall time on stderr. Runs as a child process because the hash seed is
# fixed at perl startup — one child per (fixture, seed).
my $CHILD_SRC = <<'CHILD';
use strict; use warnings;
use Graph::Easy::Parser;
use Digest::SHA qw(sha1_hex);
use Time::HiRes qw(time);
my ($file) = @ARGV;
# NOTE: the hash seed is inherited from PERL_HASH_SEED (set by the parent
# before fork) — it is read at perl startup and cannot be set from within.
my $text = do { local $/; open my $fh, '<', $file or die "read: $!"; my $x = <$fh>; close $fh; $x };
my $t0 = time();
# bound the render: the pinned oracle can infinite-loop on some inputs
# (multiline-label nodes hang as_ascii/as_html under every seed), and its
# own default alarm is not always armed. A hung fixture is a render-error,
# never a stalled sweep.
local $SIG{ALRM} = sub { print STDERR "RENDER-ERROR: alarm (oracle hung or exceeded the probe bound)\n"; exit 2 };
alarm 15;
my $g = eval { Graph::Easy::Parser->new->from_text($text) };
if ($@) { print STDERR "RENDER-ERROR: $@"; exit 2; }
my $txt   = $g->as_txt();
my $ascii = $g->as_ascii();
alarm 0;
my $t1 = time();
printf "DIGESTS txt=%s ascii=%s\n", sha1_hex($txt), sha1_hex($ascii);
printf STDERR "SECONDS=%.3f\n", $t1 - $t0;
CHILD

# ge.oracle.c4-style typed pin check: drifted environment is an error
# naming the pin and the remediation, before anything runs. (This file
# never `use`es Graph::Easy — checking $INC catches the *environment*
# drift without importing a possibly-wrong version. The claim probes do
# `use Graph::Easy` below, so version drift surfaces here first.)
my $loaded = eval { require Graph::Easy; $INC{'Graph/Easy.pm'} } // '';
my $version = defined $loaded ? Graph::Easy->VERSION : 'not installed';
unless (defined $version && $version eq $PIN_VERSION) {
    my $remediation = <<"EOF";
probes: loaded Graph::Easy is '$version', pin is $PIN — spec ge.oracle.c1
remediation: fetch the pinned dist isolated and put it on PERL5LIB:
  mkdir -p /var/tmp/ge$PIN_VERSION && cd /var/tmp/ge$PIN_VERSION
  curl -fsSLO https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-$PIN_VERSION.tar.gz
  tar xzf Graph-Easy-$PIN_VERSION.tar.gz
  PERL5LIB=/var/tmp/ge$PIN_VERSION/Graph-Easy-$PIN_VERSION/lib perl tests/repro/probes.pl <mode> ...
EOF
    die $remediation;
}

# ---------------------------------------------------------------------------
# Claim probes: one per upstream-review finding class. Each prints the
# observed upstream behavior and records it to
# tests/repro/claims/<name>.observed. A probe never fails the run — it is
# evidence generation; the falsifiable gate is the admission manifest plus
# the cargo scenario tests. Each probe names the beads it serves.
# ---------------------------------------------------------------------------

# probe -> beads served (the claims listing reads this)
my %SERVES = (
    'sharp-escape'           => 'gently-6j0, gently-bzx (text_parser c7)',
    'sharp-label'            => 'gently-6j0, gently-bzx (text_parser c7)',
    'operator-patterns'      => 'gently-6j0, gently-bzx (text_parser c9, c2)',
    'group-syntax'           => 'gently-6j0, gently-bzx (text_parser c6)',
    'anon-reference'         => 'gently-6j0, gently-bzx (text_parser c1)',
    'layout-flow-direction'  => 'gently-89d (layout c3)',
    'layout-edge-drop'       => 'gently-89d (layout c1/c4 — alarm timeout, silent edge drop at scale)',
    'subgraph-handling'      => 'gently-13f (dot_parser c4 — named/nested/bare-scope subgraphs)',
    'dot-bare-scope-in-named' => 'gently-fb1 (dot_parser c4 — bare scope nested in a named subgraph: group_stack survives bare-scope boundaries)',
    'dot-direction'          => 'gently-13f (dot_parser c1/c2 — header×operator direction matrix)',
    'dot-records-ports'      => 'gently-13f (dot_parser c5 — records, HTML-like labels, port references)',
    'cli-flags'              => 'gently-0h9 (closed — kept as upstream evidence)',
    'node-unnamed'           => 'gently-r22 (graph_model c1)',
    'anon-numbering'         => 'gently-r22 (graph_model c1 — exact #N scheme)',
    'attr-store-decompose'   => 'gently-r22 (graph_model c2 — store-layer unquote + border decomposition)',
    'group-edge'             => 'gently-r22 (graph_model c3 — group endpoints)',
    'group-edge-graphviz'    => 'gently-0kg (graphviz_render c2/c4, txt_render c3 — group-endpoint edges through as_graphviz/as_txt)',
    'deleted-node-add-edge'  => 'gently-r22 (graph_model c3/p3 — deleted-node add_edge)',
    'href-escaping'          => 'gently-dcp, gently-eyo (html_render c2)',
    'td-colspan'             => 'gently-dcp, gently-eyo (html_render c4)',
    'html-edge-styles'       => 'gently-eyo (html_render c3)',
    'html-css-rules'         => 'gently-eyo (html_render c4)',
    'html-colors-shapes'     => 'gently-eyo (html_render c5)',
    'shape-outline-collapse' => 'gently-dcp, gently-css (ascii shapes; html border-styles)',
    'ascii-render-tables'    => 'gently-0cq (ascii_render c1/c2/c5/c6)',
    'boxart-render-tables'   => 'gently-css (boxart_render c1/c2/c3/c4)',
    'attr-quote-value'       => 'gently-8ol (text_parser c5 quote-unquoting)',
    'graphviz-round-trip'    => 'gently-0kg, gently-b4v (graphviz c2/c4, txt_render c3)',
    'size-envelope'          => 'gently-k4u (perf c2/c5)',
    'perl5lib-pin'           => 'gently-ikm, gently-liz (oracle pin contract)',
    'pod-edge-styles'        => 'gently-ni9.1 (POD mining — Easy.pm EXAMPLES edge-styles block is hash-dependent)',
    'pod-fan'                => 'gently-ni9.1 (POD mining — Easy.pm EXAMPLES shared-source fan is hash-dependent)',
    'pod-inline-label'       => 'gently-ni9.1 (POD mining — Parser.pm inline-label example is hash-dependent)',
    'pod-parser-input'       => 'gently-ni9.1 (POD mining — Parser.pm Input section example is hash-dependent)',
    'pod-anon-nodes'         => 'gently-ni9.1 (POD mining — Parser.pm anonymous-node example; gently renders boxed #N, oracle renders boxless → bug bead)',
    'pod-group'              => 'gently-ni9.1 (POD mining — Parser.pm groups example; gently omits the dashed group border box → bug bead)',
    'pod-bidir-label'        => 'gently-7qz (POD mining — Parser.pm \'<-- label -->\' example — start arrowhead, BD gap width, label field; hash-seed-dependent layout, see probe note)',
    'pod-bidir-label-straight' => 'gently-7qz (POD mining — the POD \'<-- label -->\' example without the parallel red edge — the straight labeled-BD pin, hash-stable)',
    'pod-label-attr'         => 'gently-bpq (POD mining — Parser.pm arrowless-label example; gently draws a spurious target arrowhead → bug bead)',
    'pod-repeat-patterns'    => 'gently-ni9.1 (POD mining — Parser.pm repeated-pattern example; gently collapses repeated styles to plain → bug bead)',
    'edge-style-short-runs'  => 'gently-oy2 (POD repeated-pattern family — the full short-edge style run rule: arrowed runs, undirected rx-shifted runs, dot-dot-dash gap widening)',
    'pod-triangle'           => 'gently-ni9.1 + gently-klf (POD mining — Easy.pm interlinked-triangle example; hash-stable 4 seeds; gently\'s vertical back-edge attachments drifted to the node center → fixed at left+2)',
    'pod-unicode'            => 'gently-ni9.1, gently-bvb (POD mining — UTF-8 input prose; oracle width-counts bytes, gently counted chars → bug bead gently-bvb, implemented under gently-527)',
    'pod-style-family'       => 'gently-ni9.1, gently-51d (POD mining — POD wide/broad style example; oracle renders ##>, gently --> → bug bead gently-51d)',
    'edge-style-wide-undirected' => 'gently-51d (the arrowless wide/broad short-edge run — pod-style-family covers only directed runs; pins the undirected ## family bytes)',
    'pod-bidir'              => 'gently-7qz (POD mining — Easy.pm \'<->\' example — the straight bidirectional run rule)',
    'pod-bidir-parallel'     => 'gently-7qz (parallel \'<-->\' pair — the start arrowhead on a bent bidirectional edge)',
    'pod-double-dash'        => 'gently-ni9.1 (POD mining — POD \'= >\' double-dash example; gently collapses to --> → bug bead)',
    'gh-4-nodes-first'       => 'gently-ni9.3, gently-a0k, gently-2n5 (GitHub issue #4 — nodes declared first; graph-scope attrs rankdir→flow → gently-a0k, bare-scope edge endpoints → gently-2n5)',
    'bg-multiline-align'     => 'gently-ni9.2, gently-y7i (bloodgate mining — syntax.html \l\r\c alignment escapes and \r/\n label splits; gently prints them raw → bug bead gently-y7i; the split/align rule is implemented under gently-527)',
    'bg-comma-lists'         => 'gently-m4i (bloodgate mining — syntax.html comma-chained node lists; gently parse-errors → bug bead)',
    'bg-trailing-node-attr'  => 'gently-8xd (bloodgate mining — syntax.html trailing node attr block silently dropped → bug bead)',
    'pod-anon-group'         => 'gently-58m (POD mining — Parser.pm anonymous-group example; gently parse-errors on the following edge → bug bead)',
    'bg-anon-chain'          => 'gently-ni9.2 (bloodgate mining — syntax.html anon nodes in chain; gently renders boxed #N, oracle hides them → bug bead)',
    'bg-minlen-bypass'       => 'gently-ni9.2 (bloodgate mining — tutorial_bypass.html minlen:3; the bypass SHAPE diverges — ranking, see re-derivation note; minlen is incidental, gently-bg-minlen-bypass-shape; re-derived as the shared-middle bypass ranking bug gently-jrr)',
    'bg-minlen-stretch'      => 'gently-ni9.2 + gently-yhg (re-derivation probe — minlen >= 2 widens the straight gap exactly one column; minlen 2..5 identical, 4-seed stable)',
    'bg-edge-node'           => 'gently-ni9.2 (bloodgate mining — tutorial_edges.html shape:edge nodes; gently path-cells render error → bug bead gently-cbk; oracle renders the edge-shaped node merged into its edge, see research notes)',
    'bg-joint-classes'       => 'gently-ni9.2 (bloodgate mining — syntax.html joint-node class:border example; PINNED PARSER REJECTS it — site rendered by a newer Graph::Easy)',
    'bg-start-right'         => 'gently-ni9.2 (bloodgate mining — tutorial_bypass.html start:right port attr; gently ignores it → bug bead gently-89r. DIS-ENTANGLED: the inline-Trier layout is the shared-middle bypass shape, NOT the port effect — the no-port chain renders identically, see port-side-minimal; the layout shape is gently-jrr, the port attachment is gently-89r)',
    'bg-textwrap'            => 'gently-emd (bloodgate mining — syntax.html textwrap:auto + textwrap:10; the pinned bytes ground the wrap re-derivation)',
    'so-graph-convert'       => 'gently-5vf (community mining — SO 71869025 accepted answer: Graph.pm→Graph::Easy URI tree; stem column pins the left+2 rule)',
    'so-edge-labels'         => 'gently-ni9.4 (community mining — SO 47661180: bare-number edge labels on a text-frequency digraph)',
    'so-flowchart-size'      => 'gently-ni9.4, gently-527 (community mining — SO 47323345 question: { size: 5,10; } node attrs + literal-\\n edge label printed raw → gently-527 owns the label-split effect; the { size: attrs and the 2x2 layout divergence stay out of scope)',
    'so-start-port-index'    => 'gently-ni9.4 (community mining — SO 47323345 answer: {start: front,N} indexed start/end port attrs ignored → gently-byq; side mapping front = east flow-space front, gently-89r owns the unindexed side routing)',
    'so-graphviz-unknown-attrs' => 'gently-ni9.4 (community mining — SO 59659560: Graph::Easy::Parser::Graphviz unknown dot attrs warn and render anyway)',
);

my %CLAIMS = (
    # GitHub-mining claims (gently-ni9.3): reproducible inputs from the
    # upstream GitHub issue tracker, rendered under the pinned oracle.
    # bloodgate.com-mining claims (gently-ni9.2): inputs mined from the
    # live bloodgate.com Graph::Easy pages (manual/syntax.html,
    # manual/tutorial_*.html — the rendered Graph::Easy::Manual POD).
    # Each renders under the pinned oracle across seeds 0/1; gently's
    # divergence is named per probe.
    # byte-compat claim (gently-527 run): the minimal byte-width pin —
    # the oracle length()s label text in UTF-8 BYTES, so the Köln box is
    # 7 dashes wide where a char count draws 6. Kept on its own graph so
    # the width rule is pinned unentangled (gently-bvb).
    'label-byte-width' => <<'PROBE',
# serves: gently-bvb
# claim probed: the byte-width rule on its own graph — upstream measures
# label text with a BYTE-string length() when the input is raw UTF-8
# bytes (unflagged, as the pinned ascii-render-tables probe passes its
# \\x{}-escape input and as gently's byte-compat scope reads it), so the
# multi-byte 'ö' makes the Köln box one column WIDER than a char count
# would (7 dashes). NOTE: a UTF-8-DECODED input string (a :utf8 read
# layer, as pod_probe_body's children use and pod-unicode.observed
# records) flips upstream's length() to chars — the same oracle renders
# a 6-dash box there; the byte-compat pin is the RAW-BYTES path, matching
# the pinned tables (中 renders `+-----+`, 3 bytes + 4).
# Observed (pinned oracle, raw-bytes input): `+-------+` with the label
# row 9 bytes (`| Köln |`).
use Graph::Easy::Parser;
my $example = "[ A ] --> [ K\xc3\xb6ln ]\n";
for my $seed (0, 1) {
  print "== seed $seed ==\n";
  local $ENV{PERL_HASH_SEED} = $seed;
  # raw BYTES into from_text — an undecoded (unflagged) byte string,
  # the same shape gently's parser reads the input as
  my $g = Graph::Easy::Parser->new->from_text($example);
  my $ascii = $g->as_ascii();
  # the render is a raw byte string; the output capture runs a :utf8
  # layer, so flag the bytes as UTF-8 (in place, no byte change) and
  # print — the recorded bytes stay the render's bytes
  utf8::decode($ascii);
  print "as_ascii:\n", $ascii;
}
PROBE


    # byte-compat claim (gently-527 run): the label-split DIS-ENTANGLEMENT
    # — so-flowchart-size.observed couples the \n split with { size: }
    # attrs and a 2x2 layout; this probe pins the \n-split effect on its
    # own graph (two anonymous-name-free nodes, no attrs): two left-aligned
    # rows ride the band's top two lines, the arrow on the grown band's
    # H-2 line, both boxes one blank row taller.
    'edge-label-split-minimal' => <<'PROBE',
# serves: gently-527
# claim probed: the minimal literal-\n edge-label split, disentangled
# from so-flowchart-size's { size: 5,10; } attrs and 2x2 layout (out of
# scope). Observed (pinned oracle): 'put msg\n via http' renders as two
# left-aligned rows above the arrow; both node boxes grow one blank
# interior row; the gap is the widest line (8) + 5 = 13 columns.
pod_probe_body('edge-label-split-minimal', <<'EXAMPLE')
[ A ] -- put msg\n via http --> [ B ]
EXAMPLE
PROBE

    'bg-multiline-align' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/syntax.html multiline-label section.
# Observed (pinned oracle): \l / \r / \c become per-line alignment inside
# the node box, \n / \r split the edge label across lines. gently prints
# every escape raw on one line.
pod_probe_body('bg-multiline-align', <<'EXAMPLE')
[ Long Node Label\l left\r right\c center ] -- A\r long\n edge label --> [ B ]
EXAMPLE
PROBE

    'pod-anon-group' => <<'PROBE',
# serves: gently-58m
# claim probed: Graph::Easy::Parser POD 'groups' section anonymous-group
# example: '( [ Bonn ] ) -> [ Berlin ]' creates the anon group containing
# Bonn and links it to Berlin. Observed (pinned oracle): the group-end
# handler makes the closed group the node stack, so the edge links the
# GROUP to Berlin; as_txt renders the group block only, as_ascii renders
# the group-to-node edge. gently: parse error, line 1: '-' not recognized.
pod_probe_body('pod-anon-group', <<'EXAMPLE')
( [ Bonn ] ) -> [ Berlin ]

( [ Bonn ] { color: red; } )
EXAMPLE
PROBE

    'bg-comma-lists' => <<'PROBE',
# serves: gently-m4i
# claim probed: bloodgate.com/manual/syntax.html comma-chained node lists
# (shared-source fan-out and multi-source fan-in). Observed: oracle lays
# them out with the extra node(s) hanging off the chain. gently:
# parse error, line 1: ',' not recognized.
pod_probe_body('bg-comma-lists', <<'EXAMPLE')
[ A ] -> [ B ], [ C ], [ D ]

[ A ], [ B ], [ C ] --> [ D ]

[ A ], [ B ] { color: red; }

[ A ] --> [ B ]
, [ C ]
EXAMPLE
PROBE

    'bg-trailing-node-attr' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/syntax.html quoting section
# (21" edge label, "Huge" node label). Observed (pinned oracle): the
# trailing node attribute block { label: "Huge"; } renames Big → Huge;
# the 21" label renders quoted-free. gently drops the trailing node
# attribute block entirely and renders Big. Edge attr blocks mid-chain
# DO work (see examples/edge-attributes) — trailing NODE blocks don't.
pod_probe_body('bg-trailing-node-attr', <<'EXAMPLE')
[ "Monitor Size" ] --> { label: 21"; } [ Big ] { label: "Huge"; }
EXAMPLE
PROBE

    'bg-anon-chain' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/syntax.html anonymous nodes in a
# chain. Observed (pinned oracle): anon nodes render INVISIBLE — only
# Karlsruhe/Plauen boxes plus the floating incoming/outgoing arrows.
# gently renders boxed #1/#3 nodes (anon-numbering family, gently-2wt).
pod_probe_body('bg-anon-chain', <<'EXAMPLE')
[ ] -> [ Karlsruhe ] -> [ ] -> [ Plauen ]
EXAMPLE
PROBE

    'bg-minlen-bypass' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/tutorial_bypass.html minlen example
# (Bonn→Koblenz→{minlen:3}→Frankfurt→Dresden plus Koblenz→Trier→Frankfurt
# bypass). Observed (pinned oracle): the Trier bypass routes across the
# TOP with Trier INLINE on the row.
# RE-DERIVATION (gently-klf run): the SAME input WITHOUT minlen:3
# renders byte-identically — minlen is INCIDENTAL here; the divergence
# gently must close is the shared-middle bypass SHAPE (Koblenz, Trier,
# Frankfurt inline ranks with the skip-edge over the top), i.e. a
# longest-path ranking divergence, not a minlen one. See
# bg-minlen-stretch for minlen's actual effect.
pod_probe_body('bg-minlen-bypass', <<'EXAMPLE')
[ Bonn ] --> [ Koblenz ] --> { minlen: 3; } [ Frankfurt ] --> [ Dresden ]

[ Koblenz ] --> [ Trier ] --> [ Frankfurt ]
EXAMPLE
PROBE

    'bg-minlen-stretch' => <<'PROBE',
# serves: gently-ni9.2, gently-yhg
# claim probed (re-derivation, gently-klf run): minlen's actual effect —
# a two-node straight edge. Observed (pinned oracle, 4-seed stable):
# minlen >= 2 widens the straight gap by exactly ONE column (the run
# fills the doubled gap); minlen 0/1 stay adjacent; minlen 2, 3, 4 and 5
# render identically, so the stretch saturates at one column.
pod_probe_body('bg-minlen-stretch', <<'EXAMPLE')
[ Bonn ] --> { minlen: 3; } [ Berlin ]
EXAMPLE
PROBE

    'bg-edge-node' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/tutorial_edges.html edge-as-node
# example (train/car { shape: edge; } nodes fed by anon buy/rent edges).
# Observed (pinned oracle): shape:edge nodes dissolve into their edges;
# anon feed nodes invisible. gently: render error "edge path cells are
# not orthogonally adjacent".
pod_probe_body('bg-edge-node', <<'EXAMPLE')
[ Bonn ] -- [ train ] --> [ Ulm ] -- [ car ] --> [ Hof ]

[ ] -- buy --> [ train ] { shape: edge; }
[ ] -- rent --> [ car ] { shape: edge; }
EXAMPLE
PROBE

    # bloodgate divergence (oracle-side, NOT a gently bug): the site's
    # joint-node class:border example is rendered by a NEWER Graph::Easy —
    # the pinned v0.69 parser rejects 'class: |1' outright. Documented so
    # nobody mistakes the site bytes for pin-reachable output.
    'bg-joint-classes' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/syntax.html joint-node class example
# (node.1..3 border styles, [ AAAA|BBBB ] { class: |1; } ...). Observed:
# the PINNED v0.69 parser REJECTS the input — "'1' is not a valid class
# for a node" — so the site's shown boxart (dotted/dashed/dot-dash cell
# borders) is NOT pin-reachable. The site runs a newer Graph::Easy.
use Graph::Easy::Parser;
my $text = <<'EXAMPLE';
node.1 { border: dotted; }
node.2 { border: dashed; }
node.3 { border: dot-dash; }
node { border: double; }

[ AAAA|BBBB ] { class: |1; }
[ CCCC|DDDD ] { class: 2|; }
[ EEEE|FFFF ] { class: 3; }
EXAMPLE
for my $seed (0, 1) {
  print "== seed $seed ==\n";
  local $ENV{PERL_HASH_SEED} = $seed;
  my $g = eval { Graph::Easy::Parser->new->from_text($text) };
  if ($@) { my ($m) = split /\n/, $@; $m =~ s/ at .* line \d+\.$//; print "ERR  pinned parser rejects: $m\n"; next; }
  print $g->as_ascii();
}
PROBE

    'bg-start-right' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/tutorial_bypass.html start-port
# example (Bonn→Koblenz→Frankfurt→Dresden plus { start: right; }
# Koblenz→Trier→Frankfurt). Observed (pinned oracle): the port attr
# makes Trier sit INLINE on Koblenz's right, edge leaving horizontally.
# gently ignores start: and routes Trier below.
pod_probe_body('bg-start-right', <<'EXAMPLE')
[ Bonn ] --> [ Koblenz ] --> [ Frankfurt ] --> [ Dresden ]

[ Koblenz ] --> { start: right; } [ Trier ] --> [ Frankfurt ]
EXAMPLE
PROBE

    'bg-textwrap' => <<'PROBE',
# serves: gently-ni9.2
# claim probed: bloodgate.com/manual/syntax.html textwrap section
# (graph { textwrap: auto; } plus edge attr { textwrap: 10; } — note the
# SITE shows 'text-wrap: 10' but the PINNED parser only accepts
# 'textwrap'; the site page itself is newer).
# Observed (pinned oracle): node label wrapped into a 7-row box at 10
# columns — auto wraps the 33-byte label at int(sqrt(33)*1.4) = 8, the
# edge's own textwrap: 10 overrides the inherited mode. RESOLVED
# (gently-emd): gently implements _wrapped_label byte-identically.
pod_probe_body('bg-textwrap', <<'EXAMPLE')
graph { textwrap: auto; }

[ Long Node Label left right center ] -- A long edge label --> { textwrap: 10; } [ B ]
EXAMPLE
PROBE

    # Stack Overflow-mining claims (gently-ni9.4): real-world inputs pasted
    # in SO questions/answers, rendered under the pinned oracle across
    # seeds 0/1. Community inputs are the messy edge cases POD never shows.
    'so-graph-convert' => <<'PROBE',
# serves: gently-5vf
# claim probed: SO 71869025 / answer 71869162 (accepted) — converting a
# Graph.pm URI tree to Graph::Easy and printing as_ascii. Parsed input
# equivalent of the API-built edges ('/', '/a'), ('/', '/c'), ('/a', '/b').
# Observed (pinned oracle): /a → /b on the top row, /c hanging below /
# with the stem at /'s left+2 column — matches the bytes the answerer
# posted. RESOLVED (gently-klf left+2 rule; pinned gently-5vf): gently
# renders the stem column byte-identically.
pod_probe_body('so-graph-convert', <<'EXAMPLE')
[ / ] --> [ /a ]
[ / ] --> [ /c ]
[ /a ] --> [ /b ]
EXAMPLE
PROBE

    'so-edge-labels' => <<'PROBE',
# serves: gently-ni9.4
# claim probed: SO 47661180 / answer 47661977 — bare-number edge labels
# from text-frequency counts (the asker's 'frequency' attr was invalid;
# the answerer relabeled edges with the counts). Observed (pinned
# oracle): numeric labels ride above the arrowheads, three parallel
# chains laid out side by side.
pod_probe_body('so-edge-labels', <<'EXAMPLE')
[ 14 ] -- 2 --> [ million ]
[ digital ] -- 2 --> [ humanities ] -- 2 --> [ research ]
[ participants ] -- 2 --> [ may ] -- 3 --> [ want ]
EXAMPLE
PROBE

    'so-flowchart-size' => <<'PROBE',
# serves: gently-ni9.4
# claim probed: SO 47323345 (question body) — a real flowchart input with
# { size: 5,10; } node attribute blocks (comma inside the braces plus a
# trailing semicolon), a literal '\n' escape inside an edge label, and a
# label continued onto the next line with leading whitespace. Observed
# (pinned oracle): parses clean; the \\n splits the 'put msg / via http'
# label over two centered rows. gently: prints the \\n escape raw on one
# line, different layout → bug bead gently-527.
pod_probe_body('so-flowchart-size', <<'EXAMPLE')
[Client] { size: 5,10; }
[Msg Server] { size: 5,10; }
[Server]

[Server] -- put msg\n
            via http  -->  [Msg Server]


[Msg Server] -- 1. Number X of unread --> [Client]
[Msg Server] -- 3. X Msg --> [Client]

[Client] -- 2. read X --> [Msg Server]
EXAMPLE
PROBE

    'port-side-minimal' => <<'PROBE',
# serves: gently-89r
# claim probed: DIS-ENTANGLEMENT of the tutorial_bypass start:right port
# effect from the shared-middle bypass shape (gently-jrr). Example 1 (the
# chain WITHOUT the port attr): the pinned oracle already lays Trier
# inline and routes Koblenz->Frankfurt over the top — the bypass layout
# is independent of the port attribute, so gently-jrr owns that shape
# and gently-89r must not pin it. Example 2 (a lone { start: right; }
# edge): the port's OWN effect is attachment-only — in east flow
# `right` = inflow+90 = SOUTH, so the edge leaves the source's south
# side, runs along the gap row below, and climbs into the target's
# south with ^ (end: right renders identically; ports never change node
# placement).
pod_probe_body('port-side-minimal-chain', <<'EXAMPLE');
[ Bonn ] --> [ Koblenz ] --> [ Frankfurt ] --> [ Dresden ]

[ Koblenz ] --> [ Trier ] --> [ Frankfurt ]
EXAMPLE
pod_probe_body('port-side-minimal-edge', <<'EXAMPLE')
[ A ] --> { start: right; } [ B ]
EXAMPLE
PROBE

    'so-start-port-index' => <<'PROBE',
# serves: gently-ni9.4
# claim probed: SO 47323345 / answer 48119766 — the community workaround
# for label ordering uses INDEXED port attrs ({start: front,0},
# {start: front,3}, {end: front,2}) glued to the target with NO space
# ('{...}[Client]'). Observed (pinned oracle): edges land at the indexed
# ports; parser accepts the zero-space form. gently: ignores the port
# index (bg-start-right family) and draws a loop layout → bug bead
# gently-byq.
pod_probe_body('so-start-port-index', <<'EXAMPLE')
[Msg Server] -- 1. Number X of unread --> {start: front,0}[Client]
[Msg Server] -- 3. X Msg --> {start: front,3} [Client]
[Client] -- 2. read X --> {end: front,2}[Msg Server]
EXAMPLE
PROBE

    'so-graphviz-unknown-attrs' => <<'PROBE',
# serves: gently-ni9.4
# claim probed: SO 59659560 — converting a go-callvis dot dump with
# Graph::Easy::Parser::Graphviz. Real dot dumps carry attributes v0.69
# does not know (lheight, lwidth, penwidth). Observed (pinned oracle):
# 'Ignoring unknown attribute ... for class graph/node' WARNINGS on
# stderr while the render still succeeds — warnings, not errors.
# gently: dot parser rejects the attribute-statement form.
use Graph::Easy::Parser::Graphviz;
my $dot = <<'DOT';
digraph G {
  graph [lheight=0.37, lwidth=0.37, penwidth=0.8];
  node [penwidth=0.8];
  a -> b;
  a -> c;
}
DOT
my @warns;
local $SIG{__WARN__} = sub { push @warns, $_[0] };
my $g = Graph::Easy::Parser::Graphviz->from_text($dot);
print "warnings: ", scalar(@warns), "\n";
for my $w (@warns) { my ($m) = split /\n/, $w; print "  WARN $m"; }
print $g->as_ascii();
PROBE

    'gh-4-nodes-first' => <<'PROBE',
# serves: gently-ni9.3, gently-a0k, gently-2n5 (GitHub issue #4, closed
# upstream — still reproduces against the PINNED oracle v0.69, which is the
# contract).
# claim probed: issue #4's two inputs. The reported bug reproduces exactly:
# input 'a' (nodes declared first) yields the spurious self-edge
# '[ baz ] --> [ baz ]' and the wrong drawing; input 'b' (no declarations)
# is correct. Hash-stable across seeds 0/1 — but tier-1 INADMISSIBLE as a
# corpus fixture: gently parse-errors on BOTH inputs (graph-scope attribute
# statement 'rankdir="LR"' and bare-scope edge endpoint '{foo bar} -> baz'),
# so no byte-comparison is even reachable yet. Divergence beads: dot parser
# missing graph-level attribute statements (gently-a0k); bare scope
# unsupported as an edge-chain endpoint (gently-2n5). The oracle's rule,
# re-derived for gently (see specs/ge-dot_parser.md c3/c4): a graph-scope
# 'key=value' statement applies to the graph with rankdir remapped to flow
# (LR→east, RL→west, TB→south, BT→north, unknown→east); the node stack
# survives scope boundaries, an edge operator links EVERY node on it, and
# a plain '}' resets it — so the leaked pre-declared baz links itself
# first, then both scope nodes.
use Graph::Easy::Parser::Graphviz;
my $p = Graph::Easy::Parser::Graphviz->new();
my @cases = (
  [ 'nodes declared first', <<'A' ],
digraph {
  rankdir="LR"

  foo
  bar [label="bar-label!"]
  baz

  {foo bar} -> baz
}
A
  [ 'no declarations',      <<'B' ],
digraph {
  rankdir="LR"
  {foo bar} -> baz
}
B
);
for my $c (@cases) {
  my ($label, $text) = @$c;
  my $g = eval { $p->from_text($text) };
  if ($@) { my ($m) = split /\n/, $@; $m =~ s/ at \(eval \d+\) line \d+\.//; print "ERR  [$label]\n     $m\n"; next; }
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//;
  print "OK   [$label]\n     as_txt:\n$txt\n";
  print "     as_ascii:\n", $g->as_ascii();
}
PROBE

    # POD-mining claims (gently-ni9.1): the example is tier-1 INADMISSIBLE
    # (hash-dependent across seeds, ge.oracle.c7), so it ships as a claim
    # probe, not a corpus fixture. Each probe renders the POD example under
    # two hash seeds and shows the divergence the oracle itself cannot pin.
    'pod-edge-styles' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy.pm POD "Different edge styles" EXAMPLES block.
# Observed: seed-dependent layout divergence (bidirectional/double edges).
pod_probe_body('pod-edge-styles', <<'EXAMPLE')
[ Bonn ] <-> [ Berlin ]
[ Berlin ] ==> [ Rostock ]
[ Hamburg ] ..> [ Altona ]
[ Dresden ] - > [ Bautzen ]
[ Leipzig ] ~~> [ Kirchhain ]
[ Hof ] .-> [ Chemnitz ]
[ Magdeburg ] <=> [ Ulm ]
[ Magdeburg ] -- [ Ulm ]
EXAMPLE
PROBE

    'pod-fan' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy.pm POD "Three nodes" EXAMPLES block (fan-out).
# Observed: seed-dependent layout divergence.
pod_probe_body('pod-fan', <<'EXAMPLE')
[ Bonn ] -> [ Berlin ]
[ Bonn ] -> [ Hamburg ]
EXAMPLE
PROBE

    'pod-inline-label' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy::Parser POD inline edge-label example
# ("-- foo -->" / "... baz ...>" styles).
# Observed: seed-dependent layout divergence.
pod_probe_body('pod-inline-label', <<'EXAMPLE')
[ Bonn ] -- label --> [ Berlin ]
[ Bonn ] ... baz ...> [ Hamburg ]
EXAMPLE
PROBE

    'pod-double-dash' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: POD edge-styles table row "= > double-dash"
# (Parser.pm edges section). Observed: the oracle round-trips '[ A ] = >
# [ B ]' verbatim in as_txt; gently collapses the style to '-->' —
# fixture rejected, divergence bug (style-loss family).
pod_probe_body('pod-double-dash', <<'EXAMPLE')
[ A ] = > [ B ]
EXAMPLE
PROBE

    'pod-bidir' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy.pm POD "Two nodes" variant — the '<->'
# bidirectional edge from the edge-style list. Observed: the oracle draws
# '<-->' with both arrowheads; gently draws '-->' — same start-arrowhead
# root cause as the pod-bidir-label bug (gently-ni9.1 filed bead).
pod_probe_body('pod-bidir', <<'EXAMPLE')
[ Bonn ] <-> [ Berlin ]
EXAMPLE
PROBE

    'pod-style-family' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: POD prose "There are also the styles bold, wide and broad"
# (Parser.pm edges section), all three in one chain.
# Observed: the oracle renders wide AND broad as '##>' (double-line
# arrowhead); gently renders plain '-->' — fixture rejected, divergence bug.
pod_probe_body('pod-style-family', <<'EXAMPLE')
[ AB ] --> { style: bold; } [ ABC ]
[ ABC ] --> { style: wide; } [ ABCD ]
[ ABCD ] --> { style: broad; } [ ABCDE ]
EXAMPLE
PROBE

    'edge-style-wide-undirected' => <<'PROBE',
# serves: gently-51d
# claim probed: the ARROWLESS (undirected) wide/broad short-edge run —
# pod-style-family pins only the directed '##>' runs. Observed: wide AND
# broad render the bold-family '###' fill (undirected, rx-shifted like
# bold); upstream As_ascii $edge_styles rows wide/broad are byte-identical
# to bold ('##') and _correct_size does NOT widen their gap (only
# dot-dot-dash).
pod_probe_body('edge-style-wide-undirected', <<'EXAMPLE')
[ A ] -- { style: wide; } [ B ] -- { style: broad; } [ C ]
EXAMPLE
PROBE

    'pod-unicode' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: POD prose "The input ... encoded in UTF-8" (Parser.pm Input).
# Observed: the oracle as_ascii sizes the 'Köln' node box by UTF-8 BYTE
# length (extra column per multi-byte char); gently sizes by chars —
# fixture rejected, divergence bug (unicode labels).
pod_probe_body('pod-unicode', <<'EXAMPLE')
[ Bonn ] --> [ Köln ]
[ Köln ] --> [ Zürich ]
EXAMPLE
PROBE

    'pod-triangle' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy.pm POD "Three nodes, interlinked" example
# (the triangle). Observed: hash-stable in the oracle, but gently's layout
# differs by 2 columns on the back-edge — fixture rejected, divergence bug.
pod_probe_body('pod-triangle', <<'EXAMPLE')
[ Bonn ] -> [ Berlin ]
[ Berlin ] -> [ Hamburg ]
[ Bonn ] -> [ Hamburg ]
EXAMPLE
PROBE

    'pod-repeat-patterns' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy::Parser POD repeated edge-style patterns
# ("--->", "~~~~~>", "..-..-..->").
# Observed: the oracle renders '..-..-..->' with dot-dot-dash styling
# ('..->') and extra column spacing; gently collapses it to plain '-->' —
# fixture rejected, divergence bug.
pod_probe_body('pod-repeat-patterns', <<'EXAMPLE')
[ A ] ---> [ B ]
[ B ] ~~~~~> [ C ]
[ C ] ..-..-..-> [ D ]
EXAMPLE
PROBE

    'edge-style-short-runs' => <<'PROBE',
# serves: gently-oy2
# claim probed: the full short-edge style run rule behind the repeated
# patterns (POD '..-..-..->' / '.-.-.->' / '= = >'). Pins, for the
# minimum straight-edge gap (rx = 5): arrowed dot-dash '.->', dot-dot-dash
# '..->' (6-wide gap — upstream _correct_size widens w for the style),
# double-dash '= >'; arrow-less (undirected) dot-dash '-.-', dot-dot-dash
# '-..-', double-dash ' = ' (runs shift with the gap's absolute x —
# upstream _draw_hor's ofs removal applies to undirected SHORT cells
# only, so gap 2 of a chain renders '..-.').
# Custom body (not pod_probe_body): each example is its OWN graph —
# combining them into one graph yields 7 parallel a..b edges whose layout
# is hash-seed-dependent (that is the pod-repeat-patterns rejection), so
# each input is probed in isolation and annotated with its source text.
use Graph::Easy::Parser;
my @examples = (
  '[ a ] .-.-.-> [ b ]',
  '[ a ] ..-..-..-> [ b ]',
  '[ a ] = = > [ b ]',
  '[ a ] .-.- [ b ]',
  '[ a ] ..-..- [ b ]',
  '[ a ] = = [ b ]',
  '[ a ] --> [ b ] ..-..- [ c ]',
);
for my $seed (0, 1) {
  print "== seed $seed ==\n";
  local $ENV{PERL_HASH_SEED} = $seed;
  for my $text (@examples) {
    my $g = eval { Graph::Easy::Parser->new->from_text($text) };
    die "parse failed: $text\n" unless $g;
    print "-- $text --\n";
    print $g->as_txt();
    print $g->as_ascii();
  }
}
PROBE

    'pod-label-attr' => <<'PROBE',
# serves: gently-bpq
# claim probed: Graph::Easy::Parser POD arrowless edge with label attribute
# ("[ AB ] -- { label: edgelabel; } [ CD ]").
# Observed: the oracle as_ascii renders NO arrowheads on the '--' edge —
# the undirected labeled run is the fill pattern only (upstream _draw_hor
# draws no head without EDGE_END_* flags); gently appends a spurious '>'
# at the target end (gently-bpq).
pod_probe_body('pod-label-attr', <<'EXAMPLE')
[ AB ] -- { label: edgelabel; } [ CD ]
EXAMPLE
PROBE

    'pod-bidir-label' => <<'PROBE',
# serves: gently-7qz
# claim probed: Graph::Easy::Parser POD bidirectional inline-label example
# with the POD's parallel red edge ("[ A ] <-- label --> [ B ]" plus
# "[ A ] --> { color: red; } [ B ]").
# Observed: the oracle draws arrowheads at BOTH endpoints (seed 0: 'v' at
# the source of the bent route; seed 1: 'v' at the bent edge's target)
# and, on the straight route, widens the gap by one column for the
# bidirectional run (upstream _correct_size w++ for BD_HOR cells), draws
# ' <...> ' with BOTH heads, and hosts the label centered in [2, w-3)
# (upstream _draw_hor/_insert_label). CAVEAT: which of the two parallel
# edges routes straight is hash-seed (and run) dependent — the two seeds
# pin the two legitimate layouts, but a re-record can flip seed 1; the
# hash-stable pin for the straight labeled-BD rule is
# pod-bidir-label-straight (gently-7qz).
pod_probe_body('pod-bidir-label', <<'EXAMPLE')
[ A ] <-- label --> [ B ]
[ A ] --> { color: red; } [ B ]
EXAMPLE
PROBE

    'pod-bidir-label-straight' => <<'PROBE',
# serves: gently-7qz
# claim probed: the POD bidirectional inline-label example on its own
# ("[ A ] <-- label --> [ B ]", no parallel edge) — the hash-stable pin
# for the straight labeled-BD rule (pod-bidir-label's two-edge form is
# run-nondeterministic in which edge routes straight).
# Observed: hash-stable across seeds 0 and 1. The oracle widens the gap
# to 11 (label 5 + 5 + 1 for the BD cell, upstream _correct_size), draws
# ' <-------> ' with BOTH arrowheads (upstream _draw_hor END_W/END_E),
# and hosts the label at cell-x 2 (centered in [2, w-3), upstream
# _insert_label).
pod_probe_body('pod-bidir-label-straight', <<'EXAMPLE')
[ A ] <-- label --> [ B ]
EXAMPLE
PROBE

    'pod-bidir-parallel' => <<'PROBE',
# serves: gently-7qz
# claim probed: two parallel '<-->' edges between one node pair (the
# bidirectional analogue of the recorded parallel-elbow family).
# Observed: hash-stable across seeds. The oracle draws the straight edge
# as ' <--> ' (BD gap width 6, both arrowheads) and the bent edge with a
# 'v' start arrowhead at the source's north side mirroring the 'v' end
# arrowhead at the target — the polyline start-arrow rule gently-7qz
# implements.
pod_probe_body('pod-bidir-parallel', <<'EXAMPLE')
[ A ] <--> [ B ]
[ A ] <--> [ B ]
EXAMPLE
PROBE

    'pod-group' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy::Parser POD named-group example
# ("( Cities: [ Bonn ] -> [ Berlin ] )").
# Observed: the oracle as_ascii draws a dashed '+ - - -+' border box with
# the group title; gently renders only the member nodes, no border —
# byte-compat fixture rejected, filed as a divergence bug.
pod_probe_body('pod-group', <<'EXAMPLE')
( Cities:

  [ Bonn ] -> [ Berlin ]
)
EXAMPLE
PROBE

    'pod-anon-nodes' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy::Parser POD anonymous-nodes example
# ("[ ] -> [ Bonn ] -> [ ]").
# Observed: the oracle's as_ascii draws anonymous nodes with NO border and
# NO name; gently renders them as boxed '#1'/'#3' — byte-compat fixture
# rejected, filed as a divergence bug.
pod_probe_body('pod-anon-nodes', <<'EXAMPLE')
[ ] -> [ Bonn ] -> [ ]
EXAMPLE
PROBE

    'pod-parser-input' => <<'PROBE',
# serves: gently-ni9.1
# claim probed: Graph::Easy::Parser POD Input section example
# (<=>, = > double-dash forms).
# Observed: seed-dependent layout divergence.
pod_probe_body('pod-parser-input', <<'EXAMPLE')
[ Bonn ]      --> [ Berlin ]
[ Frankfurt ] <=> [ Dresden ]
[ Bonn ]      --> [ Frankfurt ]
[ Bonn ]      = > [ Frankfurt ]
EXAMPLE
PROBE

    'sharp-escape' => <<'PROBE',
# serves: gently-6j0, gently-bzx (text_parser c7)
# claim probed: the text_parser spec's sharp/quote escaping rows.
use Graph::Easy::Parser;
# Observed: a node named '#a' — upstream escapes the sharp as '\#' in
# as_txt, and the escaped name round-trips through the parser back to '#a'.
use Graph::Easy;
my $g = Graph::Easy->new();
$g->add_edge('#a', 'b');
print "as_txt:\n", $g->as_txt();
my $rt = Graph::Easy::Parser->new->from_text($g->as_txt());
print "round-trip node('#a'): ", (defined $rt->node('#a') ? 'found' : 'LOST'), "\n";
PROBE

    'sharp-label' => <<'PROBE',
# serves: gently-6j0, gently-bzx (text_parser c7)
# claim probed: c7 — the in-string sharp rule and the hex-colour special case.
# Observed: an UNescaped in-string '#' truncates the line at the '#' (it
# acts as a comment to end-of-line) even inside QUOTED attribute values —
# '{ label: x # y }' is a parse error. '\#' escapes it and the value
# round-trips back to '#'. Special case: a 3- or 6-digit hex colour token
# immediately after the attribute separator is AUTO-ESCAPED by the oracle
# and accepted; the same token NOT after a separator acts as a comment.
use Graph::Easy::Parser;
my @cases = (
  '[ a ] { label: x # y; } --> [ b ]',
  '[ a ] { label: x \# y; } --> [ b ]',
  '[ a ] { label: "x # y"; } --> [ b ]',
  '[ a ] { label: "x \# y"; } --> [ b ]',
  '[ a ] { color: #ff0000; } --> [ b ]',
  '[ a ] { color: #f00; } --> [ b ]',
  '[ a ] { color: red #ff0000; } --> [ b ]',
);
for my $c (@cases) {
  my $g = eval { Graph::Easy::Parser->new->from_text($c) };
  if ($@) { my ($m) = split /\n/, $@; print "ERR  [$c]\n     $m\n"; next; }
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$c]\n     as_txt: $txt\n";
}
PROBE

    'operator-patterns' => <<'PROBE',
# serves: gently-6j0, gently-bzx (text_parser c9, c2)
# claim probed: c9 — the repetition/arrow-less rules; c2 — bidirectional
# operators and the no-left-only-edges rule.
# Observed: a directed pattern is ONE OR MORE unit tokens (each of '= ',
# '=', '- ', '-', '..-', '.-', '.', '~') followed by '>', and the STYLE is
# determined solely by the LAST unit — mixed repetitions are accepted, so
# '..-..-..>' is a VALID dotted edge (the upstream POD's claim that the
# whole pattern must be repeated is contradicted by the oracle itself).
# Arrow-less: '.-' and '..-' are valid with a SINGLE repetition (dot-dash,
# dot-dot-dash); the plain units need at least TWO repetitions ('.' and
# '=' are errors with one). Bidirectional: a '<' prefix before any
# directed pattern ('<->' renders '<-->', '<= >' stays '<= >'); a lone
# left arrow without the closing '>' is a parse error, and an operator
# with a node missing on either side is a parse error.
use Graph::Easy::Parser;
my @wrapped = (   # wrapped as '[ a ] X [ b ]'
  '..-..-..>', '..-..-..->', '.-..-..>', '.--->', '--.>',
  '.', '..', '=', '==', '= =', '-', '- -', '---', '.-', '.-.-', '..-', '~', '~~',
  '<->', '<=>', '<.>', '<~>', '<= >', '<- >', '<.->', '<..->', '<--',
);
my @lines = (     # used verbatim (missing-endpoint cases)
  '--> [ b ]', '[ a ] -->',
);
for my $c (@wrapped) {
  my $text = "[ a ] $c [ b ]\n";
  my $g = eval { Graph::Easy::Parser->new->from_text($text) };
  if ($@) { my ($m) = split /\n/, $@; print "ERR  [$c]  $m\n"; next; }
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$c]  as_txt: $txt\n";
}
for my $c (@lines) {
  my $g = eval { Graph::Easy::Parser->new->from_text("$c\n") };
  if ($@) { my ($m) = split /\n/, $@; print "ERR  [$c]  $m\n"; next; }
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$c]  as_txt: $txt\n";
}
PROBE

    'group-syntax' => <<'PROBE',
# serves: gently-6j0, gently-bzx (text_parser c6)
# claim probed: c6 — the group colon, single-group membership, nesting,
# and anonymous groups.
# Observed: the colon after the group name is part of the NAME — '( G: ... )'
# creates a group literally named 'G:'. A node belongs to exactly ONE
# group: re-referencing it inside a later group MOVES it (the earlier
# group empties). Nested groups: a node declared inside the inner group
# belongs only to the inner group; the outer group contains only its
# directly declared nodes (nesting is recorded as a 'group: A' attribute
# on the inner group). An anonymous group '( [ a ] )' is NAMED 'Group #0'
# by the oracle.
use Graph::Easy::Parser;
my @cases = (
  '( G: [ a ] --> [ b ] )',
  '( A: [ a ] )',
  "( A: [ a ] )\n( B: [ a ] )",
  '( A [ a ] ( B: [ b ] ) [ c ] )',
  '( [ a ] ) --> [ b ]',
);
for my $c (@cases) {
  my $g = eval { Graph::Easy::Parser->new->from_text($c) };
  if ($@) { my ($m) = split /\n/, $@; print "ERR  [$c]\n     $m\n"; next; }
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  my $gs = join ';', map { $_->name()."=[".join(',', sort map{$_->name()}$_->nodes())."]" } $g->groups();
  print "OK   [$c]\n     as_txt: $txt\n     groups: $gs\n";
}
PROBE

    'anon-reference' => <<'PROBE',
# serves: gently-6j0, gently-bzx (text_parser c1; supports gently-r22)
# claim probed: c1 — anonymous nodes and the 'cannot be referenced again'
# wording.
# Observed: anonymous nodes ARE named — '#1' (odd counter) — and CAN be
# referenced again by that generated name, written escaped as '\#1' (an
# unescaped '#' is a comment): the escaped reference REUSES the same node
# (3 nodes, 2 edges). node('#1') finds them; as_txt renders anonymous
# nodes back unnamed as '[ ]'. The bare token '[ #1 ]' is a parse error.
use Graph::Easy::Parser;
my $g = Graph::Easy::Parser->new->from_text("[ a ] --> [ ]\n[ \\#1 ] --> [ b ]\n");
print "nodes: ", join(', ', sort map { $_->name() } $g->nodes()), "\n";
my @e = $g->edges(); print "edges: ", scalar(@e), "\n";
print "node('#1'): ", (defined $g->node('#1') ? 'FOUND' : 'undef'), "\n";
print "as_txt:\n", $g->as_txt();
print "bare [ #1 ]: ";
my $g2 = eval { Graph::Easy::Parser->new->from_text("[ #1 ] --> [ b ]\n") };
print ($@ ? "ERROR: ".(split /\n/, $@)[0]."\n" : "parsed ok\n");
PROBE

    'layout-flow-direction' => <<'PROBE',
# serves: gently-89d (layout c3)
# claim probed: layout c3 — "source strictly earlier than target along
# the flow axis; only self-loops exempt".
# Observed: the cycle A->B->A under flow=east renders the return edge
# against the flow axis (arrow glyphs that are not '>'/'<>' eastward),
# so no reading of c3 is consistent with the oracle's own output.
use Graph::Easy;
my $g = Graph::Easy->new();
$g->set_attribute('flow', 'east');
$g->add_edge('A', 'B');
$g->add_edge('B', 'A');
my $a = $g->as_ascii();
print $a;
my ($arrowhead_glyphs) = ($a =~ /[^-]v|<|\^/);
print "non-east arrow present: ", (defined $arrowhead_glyphs ? 'YES' : 'no'), "\n";
PROBE

    'layout-edge-drop' => <<'PROBE',
# serves: gently-89d (layout c1/c4)
# claim probed: layout c4 — "every edge is routed along an orthogonal
# path" and c1 — "pure function" (oracle-verifiability scale).
# Observed (probe a): at 50 nodes/75 edges the pinned oracle's layouter
# SILENTLY DROPS edge paths — it warns "could only place 50 nodes/74
# edges out of 50/75 - giving up" (Layout.pm:846; the exact placed count
# varies with hash seed, the drop does not) but still renders, so the
# output cannot witness c4 at that scale. Probe b: the layouter arms a
# default 5-second alarm() (Layout.pm:481), so at dense/large scales it
# dies before producing output (size-envelope.observed: 200 nodes →
# "layout did not finish in time") — the oracle cannot verify ANY claim
# (including determinism, c1) at that scale. Both scales are outside
# the oracle-verifiable envelope; c4 is scoped within-envelope, c1's
# determinism is verified gently-only beyond it (ge.oracle c7 pattern).
use Graph::Easy;
my $g = Graph::Easy->new();
my @n = map { $g->add_node("n$_") } 0..49;
for my $i (0..48) { $g->add_edge($n[$i], $n[$i+1]); }
for my $i (0..25) { $g->add_edge($n[$i], $n[($i*7+13) % 50]); }
local $SIG{__WARN__} = sub { print "WARN: $_[0]" };
$g->layout();
print "edges declared: ", scalar($g->edges()), "\n";
my $with_path = 0;
for my $e ($g->edges()) { $with_path++ if @{ $e->{cells} || [] } || (defined $e->from()->{x} && $e->from() == $e->to()); }
print "edges with a routed cell path: $with_path\n";
my $big = Graph::Easy->new();
my @m = map { $big->add_node("m$_") } 0..199;
for my $i (0..198) { $big->add_edge($m[$i], $m[$i+1]); }
eval {
  local $SIG{ALRM} = sub { die "alarm\n" };
  $big->layout();
  print "200-node layout: completed\n";
};
print "200-node layout: ", ($@ eq "alarm\n" ? "ERROR: layout did not finish in time (default 5 s alarm, Layout.pm:481)" : ($@ ? "ERROR: $@" : "ok")), "\n";
PROBE

    'subgraph-handling' => <<'PROBE',
# serves: gently-13f (dot_parser c4)
# claim probed: dot_parser c4 — "only cluster_* subgraphs become groups";
# p4 — "anonymous subgraphs keep their nodes ungrouped".
# Observed: ANY named subgraph becomes a group under its name VERBATIM
# (cluster_x AND 'named'). The nameless 'subgraph { .. }' keyword form is a
# tokenizing ERROR ('not recognized by Graph::Easy::Parser::Graphviz'), but
# a bare '{ .. }' scope parses fine and keeps its nodes ungrouped — with one
# caveat: the node preceding the scope is still linked by the scope's inner
# edge chain (the left stack survives the scope boundary), so 'a -> b;
# { c -> d }' yields a SPURIOUS 'b --> d' edge. A named subgraph is also a
# valid edge endpoint: 'a -> foo' connects to the GROUP object. Nested
# subgraphs: each level is its own group; nodes belong to the innermost
# group only, and the inner group carries a 'group: outer' attribute. A
# subgraph with an attribute list before '{' is a tokenizing error.
use Graph::Easy::Parser::Graphviz;
my $p = Graph::Easy::Parser::Graphviz->new();
my @cases = (
  [ 'named subgraphs',        "digraph G { a -> b; subgraph cluster_x { c -> d } subgraph named { e -> f } }" ],
  [ 'nameless subgraph kw',   "digraph G { a -> b; subgraph { c -> d } }" ],
  [ 'bare {} scope',          "digraph G { a -> b; { c -> d } }" ],
  [ 'subgraph as edge target',"digraph G { subgraph foo { x } ; a -> foo }" ],
  [ 'nested subgraphs',       "digraph G { subgraph outer { subgraph inner { a } b } }" ],
  [ 'subgraph with attrs',    "digraph G { subgraph foo [color=red] { x } }" ],
);
for my $c (@cases) {
  my ($label, $text) = @$c;
  my $g = eval { $p->from_text($text) };
  if ($@) { my ($m) = split /\n/, $@; $m =~ s/ at \(eval \d+\) line \d+\.//; print "ERR  [$label]\n     $m\n"; next; }
  my $gs = join(';', map { $_->name()."=[".join(',', sort map {$_->name()} $_->nodes())."]" } sort { $a->name() cmp $b->name() } $g->groups());
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$label]\n     groups: $gs\n     as_txt: $txt\n";
}
PROBE

    'dot-bare-scope-in-named' => <<'PROBE',
# serves: gently-fb1 (dot_parser c4)
# claim probed: dot_parser c4 — a bare '{ .. }' scope keeps its nodes
# ungrouped at TOP level (pinned p4, both parsers), but a bare scope nested
# INSIDE a named subgraph inherits the enclosing named group: the oracle's
# group_stack survives bare-scope boundaries, so x lands IN group 'outer'
# (GROUP(outer: x)). gently keeps x ungrouped — this divergence is the
# remaining delta.
use Graph::Easy::Parser::Graphviz;
my $p = Graph::Easy::Parser::Graphviz->new();
my @cases = (
  [ 'bare scope, top level',        "digraph G { { x } }" ],
  [ 'bare scope in named subgraph', "digraph G { subgraph outer { { x } } }" ],
);
for my $c (@cases) {
  my ($label, $text) = @$c;
  my $g = eval { $p->from_text($text) };
  if ($@) { my ($m) = split /\n/, $@; $m =~ s/ at \(eval \d+\) line \d+\.//; print "ERR  [$label]\n     $m\n"; next; }
  my $gs = join(';', map { $_->name()."=[".join(',', sort map {$_->name()} $_->nodes())."]" } sort { $a->name() cmp $b->name() } $g->groups());
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$label]\n     groups: $gs\n     as_txt: $txt\n";
  print "     as_ascii:\n", $g->as_ascii(), "\n";
}
PROBE

    'dot-direction' => <<'PROBE',
# serves: gently-13f (dot_parser c1/c2)
# claim probed: dot_parser c1 — "a digraph header maps to a directed model
# graph and a graph header to an undirected one"; c2 — the edge rules.
# Observed: the EDGE OPERATOR decides each edge's direction, not the
# header: '->' yields a directed edge and '--' an undirected edge under
# EITHER header. The header only sets the model graph's 'type' attribute —
# a 'graph' header sets 'type: undirected', a 'digraph' header leaves the
# model at its directed default.
use Graph::Easy::Parser::Graphviz;
my $p = Graph::Easy::Parser::Graphviz->new();
my @cases = (
  [ 'digraph header + ->', "digraph G { a -> b }" ],
  [ 'digraph header + --', "digraph G { a -- b }" ],
  [ 'graph header + --',   "graph G { a -- b }" ],
  [ 'graph header + ->',   "graph G { a -> b }" ],
);
for my $c (@cases) {
  my ($label, $text) = @$c;
  my $g = eval { $p->from_text($text) };
  if ($@) { my ($m) = split /\n/, $@; $m =~ s/ at \(eval \d+\) line \d+\.//; print "ERR  [$label]\n     $m\n"; next; }
  my $dir = join ';', map { $_->{undirected} ? 'undirected' : 'directed' } $g->edges();
  my $type = $g->attribute('type') // 'unset';
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$label]\n     edges: $dir; graph type: $type\n     as_txt: $txt\n";
}
PROBE

    'dot-records-ports' => <<'PROBE',
# serves: gently-13f (dot_parser c5)
# claim probed: dot_parser c5 — "records, HTML labels, and ports are
# out-of-scope constructs producing typed errors".
# Observed: they are PARSED. A record label ('|' inside the label of a
# shape=record node) splits into numbered part nodes 'name.N' with the
# '<port>' markers stripped from the composite label; a port reference
# that resolves to a part ('a:f1') reattaches the edge to that part.
# An HTML-like table label parses into the same part scheme. A port
# reference with NO matching part fails with "Cannot find autosplit node
# for <base>:<port> on edge <id>". A record-style label WITHOUT
# shape=record stays a verbatim node attribute; shape=record without a
# '|' in the label does not split.
use Graph::Easy::Parser::Graphviz;
my $p = Graph::Easy::Parser::Graphviz->new();
my @cases = (
  [ 'record label attr',       qq{digraph G { a [label="A|B"] -> b }} ],
  [ 'record autosplit',        qq{digraph G { a [shape=record, label="A|B"] -> b }} ],
  [ 'record no pipe',          qq{digraph G { a [shape=record, label="AB"] }} ],
  [ 'record nested braces',    qq{digraph G { a [shape=record, label="A|{B|C}"] -> b }} ],
  [ 'record w/ ports',         qq{digraph G { a [shape=record, label="<f1>A|<f2>B"] ; a:f1 -> b ; c -> a:f2 }} ],
  [ 'port resolved w/ compass',qq{digraph G { a [shape=record, label="<f1>A|<f2>B"] ; a:f1:e -> b }} ],
  [ 'html label table',        qq{digraph G { a [label=<<TABLE><TR><TD>one</TD><TD>two</TD></TR></TABLE>>] ; a -> b }} ],
  [ 'html label malformed',    qq{digraph G { a [label=<<table>...</table>>] }} ],
  [ 'port w/o record part',    qq{digraph G { a -> b:p1 }} ],
  [ 'record port unknown',     qq{digraph G { a [shape=record, label="A|B"] ; a:zz -> b }} ],
  [ 'compass on plain node',   qq{digraph G { a -> b:nw }} ],
);
for my $c (@cases) {
  my ($label, $text) = @$c;
  my $g = eval { $p->from_text($text) };
  if ($@) { my ($m) = split /\n/, $@; $m =~ s/ at \(eval \d+\) line \d+\.//; print "ERR  [$label]\n     $m\n"; next; }
  my @n = sort map { $_->name() } $g->nodes();
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$label]\n     nodes: ", join(',', @n), "\n     as_txt: $txt\n";
}
PROBE

    'cli-flags' => <<'PROBE',
# serves: gently-0h9 (closed — kept as upstream evidence)
# claim probed: the cli flag roles. Upstream bin/graph-easy: --as selects
# the format (dot aliases graphviz), --output names the output FILE, the
# second positional is the output file, extras are ignored, unknown
# format exits 255.
my $bin = do { my $d = $INC{'Graph/Easy.pm'}; $d =~ s|lib/Graph/Easy\.pm$|bin/graph-easy|; $d };
my $input = '/tmp/probes-cli-flags.txt';
open my $fh, '>', $input or die $!; print $fh "[ a ] --> [ b ]\n"; close $fh;
my @cases = (
  ['default stdout',    qq{"$bin" "$input"}],
  ['--as dot aliases graphviz', qq{"$bin" --as dot "$input"}],
  ['--output FILE, stdout silent', qq{"$bin" --output /tmp/probes-cli-out "$input" && echo "out:\$(cat /tmp/probes-cli-out)"}],
  ['second positional is output', qq{"$bin" "$input" /tmp/probes-cli-out2 && echo "out:\$(cat /tmp/probes-cli-out2)"}],
  ['extra positionals ignored', qq{"$bin" "$input" /tmp/probes-cli-out3 extra junk && echo "out:\$(cat /tmp/probes-cli-out3)"}],
  ['unknown format exit 255', qq{"$bin" --as nosuch "$input"; echo "exit=\$?"}],
);
for my $case (@cases) {
  my ($label, $cmd) = @$case;
  my $out = `$cmd 2>/dev/null`;
  $out =~ s/\s+\z//;
  print "[$label]\n$out\n";
}
PROBE

    'node-unnamed' => <<'PROBE',
# serves: gently-r22 (graph_model c1)
# claim probed: graph_model c1 — "anonymous nodes are unnamed".
# Observed: upstream NAMES anonymous nodes with #<odd-counter> names
# (#1, #3 — the counter is shared with group numbering), node('#1')
# finds them, and as_txt renders them back as unnamed [ ].
use Graph::Easy::Parser;
my $p = Graph::Easy::Parser->new();
my $g = $p->from_text("[ a ] --> [ ]\n[ ] --> [ b ]\n");
print "node names: ", join(', ', sort map { $_->name() } $g->nodes()), "\n";
print "node('#1'): ", (defined $g->node('#1') ? 'FOUND' : 'undef'), "\n";
print "node('#0'): ", (defined $g->node('#0') ? 'FOUND' : 'undef'), "\n";
print "as_txt:\n", $g->as_txt();
PROBE

    'anon-numbering' => <<'PROBE',
# serves: gently-r22 (graph_model c1)
# claim probed: the EXACT anonymous-node naming scheme. Observed: the name
# is '#' . <object id> from a global object-id counter (Graph::Easy::Base
# _new_id) shared by every object the process creates — nodes, edges,
# groups — so an anon node's number depends on object-creation order within
# the parse, not on a per-graph anon-node count. The PARSER resets that
# counter at every parse (Parser::reset -> _reset_id), making parser-derived
# anon names deterministic; API-created anon nodes (add_anon_node without a
# parser) continue the counter from whatever the process created before.
use Graph::Easy;
use Graph::Easy::Parser;
my $g = Graph::Easy::Parser->new->from_text("[ a ] --> [ ] --> [ ]\n");
print "graph1 (node a + edge + 2 anon): ", join(', ', map { $_->name() } $g->nodes()), "\n";
my $g2 = Graph::Easy::Parser->new->from_text("( G: [ x ] )\n[ ] --> [ y ]\n");
print "graph2 nodes: ", join(', ', sort map { $_->name() } $g2->nodes()), "\n";
print "graph2 groups: ", join(', ', map { $_->name() } $g2->groups()), "\n";
my $g3 = Graph::Easy->new();
my $anon = $g3->add_anon_node();
print "graph3 API add_anon_node: ", $anon->name(), " is_anon: ", ($anon->is_anon() ? 'yes' : 'no'), "\n";
my $g4 = Graph::Easy::Parser->new->from_text("[ ]\n");
my ($n4) = $g4->nodes();
print "graph4 single anon: ", $n4->name(), "\n";
print "anon node has default label ' ' stored: ", (defined $n4->{att}->{label} ? "'" . $n4->{att}->{label} . "'" : 'undef'), "\n";
PROBE

    'attr-store-decompose' => <<'PROBE',
# serves: gently-r22 (graph_model c2)
# claim probed: c2 — "values stored verbatim; border components computed at
# assignment time". Observed: EVERY assignment value passes through the
# store-layer unquote (set_attribute -> unquote_attribute) — a quoted
# string is stored unquoted; the 'border' attribute is DECOMPOSED at
# assignment time into border-style/border-width/border-color (the 'border'
# key itself is never stored) and RECOMPOSED on read (attribute('border') is
# a virtual attribute via _border_attribute); class-scope assignments
# (node { color: ... }) are stored separately from object attributes and
# instance reads override them.
use Graph::Easy;
my $g = Graph::Easy->new();
my $n = $g->add_node('A');
$n->set_attribute('color', 'red');
print "color readback: ", $n->attribute('color'), "\n";
$n->set_attribute('border', 'dotted bold red');
print "border readback (virtual): ", $n->attribute('border'), "\n";
print "border-style: ", ($n->attribute('border-style') // 'undef'), "\n";
print "border-width: ", ($n->attribute('border-width') // 'undef'), "\n";
print "border-color: ", ($n->attribute('border-color') // 'undef'), "\n";
$n->set_attribute('label', '"hello world"');
print "label readback (quoted at set): ", $n->attribute('label'), "\n";
$g->set_attribute('node', 'color', 'blue');
my $b = $g->add_node('B');
print "node A color (instance): ", $n->attribute('color'), "\n";
print "node B color (class): ", $b->attribute('color'), "\n";
my $e = $g->add_edge('A', 'B');
$e->set_attribute('label', 'go');
print "edge label readback: ", $e->attribute('label'), "\n";
my $txt = eval { $g->as_txt() };
print $@ ? "as_txt ERROR: $@\n" : "as_txt:\n$txt";
PROBE

    'group-edge' => <<'PROBE',
# serves: gently-r22 (graph_model c3)
# claim probed: c3 — "every edge references exactly two NODES". Observed:
# edges can carry a GROUP as an endpoint. The parser's _link_lists adds
# edges from every left-stack node to the GROUP OBJECT itself (not rewritten
# to group members); the API add_edge($group1,$group2) does the same and
# indexes the group endpoint in the groups hash.
use Graph::Easy;
use Graph::Easy::Parser;
sub dump_edges {
  my ($g) = @_;
  my @e = $g->edges();
  print "edges: ", scalar(@e), "\n";
  for my $e (@e) {
    my ($f, $t) = ($e->{from}, $e->{to});
    my $fname = eval { $f->name() }; $fname = 'undef' if !defined $fname;
    my $tname = eval { $t->name() }; $tname = 'undef' if !defined $tname;
    print "  edge ", ref($f), "($fname) -> ", ref($t), "($tname)\n";
  }
}
my $g = eval { Graph::Easy::Parser->new->from_text("( A: [ x ] ) --> ( B: [ y ] )\n") };
if ($@) { print "parse group-to-group: ERROR: ", (split /\n/, $@)[0], "\n"; }
else {
  print "group-to-group (with inner nodes) parsed\n";
  print "nodes: ", join(', ', sort map { $_->name() } $g->nodes()), "\n";
  print "groups: ", join(', ', map { $_->name() } $g->groups()), "\n";
  dump_edges($g);
  my $txt = eval { $g->as_txt() };
  if ($@) { print "as_txt ERROR: ", (split /\n/, $@)[0], "\n"; }
  else { print "as_txt:\n$txt"; }
}
my $g2 = eval { Graph::Easy::Parser->new->from_text("( A ) --> ( B )\n") };
if ($@) { print "parse bare group-to-group: ERROR: ", (split /\n/, $@)[0], "\n"; }
else {
  print "bare group-to-group ( A ) --> ( B ) parsed\n";
  print "nodes: ", join(', ', sort map { $_->name() } $g2->nodes()), "\n";
  print "groups: ", join(', ', map { $_->name() } $g2->groups()), "\n";
  dump_edges($g2);
}
my $g3 = Graph::Easy->new();
my $ga = $g3->add_group('G1');
my $gb = $g3->add_group('G2');
my ($x, $y, $ed) = $g3->add_edge($ga, $gb);
print "API add_edge(G1,G2): from ", ref($x), " to ", ref($y), "\n";
dump_edges($g3);
PROBE

    'group-edge-graphviz' => <<'PROBE',
# serves: gently-0kg (graphviz_render c2/c4, txt_render c3)
# claim probed: graphviz_render c2 — "the arrow matches the model edge
# direction" and c4 — "the emitted DOT re-parses isomorphically", for the
# group-endpoint case, plus txt_render c3. Observed: a member-bearing
# group-to-group edge does NOT crash as_graphviz — it emits the two member
# nodes with ltail/lhead cluster references and NO ARROW at all
# (`x  y [ ..., ltail="cluster2", lhead="cluster3" ]`) — malformed DOT that
# no edge operator parses. A BARE group-to-group edge (groups without
# members) DOES crash: "Can't call method \"_graphviz_point\" on an
# undefined value at .../As_graphviz.pm line 547". as_txt serializes the
# member-bearing group edge as a plain NODE edge (`[ x ] --> [ y ]`),
# dropping the group-endpoint-ness on round-trip.
use Graph::Easy;
my $g = Graph::Easy->new();
my $x = $g->add_node('x'); my $y = $g->add_node('y');
my $A = $g->add_group('A'); $A->add_node($x);
my $B = $g->add_group('B'); $B->add_node($y);
$g->add_edge($A, $B);
my $dot = eval { $g->as_graphviz() };
if ($@) { print "member-bearing group edge as_graphviz: CRASH: ", (split /\n/, $@)[0], "\n"; }
else {
  my @stmt = grep { /x/ && /y/ } split /\n/, $dot;
  print "member-bearing group edge as_graphviz: OK, edge statement:\n  @stmt\n";
  print "  statement contains an arrow operator: ", (grep { /[->-]{2}/ } @stmt) ? "yes" : "NO (arrowless)", "\n";
}
my $txt = eval { $g->as_txt() };
print "member-bearing group edge as_txt:\n$txt" if !($@) && defined $txt;
my $bare = Graph::Easy->new();
my $G1 = $bare->add_group('G1'); my $G2 = $bare->add_group('G2');
$bare->add_edge($G1, $G2);
my $bdot = eval { $bare->as_graphviz() };
if ($@) { print "bare group-to-group as_graphviz: CRASH: ", (split /\n/, $@)[0], "\n"; }
else { print "bare group-to-group as_graphviz: OK\n$bdot\n"; }
PROBE

    'deleted-node-add-edge' => <<'PROBE',
# serves: gently-r22 (graph_model c3/p3)
# claim probed: p3 — "no dangling edge ever escapes a mutation" vs
# add_edge on a DELETED node. Observed: del_node drops the node AND all its
# incident edges; a later add_edge naming the deleted node SUCCEEDS by
# implicitly re-creating it as a fresh bare node (stored attributes of the
# deleted node are gone; the old incident edges are NOT revived).
use Graph::Easy;
my $g = Graph::Easy->new();
$g->add_edge('A', 'B');
$g->add_edge('A', 'C');
my $a = $g->node('A');
$a->set_attribute('color', 'red');
print "before delete: nodes: ", join(', ', sort map { $_->name() } $g->nodes()), "; edges: ", scalar($g->edges()), "\n";
$g->del_node('A');
print "after del_node('A'): nodes: ", join(', ', sort map { $_->name() } $g->nodes()), "; edges: ", scalar($g->edges()), "\n";
my ($x, $y, $e) = $g->add_edge('A', 'B');
print "add_edge('A','B') after delete: ", (defined $e ? 'succeeded' : 'FAILED'), "\n";
print "nodes now: ", join(', ', sort map { $_->name() } $g->nodes()), "\n";
print "recreated 'A' color: '", ($g->node('A')->attribute('color') // 'undef'), "'\n";
print "edges now: ", scalar($g->edges()), "\n";
my $txt = eval { $g->as_txt() };
if ($@) { print "as_txt ERROR: ", (split /\n/, $@)[0], "\n"; }
else { print "as_txt:\n$txt"; }
PROBE

    'href-escaping' => <<'PROBE',
# serves: gently-dcp, gently-eyo (html_render c2)
# claim probed: html_render c2 — "link URLs escaped".
# Observed: the href carries the raw ampersand; upstream does NOT escape
# it (&amp; would be required).
use Graph::Easy;
my $g = Graph::Easy->new();
my $n = $g->add_node('A');
$n->set_attribute('link', 'http://x/?a=1&b=2');
my $h = $g->as_html();
print $h;
print "raw & in href: ", ($h =~ /href=[^>]*\?a=1&b=2/ ? 'YES (unescaped)' : 'no'), "\n";
PROBE

    'td-colspan' => <<'PROBE',
# serves: gently-dcp, gently-eyo (html_render c4)
# claim probed: html_render c4 — "td count equals cell count".
# Observed: a two-line label node emits ONE td carrying colspan=4
# rowspan=4 plus three empty tr rows — td count differs from cell count
# because of colspan/rowspan.
use Graph::Easy;
my $g = Graph::Easy->new();
my $n = $g->add_node('A');
$n->set_attribute('label', "A\nB");
# The oracle HANGS on this input class under as_html/as_ascii — an
# infinite loop the alarm() idiom cannot bound (upstream's internal eval
# blocks swallow the die). The SIGKILL-bounded child reproduces it:
my ($status, $out, $outerr) = run_perl_snippet(<<'CHILD_HTML', 10);
use Graph::Easy;
my $g = Graph::Easy->new();
my $n = $g->add_node('A');
$n->set_attribute('label', "A\\nB");
my $h = $g->as_html();
print $h;
my $tds = () = $h =~ /<td/g;
my $trs = () = $h =~ /<tr/g;
print "td count: $tds, tr count: $trs\n";
CHILD_HTML
if ($status ne 'ok') {
  print "oracle HANGS on a multiline-label node under as_html (SIGKILL-bounded at 10 s)\n";
  print "earlier observation when the layouter completed: ONE td with\n";
  print "colspan=4 rowspan=4 carrying 'A<br>B' plus three empty tr rows —\n";
  print "td count differs from cell count because of colspan/rowspan\n";
}
else { print $out; }
print "as_txt of the same node (does not hang):\n", $g->as_txt();
PROBE

    'html-edge-styles' => <<'PROBE',
# serves: gently-eyo (html_render c3)
# claim probed: c3 — edge cells in as_html: the border remap per edge
# style, the orientation classes (lh horizontal / lv vertical / eb
# padding-corner), the arrow spans per direction, and the label cell.
use Graph::Easy;
use Graph::Easy::Parser;
sub edge_tds {
  my ($g) = @_;
  my $h = $g->as_html();
  my @t = $h =~ /<td[^>]*class="edge[^"]*"[^>]*>.*?<\/td>/g;
  for (@t) { s/\s+/ /g; s/^ //; }
  @t;
}
for my $s (qw(solid dotted dashed double wave bold wide broad dot-dash dot-dot-dash double-dash bold-dash)) {
  my $g = Graph::Easy->new(); $g->add_edge('a','b');
  my @e = $g->edges(); $e[0]->set_attribute('style', $s);
  print "== style $s ==\n  ", join(" ; ", edge_tds($g)), "\n";
}
my $gn = Graph::Easy->new(); $gn->add_edge('a','b');
my @e = $gn->edges(); eval { $e[0]->set_attribute('style', 'none') };
print "style none: ", ($@ ? 'REJECTED' : 'accepted'), "\n";
for my $flow (qw(east south west north)) {
  my $g = Graph::Easy->new(); $g->add_edge('a','b');
  $g->set_attribute('flow', $flow);
  print "== flow $flow (arrow direction) ==\n  ", join(" ; ", edge_tds($g)), "\n";
}
# arrowless: parsed from the arrowless operator (no arrowhead at all)
my $g3 = Graph::Easy::Parser->new->from_text("[ a ] -- { style: dashed; } [ b ]\n");
print "== arrowless dashed ==\n  ", join(" ; ", edge_tds($g3)), "\n";
my $g4 = Graph::Easy->new(); $g4->add_edge('a','b');
my @e4 = $g4->edges(); $e4[0]->set_attribute('label', 'go');
print "== labelled edge ==\n  ", join(" ; ", edge_tds($g4)), "\n";
PROBE

    'html-css-rules' => <<'PROBE',
# serves: gently-eyo (html_render c4)
# claim probed: c4 — the CSS rules as_html documents embed (upstream
# exposes them as ->css(); as_html itself emits only the table).
use Graph::Easy;
my $g = Graph::Easy->new(); $g->add_edge('a','b');
print "== plain graph css ==\n", $g->css(), "\n";
my $g2 = Graph::Easy->new();
my $n = $g2->add_node('A'); $n->set_attribute('shape','rounded');
print "== rounded-shape css ==\n", $g2->css(), "\n";
my $g3 = Graph::Easy->new(); $g3->add_edge('a','b'); $g3->add_group('grp');
print "== group css ==\n", $g3->css(), "\n";
PROBE

    'html-colors-shapes' => <<'PROBE',
# serves: gently-eyo (html_render c5)
# claim probed: c5 — the W3C color-name scheme and the shape classes as
# observed in the node/edge td bytes (the node td is the first
# <td colspan=4 rowspan=4 ...> of the as_html output).
use Graph::Easy;
sub node_td {
  my ($g) = @_;
  my $h = $g->as_html();
  my ($t) = $h =~ /<td colspan=4 rowspan=4.*?<\/td>/gs;
  return ($t // '(none)') =~ s/\s+/ /gr;
}
for my $c (qw(red blue white black lime fuchsia aqua silver gray olive purple teal navy)) {
  my $g = Graph::Easy->new(); my $n = $g->add_node('A');
  $n->set_attribute('fill', $c);
  print "fill=$c: ", node_td($g), "\n";
}
my $g = Graph::Easy->new(); my $n = $g->add_node('A');
$n->set_attribute('color', 'blue');
print "color=blue: ", node_td($g), "\n";
my $g2 = Graph::Easy->new(); my $n2 = $g2->add_node('A');
$n2->set_attribute('background', '#00ff00');
print "background=#00ff00 (plain node): ", node_td($g2), "\n";
my $g3 = Graph::Easy->new(); my $n3 = $g3->add_node('A');
$n3->set_attribute('fill', 'rgb(10,20,30)');
print "fill=rgb(10,20,30): ", node_td($g3), "\n";
my $g4 = Graph::Easy->new(); my $n4 = $g4->add_node('A');
eval { $n4->set_attribute('fill', 'notacolor') };
print "fill=notacolor: ", ($@ ? 'REJECTED' : node_td($g4)), "\n";
for my $s (qw(rounded circle ellipse point invisible diamond)) {
  my $g = Graph::Easy->new(); my $n = $g->add_node('A');
  eval { $n->set_attribute('shape', $s) };
  print "shape=$s: ", ($@ ? 'REJECTED' : node_td($g)), "\n";
}
my $g5 = Graph::Easy->new(); $g5->add_edge('a','b');
my @e = $g5->edges(); $e[0]->set_attribute('color', 'red');
my @t = $g5->as_html() =~ /<td[^>]*class="edge[^"]*"[^>]*>.*?<\/td>/g;
print "edge color=red: ", join(" ; ", map { s/\s+/ /gr } @t), "\n";
PROBE

    'shape-outline-collapse' => <<'PROBE',
# serves: gently-dcp, gently-css (ascii_render shapes; html border-styles)
# claim probed: the ascii_render "14 shapes, bold/wide/broad distinct"
# rows.
# Observed: several of the 14 documented shape names are rejected by the
# oracle outright, and in as_html the shape does not change the node
# class at all (class='graph'/'node'); bold/wide/broad differ only in
# border-width units (4px / 1em / 0.5em).
use Graph::Easy;
my @shapes = qw(rect rounded ellipse circle diamond hexagon house parallelogram parallelogram_alt trapezoid trapezoid_alt triangle);
for my $s (@shapes) {
  my $g = Graph::Easy->new();
  my $n = $g->add_node('A');
  eval { $n->set_attribute('shape', $s) };
  if ($@) { print "shape=$s REJECTED\n"; next; }
  my $h = $g->as_html();
  my ($cls) = $h =~ /class=.([^'"]+)/;
  print "shape=$s html-class=$cls\n";
}
for my $bs (qw(bold wide broad)) {
  my $g = Graph::Easy->new();
  $g->add_node('A')->set_attribute('border-style', $bs);
  my $h = $g->as_html();
  my ($style) = $h =~ /style=.([^'"]+)/;
  print "border-style=$bs style=$style\n";
}
PROBE

    'graphviz-round-trip' => <<'PROBE',
# serves: gently-0kg, gently-b4v (graphviz_render c2/c4, txt_render c3)
# claim probed: round-trip fidelity the oracle itself lacks.
# Observed: as_graphviz ALWAYS emits digraph with -> (an undirected
# edge comes back directed); a named group round-trips as
# subgraph "cluster0" (group name lost); as_graphviz output embeds a
# wall-clock timestamp ("Generated by ... at <date>"), so it is not
# byte-reproducible across runs.
use Graph::Easy;
my $g = Graph::Easy->new();
$g->add_edge('a', 'b');
print "undirected edge as_graphviz:\n", $g->as_graphviz();
my $g2 = Graph::Easy->new();
$g2->add_group('grp');
$g2->add_edge('a', 'b');
print "named group as_graphviz:\n", $g2->as_graphviz();
PROBE

    'size-envelope' => <<'PROBE',
# serves: gently-k4u (perf c2/c5)
# claim probed: perf c2 — "1000-node within 2 s" gated on oracle
# byte-equality.
# Observed: the oracle renders a 200-node chain in seconds with
# superlinear growth (and carries a default 5 s alarm() that kills long
# renders) — a 1000-node fixture is unrecordable in practice, so large
# budgets can only be verified against gently alone (tier 3).
use Graph::Easy;
use Time::HiRes qw(time);
for my $n (50, 100, 200) {
  my $g = Graph::Easy->new();
  $g->add_node("n$_") for 0 .. $n - 1;
  # chain + skip edges — realistic density; a bare chain layouts trivially
  for my $i (0 .. $n - 2) { $g->add_edge("n$i", 'n' . ($i + 1)); }
  for my $i (0 .. $n - 5) { $g->add_edge("n$i", 'n' . ($i + 3)); $g->add_edge("n$i", 'n' . ($i + 4)); }
  my $t0 = time();
  my $art = eval { $g->as_ascii() };
  my $t1 = time();
  if ($@) { printf "nodes=%d: ERROR: %s", $n, $@; }
  else { printf "nodes=%d edges=%d: %.3f s (%d bytes)\n", scalar($g->nodes()), scalar($g->edges()), $t1 - $t0, length $art; }
}
PROBE

    'ascii-render-tables' => <<'PROBE',
# serves: gently-0cq (ascii_render c1/c2/c5/c6)
# claim probed: the ASCII renderer's tables — node border styles, node
# shapes, edge styles, and label wrapping/display-width, observed as
# bytes. Multiline labels are alarm-bounded: upstream can hang on them.
use Graph::Easy;
use Graph::Easy::Parser;
my $bound = 5;
sub render {
  my ($text) = @_;
  my $out;
  eval {
    local $SIG{ALRM} = sub { die "alarm\n" };
    alarm $bound;
    my $g = Graph::Easy::Parser->new->from_text($text);
    $out = $g->as_ascii();
    alarm 0;
  };
  alarm 0;
  return $@ ? "HANG-OR-ERROR: " . (split /\n/, $@)[0] : $out;
}
for my $style (qw(solid dotted dashed double wave bold wide broad dot-dash dot-dot-dash double-dash none)) {
  print "== border $style ==\n", render("[ x ] { border: $style; }\n"), "\n";
}
for my $style (qw(solid double dotted wave dashed bold)) {
  print "== edge $style ==\n", render("[ a ] --> { style: $style; } [ b ]\n"), "\n";
  print "== edge $style arrowless ==\n", render("[ a ] -- { style: $style; } [ b ]\n"), "\n";
}
for my $shape (qw(box rounded point circle ellipse diamond triangle pentagon hexagon octagon parallelogram house invisible img)) {
  print "== shape $shape ==\n", render("[ x ] { shape: $shape; }\n"), "\n";
}
print "== label long ==\n", render("[ this is a very long node label that must wrap somewhere ]\n"), "\n";
print "== label wide glyph ==\n", render("[ \x{e4}\x{b8}\x{ad} ]\n"), "\n";
PROBE

    'attr-quote-value' => <<'PROBE',
# serves: gently-8ol (text_parser c5 quote-unquoting)
# claim probed: the attribute-value quote-stripping layer. Upstream strips
# quotes NOT in the parser (_unquote leaves them in) but at the STORE
# layer (Graph::Easy::Attributes::unquote_attribute, run from
# set_attribute): s/^["'](.*)["']\z/$1/ — either quote char, GREEDY
# (first char + last char, even mismatched), then \([#"';\\]) unescape,
# then %XX entity decode. The probe prints the STORED value
# (->attribute) plus as_txt for each case.
use Graph::Easy::Parser;
my @cases = (
  '[ a ] { label: "hello world"; } --> [ b ]',   # double-quoted
  "[ a ] { label: 'hello'; } --> [ b ]",          # single-quoted
  '[ a ] { label: "a" "b"; } --> [ b ]',          # greedy strip: first+last
  "[ a ] { label: \"mixed'; } --> [ b ]",        # mismatched quote chars
  '[ a ] { label: x"y"z; } --> [ b ]',            # mid-value quotes
  '[ a ] { label: "a\\"b"; } --> [ b ]',          # escaped quote inside
  "[ a ] { label: a\\'b; } --> [ b ]",            # escaped single quote
  '[ a ] { label: a\\;b; } --> [ b ]',            # escaped semicolon
  '[ a ] { label: a\\\\b; } --> [ b ]',           # escaped backslash
  '[ a ] { label: "abc; } --> [ b ]',             # unterminated quote
  '[ a ] { label: ""; } --> [ b ]',               # empty quoted value
  '[ a ] { label: %41; } --> [ b ]',              # %XX entity decode
);
for my $c (@cases) {
  my $g = eval { Graph::Easy::Parser->new->from_text("$c\n") };
  if ($@) { my ($m) = split /\n/, $@; print "ERR  [$c]\n     $m\n"; next; }
  my $n = $g->node('a');
  my $stored = defined $n ? $n->attribute('label') : '(no node a)';
  my $txt = $g->as_txt(); $txt =~ s/\s+\z//; $txt =~ s/\n/ | /g;
  print "OK   [$c]\n     stored: [$stored]\n     as_txt: $txt\n";
}
PROBE

    'boxart-render-tables' => <<'PROBE',
# serves: gently-css (boxart_render c1/c2/c3/c4)
# claim probed: the boxart renderer's tables (as_boxart — the same render
# path as as_ascii with _ascii_style=1) — node border styles, edge styles
# (arrowed/arrowless), junction/neighbourhood combinations (edge styles
# meeting node borders, mixed-style edge chains, bends), shapes, and wide
# labels (repeat units), observed as bytes.
use Graph::Easy;
use Graph::Easy::Parser;
binmode(STDOUT, ':utf8');
my $bound = 5;
sub render {
  my ($text) = @_;
  my $out;
  eval {
    local $SIG{ALRM} = sub { die "alarm\n" };
    alarm $bound;
    my $g = Graph::Easy::Parser->new->from_text($text);
    $out = $g->as_boxart();
    alarm 0;
  };
  alarm 0;
  return $@ ? "HANG-OR-ERROR: " . (split /\n/, $@)[0] : $out;
}
for my $style (qw(solid dotted dashed double wave bold wide broad dot-dash dot-dot-dash double-dash bold-dash none)) {
  print "== border $style ==\n", render("[ x ] { border: $style; }\n"), "\n";
  print "== border $style wide label ==\n", render("[ xxxxxxxxxx ] { border: $style; }\n"), "\n";
}
for my $style (qw(solid double dotted wave dashed bold dot-dash dot-dot-dash double-dash bold-dash)) {
  print "== edge $style ==\n", render("[ a ] --> { style: $style; } [ b ]\n"), "\n";
  print "== edge $style arrowless ==\n", render("[ a ] -- { style: $style; } [ b ]\n"), "\n";
}
print "== junction: dashed border + solid edge ==\n", render("[ a ] { border: dashed; } --> [ b ]\n"), "\n";
print "== junction: bold border + solid edge ==\n", render("[ a ] { border: bold; } --> [ b ]\n"), "\n";
print "== junction: solid border + dashed edge ==\n", render("[ a ] --> { style: dashed; } [ b ]\n"), "\n";
print "== junction: solid border + double edge ==\n", render("[ a ] --> { style: double; } [ b ]\n"), "\n";
print "== junction: mixed chain dashed->a->double->b->dotted->c ==\n", render("[ x ] --> { style: dashed; } [ a ] --> { style: double; } [ b ] --> { style: dotted; } [ c ]\n"), "\n";
print "== junction: bend (cycle) solid ==\n", render("[ a ] --> [ b ] --> [ a ]\n"), "\n";
print "== junction: bend (cycle) mixed dashed/double ==\n", render("[ a ] --> { style: dashed; } [ b ] --> { style: double; } [ a ]\n"), "\n";
print "== junction: crossing edges ==\n", render("[ a ] --> [ c ]\n[ b ] --> [ d ]\n"), "\n";
for my $shape (qw(box rounded point circle ellipse diamond triangle pentagon hexagon octagon parallelogram house invisible img)) {
  print "== shape $shape ==\n", render("[ x ] { shape: $shape; }\n"), "\n";
}
print "== shape rounded + border double ==\n", render("[ x ] { shape: rounded; border: double; }\n"), "\n";
print "== shape rounded + border dotted ==\n", render("[ x ] { shape: rounded; border: dotted; }\n"), "\n";
print "== shape rounded + border wave ==\n", render("[ x ] { shape: rounded; border: wave; }\n"), "\n";
print "== shape rounded + border bold ==\n", render("[ x ] { shape: rounded; border: bold; }\n"), "\n";
print "== shape rounded + border dot-dot-dash ==\n", render("[ x ] { shape: rounded; border: dot-dot-dash; }\n"), "\n";
print "== edge wide ==\n", render("[ a ] --> { style: wide; } [ b ]\n"), "\n";
print "== edge broad ==\n", render("[ a ] --> { style: broad; } [ b ]\n"), "\n";
print "== edge wide arrowless ==\n", render("[ a ] -- { style: wide; } [ b ]\n"), "\n";
print "== bend (cycle) bold ==\n", render("[ a ] --> { style: bold; } [ b ] --> { style: bold; } [ a ]\n"), "\n";
print "== bend (cycle) dashed ==\n", render("[ a ] --> { style: dashed; } [ b ] --> { style: dashed; } [ a ]\n"), "\n";
print "== bend (cycle) dot-dash ==\n", render("[ a ] --> { style: dot-dash; } [ b ] --> { style: dot-dash; } [ a ]\n"), "\n";
print "== bend (cycle) double-dash ==\n", render("[ a ] --> { style: double-dash; } [ b ] --> { style: double-dash; } [ a ]\n"), "\n";
print "== bend (cycle) dot-dot-dash ==\n", render("[ a ] --> { style: dot-dot-dash; } [ b ] --> { style: dot-dot-dash; } [ a ]\n"), "\n";
print "== westward solid ==\n", render("[ b ] <-- [ a ]\n"), "\n";
print "== westward solid arrowless ==\n", render("[ b ] -- [ a ]\n"), "\n";
print "== westward double ==\n", render("[ b ] <= { style: double; } [ a ]\n"), "\n";
print "== westward dot-dash ==\n", render("[ b ] <-- { style: dot-dash; } [ a ]\n"), "\n";
print "== westward dot-dot-dash ==\n", render("[ b ] <-- { style: dot-dot-dash; } [ a ]\n"), "\n";
print "== westward labelled ==\n", render("[ b ] <-- { label: go; } [ a ]\n"), "\n";
print "== bidi solid ==\n", render("[ a ] <-> [ b ]\n"), "\n";
print "== bidi double ==\n", render("[ a ] <=> { style: double; } [ b ]\n"), "\n";
print "== junction: none border + edge ==\n", render("[ a ] { border: none; } --> [ b ]\n"), "\n";
print "== junction: none border + bend ==\n", render("[ a ] { border: none; } --> [ b ] --> [ a ]\n"), "\n";
print "== selfloop solid ==\n", render("[ a ] --> [ a ]\n"), "\n";
print "== selfloop long label ==\n", render("[ abcdefgh ] --> [ abcdefgh ]\n"), "\n";
print "== selfloop arrowless ==\n", render("[ a ] -- [ a ]\n"), "\n";
print "== selfloop double ==\n", render("[ a ] --> { style: double; } [ a ]\n"), "\n";
print "== selfloop dotted ==\n", render("[ a ] { border: dotted; } --> { style: dotted; } [ a ]\n"), "\n";
print "== labelled edge solid ==\n", render("[ a ] --> { label: go; } [ b ]\n"), "\n";
print "== labelled edge dotted ==\n", render("[ a ] --> { style: dotted; label: go; } [ b ]\n"), "\n";
print "== labelled edge dot-dot-dash ==\n", render("[ a ] --> { style: dot-dot-dash; label: go; } [ b ]\n"), "\n";
print "== arrowless bend (cycle) ==\n", render("[ a ] -- [ b ] -- [ a ]\n"), "\n";
print "== bend arrows all four directions ==\n", render("[ a ] --> [ b ] --> [ c ] --> [ d ]\n"), "\n";
PROBE

    'perl5lib-pin' => 'PROBE_PERL5LIB_PIN',
);

sub probe_perl5lib_pin {
    my $inc_path = $INC{'Graph/Easy.pm'} // '';
    return <<"EOF";
# claim probed: the pin chain — version+commit only, no Perl version,
# no PERL_HASH_SEED, no PERL5LIB/source contract.
perl: $^V
Graph::Easy version: $version (pin: v$PIN_VERSION)
loaded from: $inc_path
PERL_HASH_SEED: ${\($ENV{PERL_HASH_SEED} // '(unset — hash randomization is LIVE)')}
commit verification: NOT POSSIBLE from an installed dist — the pin names
  commit $PIN_COMMIT, but neither cpanm nor a backpan tarball exposes the
  git commit; the tarball may or may not be byte-identical to that
  commit. This is the open pin-contract question gently-ikm redesigns
  (source checkout vs cpanm, commit verification mechanism).
EOF
}

# Renders one POD-mined example (gently-ni9.1) under two hash seeds via
# run_perl_snippet and prints both renders — making seed divergence visible
# in the recorded claim without ever pretending the bytes are stable.
sub pod_probe_body {
    my ($name, $example) = @_;
    my $child = <<'CHILD';
use strict; use warnings;
use Graph::Easy::Parser;
my ($file) = @ARGV;
local $/; open my $fh, '<:encoding(UTF-8)', $file or die "open: $!";
my $text = <$fh>;
my $g = Graph::Easy::Parser->new->from_text($text);
die "parse failed\n" unless $g;
print "as_txt:\n", $g->as_txt();
print "as_ascii:\n", $g->as_ascii();
CHILD
    my $tmp = File::Temp->newdir('pod-probe-XXXXXX', TMPDIR => 1, CLEANUP => 1);
    my $file = "$tmp/example.txt";
    open my $fh, '>', $file or die "probes: cannot write example: $!\n";
    print $fh $example;
    close $fh;
    for my $seed (0, 1) {
        print "== seed $seed ==\n";
        local $ENV{PERL_HASH_SEED} = $seed;
        my ($status, $out, $err) = run_perl_snippet($child, 30, $file);
        chomp(my $msg = $err // '');
        die "probes: pod claim '$name' seed $seed: $status $msg" if $status ne 'ok';
        print $out;
    }
}

if ($mode eq 'claims') {
    print "probes: claim probes (run: perl tests/repro/probes.pl claim <name>)\n";
    for my $name (sort keys %CLAIMS) {
        print "  $name\n    serves: $SERVES{$name}\n";
    }
    exit 0;
}

# claims dir for recorded observations
use POSIX qw(WNOHANG);
use File::Path qw(make_path);
use File::Temp;
use File::Basename qw(dirname);
my $CLAIMS_DIR = dirname(__FILE__) . '/claims';

# Run a perl snippet as a child process with a hard SIGKILL bound. The
# alarm() idiom does NOT bound the oracle's infinite loops — upstream's
# internal eval blocks swallow the die before our handler sees it — so the
# only reliable bound is killing the child from the parent.
sub run_perl_snippet {
    my ($src, $timeout, @args) = @_;
    my $tmp = File::Temp->newdir('probe-snippet-XXXXXX', TMPDIR => 1, CLEANUP => 1);
    my $file = "$tmp/snippet.pl";
    open my $fh, '>', $file or die "probes: cannot write snippet: $!\n";
    print $fh $src;
    close $fh;
    my $errfile = "$tmp/err";
    my $outfile = "$tmp/out";
    my $pid = fork();
    die "probes: fork: $!\n" unless defined $pid;
    if ($pid == 0) {
        open(STDOUT, '>', $outfile) or exit 126;
        open(STDERR, '>', $errfile) or exit 126;
        exec $^X, $file, @args or exit 127;
    }
    my $deadline = time() + $timeout;
    while (1) {
        my $r = waitpid($pid, WNOHANG);
        if ($r == 0 && time() > $deadline) {
            kill 'KILL', $pid;
            waitpid($pid, 0);
            return ('timeout', '', "child exceeded the ${timeout}s probe bound (SIGKILL)");
        }
        if ($r == $pid) {
            my $slurp = sub { local $/; open my $fh, '<', $_[0]; <$fh> // '' };
            return ($? == 0 ? 'ok' : 'error', $slurp->($outfile) // '', $slurp->($errfile) // '');
        }
        select(undef, undef, undef, 0.05);
    }
}

if ($mode eq 'claim') {
    my $name = shift @ARGV;
    my $src = $CLAIMS{$name} // die "probes: unknown claim probe '$name'\n$usage";
    make_path($CLAIMS_DIR);
    my $body = $src;
    if (ref $src ne 'CODE') {
        # strip the leading comment block, run the rest in one process
        my @lines = split /\n/, $src;
        shift @lines while @lines && $lines[0] =~ /^#/;
        $body = join "\n", @lines;
    }
    my $observed;
    if ($src eq 'PROBE_PERL5LIB_PIN') {
        $observed = probe_perl5lib_pin();
    }
    else {
        # capture the probe body's stdout so it can be both shown and
        # recorded. The :utf8 layer is REQUIRED for the boxart probes —
        # their output is Unicode box-drawing — and a no-op for the
        # pure-ASCII probes.
        my $buf = '';
        open my $cap, '>>', \$buf or die "probes: cannot capture output: $!\n";
        binmode $cap, ':encoding(UTF-8)';
        my $old = select($cap);
        eval $body;
        my $err = $@;
        select($old);
        close $cap;
        die "probes: claim '$name' died: $err" if $err;
        $observed = $buf;
    }
    print $observed;
    my $outfile = "$CLAIMS_DIR/$name.observed";
    open my $fh, '>', $outfile or die "probes: cannot write '$outfile': $!\n";
    print $fh "# observed: $PIN — probe: $name\n", $observed;
    close $fh;
    print "\nrecorded: $outfile\n";
    exit 0;
}

sub uniq { my %seen; grep { !$seen{$_}++ } @_; }

if ($mode eq 'admit') {
    my $dir  = shift @ARGV // die "probes: admit needs a fixtures dir\n$usage";
    my $out  = shift @ARGV // dirname(__FILE__) . '/admission.tsv';

    opendir(my $dh, $dir) or die "probes: cannot open fixture dir '$dir': $!\n";
    my @inputs = sort grep { /\.txt$/ && -f "$dir/$_" } readdir $dh;
    closedir($dh);
    die "probes: no *.txt inputs in '$dir' — spec ge.oracle.c2\n" unless @inputs;

    my $env_seed = $ENV{PERL_HASH_SEED} // '';
    my $seeds_note = $env_seed ne '' ? "pinned=$env_seed" : 'UNPINNED';

    open(my $o, '>', $out) or die "probes: cannot write '$out': $!\n";
    print $o "# admission: $PIN perl=$^V seeds=@{[scalar @SEEDS]} seed-pin=$seeds_note envelope=${ENVELOPE_SECONDS}s\n";

    for my $input (@inputs) {
        my @txt_digests = ();
        my @ascii_digests = ();
        my $seconds = 0;
        my $error;
        for my $seed (@SEEDS) {
            my $child_seed = $seed == 0 && $env_seed ne '' ? $env_seed : $seed;
            # the child must start with the seed already in its environment
            local $ENV{PERL_HASH_SEED} = $child_seed;
            my ($status, $out, $stderr) = run_perl_snippet($CHILD_SRC, 30, "$dir/$input");
            if ($status ne 'ok') {
                chomp(my $msg = $stderr // "child $status");
                $error = $msg;
                last;
            }
            my ($dt, $da) = $out =~ /^DIGESTS txt=(\S+) ascii=(\S+)\n/m;
            push @txt_digests,   $dt;
            push @ascii_digests, $da;
            $seconds = $1 if $stderr =~ /SECONDS=([\d.]+)/;
        }

        my ($verdict, $seconds_out);
        if (defined $error) {
            $verdict = 'render-error';
            $seconds_out = '999.999';
        }
        else {
            my $stable = (uniq(@txt_digests) == 1) && (uniq(@ascii_digests) == 1);
            $verdict = $stable ? 'stable' : 'hash-dependent';
            $seconds_out = sprintf('%.3f', $seconds);
        }
        my $base = $input;
        $base =~ s/\.txt$//;
        print $o "$base\t$verdict\t$seconds_out\n";
        print "$base: $verdict ($seconds_out s)\n";
    }
    close($o);
    print "admission manifest written: $out (", scalar(@inputs), " inputs)\n";
    exit 0;
}

die $usage;
