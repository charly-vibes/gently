# gently — task runner
# Spec-defined recipes: ge.oracle (oracle), ge.perf (perf gates).
# Oracle/perf recipes still need the built CLI (spec pointers below);
# the cargo gate is live since the tb.workspace scaffold (gently-2po.1).

default:
    @just --list

# Lint the spec corpus (gate: 0 findings, 0 warnings)
lint:
    specodelic lint --human

# Compile + model-check the spec corpus (pre-push equivalent)
gates:
    specodelic compile specs --human
    specodelic model-check specs --human

# Cargo gates (live since gently-2po.1)
test:
    cargo test --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# ddl-standard CI parity entry point: the same gates lefthook runs, in one
# recipe. CI (.github/workflows/ci.yml) runs exactly this — if CI fails,
# run `just ci` locally to reproduce.
ci:
    just lint
    just gates
    spk lint openspec || test $? -eq 2
    ah check
    just clippy
    just test

# Record oracle fixtures for new inputs (spec: ge.oracle.c1/c5). Runner:
# tools/oracle.pl — needs the pinned source checkout (Graph::Easy 0.69 @
# the pinned commit) on PERL5LIB and an explicit PERL_HASH_SEED (see
# tests/fixtures/graph-easy/README.md 'Recording' for the setup).
oracle-record:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v perl >/dev/null || { echo "oracle: perl not found — spec ge.oracle.c4 (remediation: install perl)"; exit 1; }
    FIX=tests/fixtures/graph-easy
    [ -d "$FIX" ] || { echo "oracle: no fixture corpus at tests/fixtures/graph-easy — spec ge.oracle.c2 (remediation: create it, see $FIX/README.md)"; exit 1; }
    LIB=/var/tmp/ge0.69/Graph-Easy-0.69/lib
    [ -f "$LIB/Graph/Easy.pm" ] || { echo "oracle: pinned source checkout missing at $LIB — spec ge.oracle.c1/c4 (remediation: see $FIX/README.md 'Recording')"; exit 1; }
    export PERL_HASH_SEED="${PERL_HASH_SEED:-0}"
    export PERL5LIB="$LIB${PERL5LIB:+:$PERL5LIB}"
    perl tools/oracle.pl record "$FIX"

# Tier-1 admission sweep (spec ge-oracle.c7/p7): recompute the per-fixture
# hash-stability × envelope classification into tests/repro/admission.tsv.
# Needs the pinned oracle on PERL5LIB (see tests/repro/README.md); the
# runner itself types the remediation on a drifted environment.
repro:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v perl >/dev/null || { echo "repro: perl not found — spec ge.oracle.c4 (remediation: install perl)"; exit 1; }
    FIX=tests/fixtures/graph-easy
    [ -d "$FIX" ] || { echo "repro: no fixture corpus at $FIX — spec ge.oracle.c2"; exit 1; }
    perl tests/repro/probes.pl admit "$FIX"

# List the claim probes and the beads each serves
repro-claims:
    perl tests/repro/probes.pl claims

# Verify gently output byte-identically against recorded oracle outputs
# (spec: ge.oracle.c3). Needs no perl: ascii goes through the built
# gently binary, txt through the tb::oracle differential tests driving
# gently-core's pipeline.
oracle-verify:
    #!/usr/bin/env bash
    set -euo pipefail
    FIX=tests/fixtures/graph-easy
    PIN="# oracle: Graph::Easy v0.69 @ ededa3d787ad89ac532c578c06390e8a7b270499"
    [ -d "$FIX" ] || { echo "oracle: no fixture corpus at tests/fixtures/graph-easy — spec ge.oracle.c2"; exit 1; }
    ls "$FIX"/*.txt >/dev/null 2>&1 || { echo "oracle: no *.txt inputs in $FIX — spec ge.oracle.c2"; exit 1; }
    # ge.oracle.c4: stale or missing pin headers are typed errors —
    # comparing against bytes recorded from a foreign revision would be
    # worse than failing.
    for exp in "$FIX"/*.expected; do
        [ -f "$exp" ] || { echo "oracle: no recorded companions in $FIX — run just oracle-record (spec ge.oracle.c2)"; exit 1; }
        head -n1 "$exp" | grep -qxF "$PIN" \
            || { echo "oracle: stale pin header in $exp — spec ge.oracle.c4 (remediation: deliberate pin change → just oracle-record)"; exit 1; }
    done
    # ge.oracle.c2: every input has exactly one companion per format, no
    # orphans.
    for input in "$FIX"/*.txt; do
        base=$(basename "$input" .txt)
        for fmt in txt ascii; do
            [ -f "$FIX/$base.$fmt.expected" ] \
                || { echo "oracle: missing recorded companion for $base.$fmt.expected — run just oracle-record (spec ge.oracle.c2)"; exit 1; }
        done
    done
    for exp in "$FIX"/*.expected; do
        base=$(basename "$exp" | sed -E 's/\.(txt|ascii)\.expected$//')
        [ -f "$FIX/$base.txt" ] \
            || { echo "oracle: orphan companion $exp has no $FIX/$base.txt input — spec ge.oracle.c2"; exit 1; }
    done
    cargo build -q -p gently-cli
    BIN=target/debug/gently
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    for input in "$FIX"/*.txt; do
        base=$(basename "$input" .txt)
        "$BIN" --as ascii < "$input" > "$tmp/out" \
            || { echo "oracle: gently failed on $input — spec ge.oracle.c3"; exit 1; }
        awk 'done{print;next} /^# oracle: /{next} {done=1;print}' "$FIX/$base.ascii.expected" > "$tmp/body"
        if ! cmp -s "$tmp/out" "$tmp/body"; then
            where=$(cmp "$tmp/out" "$tmp/body" 2>&1 | head -n1 || true)
            echo "oracle-verify: MISMATCH input=$base.txt format=ascii first differing line: $where — spec ge.oracle.c3"
            diff "$tmp/body" "$tmp/out" | head -n10 || true
            exit 1
        fi
    done
    echo "oracle-verify: ascii companions byte-identical (built gently binary)"
    echo "oracle-verify: txt companions verified via the tb::oracle differential tests (gently-core pipeline)"
    cargo test -q -p gently-core --test scenarios tb::oracle::
    echo "oracle-verify: green — corpus verified against pin v0.69 @ ededa3d7"

# Run the performance budget gates (spec: ge.perf c1-c5)
perf-check:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "perf-check: running ge.perf budget gates (specs/ge-perf.md c1-c5, scenarios ge_perf::p1-p5)"
    cargo test -q -p gently-core --test scenarios ge_perf --release -- --nocapture
    echo "perf-check: all budgets green — latency, memory shape, and scaling within spec budgets"
