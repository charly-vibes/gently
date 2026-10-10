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
# ededa3d787ad89ac532c578c06390e8a7b270499, run from the pinned source
# checkout under the pinned hash seed (recordings only).
my $PIN_VERSION = '0.69';
my $PIN_COMMIT  = 'ededa3d787ad89ac532c578c06390e8a7b270499';
my $PIN_LIB     = "/var/tmp/ge$PIN_VERSION/Graph-Easy-$PIN_VERSION/lib";
my $HEADER      = "# oracle: Graph::Easy v$PIN_VERSION @ $PIN_COMMIT\n";
my $PERL_V      = "$^V";   # e.g. v5.44.0

sub fail_pin {
    my ($what) = @_;
    die <<"EOF";
oracle: $what — spec ge.oracle.c4
remediation: recordings run against the pinned source checkout, never a
  cpan-installed copy (a dist tarball cannot be verified against the pin
  commit; a git clone checked out at the commit can):
     git clone https://github.com/shlomif/Graph-Easy /var/tmp/ge$PIN_VERSION/Graph-Easy-$PIN_VERSION
     git -C /var/tmp/ge$PIN_VERSION/Graph-Easy-$PIN_VERSION checkout $PIN_COMMIT
     PERL_HASH_SEED=0 PERL5LIB=$PIN_LIB just oracle-record
EOF
}

# ge.oracle.c4: drifted environment is a typed error naming the pin and
# the remediation, before any companion is touched. The pin is the full
# environment contract: Graph::Easy version, source-checkout load path,
# and an explicitly pinned PERL_HASH_SEED (byte-identical recordings are
# only reproducible at the same seed — c1/c3).
my $version = eval { Graph::Easy->VERSION } // 'not installed';
fail_pin("installed Graph::Easy is '$version', pin is v$PIN_VERSION \@ $PIN_COMMIT")
    unless defined $version && $version eq $PIN_VERSION;
my $load_path = $INC{'Graph/Easy.pm'} // 'not loaded';
fail_pin("Graph::Easy loaded from '$load_path', pin requires the source checkout $PIN_LIB")
    unless index($load_path, $PIN_LIB) == 0;
fail_pin("PERL_HASH_SEED is unset — hash randomization is live, recordings would not reproduce")
    unless defined $ENV{PERL_HASH_SEED} && $ENV{PERL_HASH_SEED} =~ /^\d+$/;
my $META_LINE = "# oracle: perl $PERL_V PERL_HASH_SEED=$ENV{PERL_HASH_SEED}\n";

use Graph::Easy::Parser;

my $mode = shift @ARGV // '';
my $dir  = shift @ARGV
    // die "oracle: usage: tools/oracle.pl record <fixtures-dir>\n";

die "oracle: unknown mode '$mode' — usage: tools/oracle.pl record <fixtures-dir>\n"
    unless $mode eq 'record';

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
        print $out $HEADER, $META_LINE, $outputs{$companion};
        close($out);
        print "oracle-record: recorded $companion\n";
        $recorded++;
    }
}

print "oracle-record: done — $recorded companion(s) written, pin v$PIN_VERSION \@ $PIN_COMMIT\n";