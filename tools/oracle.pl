#!/usr/bin/env perl
# Purpose: the oracle runner for the differential fixture corpus under
# tests/fixtures/graph-easy — records Graph::Easy outputs as companions.
# Responsibilities: in `record` mode, for every `*.txt` input write the
# `as_txt` and `as_ascii` companions with a leading `# oracle: ` pin
# header; refuse to re-record a companion whose header already matches the
# current pin (ge.oracle.c5) and regenerate only stale-pinned or missing
# companions. The Graph::Easy version is checked against the pin up front
# and any drift is a typed error naming the remediation (ge.oracle.c4).
# Rationale: recording is rare and deliberate — the runner lives outside
# the cargo workspace so the corpus, not the toolchain, stays the source
# of truth; verification needs no perl at all (`just oracle-verify`
# compares the built gently binary against the committed bytes).
use strict;
use warnings;

# The pinned oracle (specs/ge-oracle.md c1): Graph::Easy v0.69 at
# ededa3d787ad89ac532c578c06390e8a7b270499.
my $PIN_VERSION = '0.69';
my $PIN_COMMIT  = 'ededa3d787ad89ac532c578c06390e8a7b270499';
my $HEADER      = "# oracle: Graph::Easy v$PIN_VERSION @ $PIN_COMMIT\n";

my $mode = shift @ARGV // '';
my $dir  = shift @ARGV
    // die "oracle: usage: tools/oracle.pl record <fixtures-dir>\n";

die "oracle: unknown mode '$mode' — usage: tools/oracle.pl record <fixtures-dir>\n"
    unless $mode eq 'record';

# ge.oracle.c4: drifted environment is a typed error naming the pin and
# the remediation, before any companion is touched.
my $version = eval { Graph::Easy->VERSION } // 'not installed';
unless (defined $version && $version eq $PIN_VERSION) {
    die <<"EOF";
oracle: installed Graph::Easy is '$version', pin is v$PIN_VERSION \@ $PIN_COMMIT — spec ge.oracle.c4
remediation: cpanm Graph::Easy==$PIN_VERSION
   or fetch the pinned dist isolated and put it on PERL5LIB:
     mkdir -p /var/tmp/ge$PIN_VERSION && cd /var/tmp/ge$PIN_VERSION
     curl -fsSLO https://backpan.perl.org/authors/id/S/SH/SHLOMIF/Graph-Easy-$PIN_VERSION.tar.gz
     tar xzf Graph-Easy-$PIN_VERSION.tar.gz
     PERL5LIB=/var/tmp/ge$PIN_VERSION/Graph-Easy-$PIN_VERSION/lib just oracle-record
EOF
}

use Graph::Easy::Parser;

opendir(my $dh, $dir) or die "oracle: cannot open fixture dir '$dir': $!\n";
my @inputs = sort grep { /\.txt$/ && -f "$dir/$_" } readdir $dh;
closedir($dh);

die "oracle: no *.txt inputs in '$dir' — spec ge.oracle.c2\n" unless @inputs;

my $recorded = 0;
for my $input (@inputs) {
    my $base   = $input;
    $base =~ s/\.txt$//;
    open(my $in, '<', "$dir/$input") or die "oracle: cannot read '$dir/$input': $!\n";
    my $text = do { local $/; <$in> };
    close($in);

    my $graph = Graph::Easy::Parser->new->from_text($text);
    die "oracle: input '$dir/$input' does not parse through the oracle\n"
        unless $graph;

    # Companion naming (ge.oracle.c2): <base>.txt.expected holds as_txt,
    # <base>.ascii.expected holds as_ascii — exactly one per format.
    my %outputs = (
        "$base.txt.expected"   => $graph->as_txt,
        "$base.ascii.expected" => $graph->as_ascii,
    );
    for my $companion (sort keys %outputs) {
        my $path = "$dir/$companion";
        if (open(my $old, '<', $path)) {
            my $header = <$old>;
            close($old);
            if (defined $header && $header eq $HEADER) {
                # ge.oracle.c5: the pin is unchanged — refuse to overwrite.
                print "oracle-record: $companion pin unchanged — refusing to overwrite"
                    . " (re-recording requires a pin change, spec ge.oracle.c5)\n";
                next;
            }
            # Stale pin (or a headerless stray) — deliberate pin change,
            # regenerate (ge.oracle.c5).
            print "oracle-record: pin changed — regenerating $companion\n";
        }
        open(my $out, '>', $path) or die "oracle: cannot write '$path': $!\n";
        print $out $HEADER, $outputs{$companion};
        close($out);
        print "oracle-record: recorded $companion\n";
        $recorded++;
    }
}

print "oracle-record: done — $recorded companion(s) written, pin v$PIN_VERSION \@ $PIN_COMMIT\n";