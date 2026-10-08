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

# Record oracle fixtures for new inputs (spec: ge.oracle.c5)
oracle-record:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v perl >/dev/null || { echo "oracle: perl not found — spec ge.oracle.c4 (remediation: install perl)"; exit 1; }
    perl -MGraph::Easy -e 'exit((Graph::Easy->VERSION eq "0.69") ? 0 : 1)' \
        || { echo "oracle: installed Graph::Easy differs from pin v0.69 @ ededa3d7 — spec ge.oracle.c4 (remediation: cpanm Graph::Easy==0.69 or just oracle-pin)"; exit 1; }
    echo "oracle-record: implementation phase not started — runner spec'd in specs/ge-oracle.md c5"; exit 1

# Verify gently output byte-identically against recorded oracle outputs (spec: ge.oracle.c3)
oracle-verify:
    #!/usr/bin/env bash
    set -euo pipefail
    [ -d tests/fixtures/graph-easy ] || { echo "oracle: no fixture corpus at tests/fixtures/graph-easy — spec ge.oracle.c2"; exit 1; }
    echo "oracle-verify: implementation phase not started — runner spec'd in specs/ge-oracle.md c3"; exit 1

# Run the performance budget gates (spec: ge.perf c1-c4)
perf-check:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "perf-check: implementation phase not started — budgets spec'd in specs/ge-perf.md c1-c3"; exit 1
