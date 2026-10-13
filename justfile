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

# === Build Commands ===

# Build debug binary
build:
    cargo build

# Build release binary (optimized)
build-release:
    cargo build --release

# Run with arguments (e.g., `just run --as ascii < graph.txt`)
run *args:
    cargo run -p gently-cli -- {{args}}

# Install locally to ~/.cargo/bin
install:
    cargo install --path crates/gently-cli

# === Test Commands ===

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run a specific test (e.g., `just test-one tb::oracle::`)
test-one name:
    cargo test -q -p gently-core --test scenarios {{name}} -- --nocapture

# === Lint Commands ===

# Run clippy linter
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Format all Rust files
fmt:
    cargo fmt

# Check formatting (no changes)
fmt-check:
    cargo fmt --all -- --check

# Show dependency tree
deps:
    cargo tree

# Update dependencies
update:
    cargo update

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

# List the managed ecosystem tools gently gates on
tools:
    @echo "Rust tools:     specodelic (spec lint/compile/model-check gates)"
    @echo "                ah / espectacular (spec-test correspondence, ah check)"
    @echo "Non-Rust tools: bd (beads, Go), openspec, pretender, lefthook"
    @echo
    @echo "Justfile utilities use cargo only; the gates above run in just ci"
    @echo "and in the lefthook pre-commit/pre-push hooks."

# Record oracle fixtures for new inputs (spec: ge.oracle.c1/c5). Runner:
# tools/oracle.pl — needs the pinned source checkout (Graph::Easy 0.69 @
# the pinned commit) on PERL5LIB and an explicit PERL_HASH_SEED (see
# tests/fixtures/graph-easy/README.md 'Recording' for the setup).
oracle-record:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v perl >/dev/null || { echo "oracle: perl not found — spec ge.oracle.c4 (remediation: install perl)"; exit 1; }
    # FIX is overridable so drifted-environment probes (ge-oracle p4) can
    # exercise the recipes against a tempdir corpus, never the repo tree.
    FIX="${FIX:-tests/fixtures/graph-easy}"
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
    # FIX overridable for the p4 drifted-environment probes (see above).
    FIX="${FIX:-tests/fixtures/graph-easy}"
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

# Byte-verify the published examples (gently-ni9.5): for every
# examples/<name>/, render input.gv with the built CLI and diff against
# expected.txt. Fails on any mismatch — no example is published without
# its expected-output file.
examples-verify:
    #!/usr/bin/env bash
    set -euo pipefail
    [ -d examples ] || { echo "examples-verify: no examples/ directory"; exit 1; }
    ls examples/*/input.gv >/dev/null 2>&1 || { echo "examples-verify: no examples/*/input.gv found"; exit 1; }
    for dir in examples/*/; do
        name=$(basename "$dir")
        [ -f "$dir/expected.txt" ] || { echo "examples-verify: $name has no expected.txt — an example is never published without its expected output"; exit 1; }
    done
    cargo build -q -p gently-cli
    BIN=target/debug/gently
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    for dir in examples/*/; do
        name=$(basename "$dir")
        "$BIN" --as ascii < "$dir/input.gv" > "$tmp/out" 2>/dev/null \
            || { echo "examples-verify: gently failed on $name/input.gv"; exit 1; }
        if ! cmp -s "$tmp/out" "$dir/expected.txt"; then
            echo "examples-verify: MISMATCH example=$name — rendered output differs from expected.txt"
            diff "$dir/expected.txt" "$tmp/out" | head -n10 || true
            exit 1
        fi
    done
    echo "examples-verify: all examples byte-identical to their committed expected output"

# Drift gate for the visual gallery (gently-irk): regenerate every
# docs/gallery/<name>.svg through the gallery generator (gently's own DOT
# render) piped through `dot -Tsvg`, and fail on any byte mismatch with the
# committed artifact — same culture as examples-verify. Needs graphviz's
# dot on PATH.
gallery-verify:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v dot >/dev/null || { echo "gallery-verify: dot not found — spec ge.graphviz_render pipeline needs graphviz (remediation: install graphviz)"; exit 1; }
    ls docs/gallery/*.svg >/dev/null 2>&1 || { echo "gallery-verify: no committed SVGs under docs/gallery/"; exit 1; }
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    for svg in docs/gallery/*.svg; do
        name=$(basename "$svg" .svg)
        input="examples/$name/input.gv"
        [ -f "$input" ] || { echo "gallery-verify: orphan SVG $svg — no examples/$name/input.gv"; exit 1; }
        target/debug/examples/gallery "$input" | dot -Tsvg > "$tmp/out" 2>/dev/null \
            || { echo "gallery-verify: regeneration failed for $name"; exit 1; }
        if ! cmp -s "$tmp/out" "$svg"; then
            echo "gallery-verify: MISMATCH example=$name — committed SVG differs from regenerated output"
            diff "$tmp/out" "$svg" | head -n10 || true
            exit 1
        fi
    done
    echo "gallery-verify: all committed SVGs byte-identical to the regenerated path"

# === Utility Commands ===

# Clean build artifacts
clean:
    cargo clean

# Check without building (faster feedback)
check:
    cargo check --workspace

# Show available commands
help:
    @just --list

# Run the performance budget gates (spec: ge.perf c1-c5)
perf-check:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "perf-check: running ge.perf budget gates (specs/ge-perf.md c1-c5, scenarios ge_perf::p1-p5)"
    cargo test -q -p gently-core --test scenarios ge_perf --release -- --nocapture
    echo "perf-check: all budgets green — latency, memory shape, and scaling within spec budgets"
