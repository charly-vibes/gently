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
    'subgraph-handling'      => 'gently-13f (dot_parser c3)',
    'cli-flags'              => 'gently-0h9 (closed — kept as upstream evidence)',
    'node-unnamed'           => 'gently-r22 (graph_model c1)',
    'href-escaping'          => 'gently-dcp, gently-eyo (html_render c2)',
    'td-colspan'             => 'gently-dcp, gently-eyo (html_render c4)',
    'shape-outline-collapse' => 'gently-dcp, gently-css (ascii shapes; html border-styles)',
    'graphviz-round-trip'    => 'gently-0kg, gently-b4v (graphviz c2/c4, txt_render c3)',
    'size-envelope'          => 'gently-k4u (perf c2/c5)',
    'perl5lib-pin'           => 'gently-ikm, gently-liz (oracle pin contract)',
);

my %CLAIMS = (
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

    'subgraph-handling' => <<'PROBE',
# serves: gently-13f (dot_parser c3)
# claim probed: dot_parser c3 — "only cluster_* subgraphs become groups;
# records and ports rejected".
# Observed: ANY named subgraph becomes a group (cluster_x AND 'named');
# an anonymous subgraph {} is a parse ERROR, not a group.
use Graph::Easy::Parser::Graphviz;
my $p = Graph::Easy::Parser::Graphviz->new();
my $g = $p->from_text("digraph G { a -> b; subgraph cluster_x { c -> d } subgraph named { e -> f } }");
print "named subgraphs parse; groups: ", join(', ', map { $_->name() } $g->groups()), "\n";
eval { $p->from_text("digraph G { a -> b; subgraph { c -> d } }") };
print "anonymous subgraph: ", ($@ ? "ERROR: $@" : "parsed ok"), "\n";
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
        # capture the probe body's stdout so it can be both shown and recorded
        my $buf = '';
        open my $cap, '>>', \$buf or die "probes: cannot capture output: $!\n";
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
