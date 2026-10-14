#!/usr/bin/env bash
# E1 retrospective localization driver (gently-asc) — decision-trace prototype.
#
# For each case, builds throwaway worktrees at fix^ and fix (shared target
# dir), runs the fixture with GENTLY_TRACE=<pre-registered stage>, and diffs
# traces against the pre-registered reference:
#   - gently-awd  (fix 8a0d56b): stage layout.place; reference = the E2
#     oracle _prepare_layout dump (tests/repro/dumps/so-flowchart-size.dump.txt)
#     aligned to placement lines, + size-attr.txt.expected bytes.
#   - gently-4ga  (fix 280f9dc): stage render.route; reference = the fix
#     commit's own trace (the oracle has no routing trace).
#
# Emits the trace diffs + first differing event group (the minimal cut
# candidate) to stdout; the verdict lives in README.md.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
step() { CMDS=$((CMDS+1)); }
CMDS=0
step; git worktree prune

AWD_FIX=8a0d56b
GA_FIX=280f9dc
TARGET=/tmp/e1-target
RUNS=/tmp/e1-runs
mkdir -p "$RUNS"
T0=$(date +%s)

# build_at <rev> <id> — worktree at rev, inject the env-gated trace
# prototype from the CURRENT checkout (trace.rs + `mod trace;` + the
# emission block extracted from layout.rs), build, echo binary path.
build_at() {
  local rev=$1
  local id=$2
  local wt="/tmp/e1-$id"
  step; if [ ! -d "$wt" ]; then git worktree add --detach "$wt" "$rev" >/dev/null; fi
  step; cp crates/gently-core/src/trace.rs "$wt/crates/gently-core/src/trace.rs"
  step; grep -q '^mod trace;' "$wt/crates/gently-core/src/lib.rs" || \
    perl -0pi -e 's/\npub mod render;/\npub mod render;\nmod trace;/' \
    "$wt/crates/gently-core/src/lib.rs"
  # Extract the emission block (comment marker through the labels anchor)
  # from the current checkout — single source of truth.
  step; awk '/^    \/\/ Env-gated decision-trace prototype/{f=1} f{print} /^    let labels = label_cells/{exit}' \
    crates/gently-core/src/layout.rs > /tmp/e1-emit.rs
  step; grep -q 'Env-gated decision-trace prototype' "$wt/crates/gently-core/src/layout.rs" || \
    perl -0pi -e 'BEGIN{ local $/; open F, "<", "/tmp/e1-emit.rs" or die; $b=<F>; close F } s/^    let labels = label_cells\(graph, &edge_paths\);$/$b/m' \
    "$wt/crates/gently-core/src/layout.rs"
  step; (cd "$wt" && CARGO_TARGET_DIR="$TARGET" cargo build -p gently-cli >/dev/null 2>&1)
  # Per-rev binary: the shared target dir overwrites debug/gently on every
  # build, so each revision's binary must be captured separately.
  step; cp "$TARGET/debug/gently" "$RUNS/gently-$id"
  echo "$RUNS/gently-$id"
}

# trace <binary> <fixture> <stage> <out> — run with trace; errors tolerated.
trace() {
  local bin=$1 fx=$2 stage=$3 out=$4
  step; rm -f "$out"
  step; GENTLY_TRACE="$stage" GENTLY_TRACE_FILE="$out" "$bin" "$fx" --as ascii >/dev/null 2>&1 || true
}

# align_dump <dump> — normalize the oracle dump's band-0 node lines of the
# first invocation to placement-line form. The alignment shim: a dump line
# `  Client @ (8,0) 10x10 cx=5 cy=10` becomes `place Client x=8 y=0 cx=5 cy=10`;
# a single-celled node (`  Server @ (0,0) 10x4`, no cx/cy) maps to cx=1 cy=1.
# NOTE: the oracle's cx/cy are its CELL spans; gently's trace cx/cy are the
# same cell-span units, so the lines are directly comparable.
align_dump() {
  awk '/^band y=/{f=($0 ~ /band y=0 /)} /^  /{if(f) print} /^-- seed 1/{exit}' "$1" \
    | sed -E 's/^  (.+) @ \(([0-9]+),([0-9]+)\) [0-9]+x[0-9]+( cx=([0-9]+) cy=([0-9]+))?$/place \1 x=\2 y=\3 cx=\5 cy=\6/' \
    | sed -E 's/cx= cy=/cx=1 cy=1/'
}

echo "== case gently-awd (stage: layout.place, fix $AWD_FIX) =="
AWD_PRE=$(build_at "$AWD_FIX^" awd-pre)
AWD_POST=$(build_at "$AWD_FIX" awd-post)
trace "$AWD_PRE" tests/fixtures/graph-easy/size-attr.txt layout.place "$RUNS/awd-pre.trace"
trace "$AWD_POST" tests/fixtures/graph-easy/size-attr.txt layout.place "$RUNS/awd-post.trace"
step; "$AWD_PRE" tests/fixtures/graph-easy/size-attr.txt --as ascii > "$RUNS/awd-pre.ascii" || true
step; "$AWD_POST" tests/fixtures/graph-easy/size-attr.txt --as ascii > "$RUNS/awd-post.ascii" || true
# strip the pinned `# oracle:` metadata header lines from expected bytes
expect_bytes() { grep -v '^# oracle:' tests/fixtures/graph-easy/size-attr.ascii.expected; }

step; if "$AWD_POST" tests/fixtures/graph-easy/size-attr.txt --as ascii 2>/dev/null | cmp -s - <(expect_bytes); then
  echo "divergence-confirmation: post-fix renders size-attr.txt BYTE-IDENTICAL to .ascii.expected (fix is the reference)"
else
  echo "divergence-confirmation: post-fix DOES NOT match .ascii.expected (unexpected)"
fi
step; if "$AWD_PRE" tests/fixtures/graph-easy/size-attr.txt --as ascii 2>/dev/null | cmp -s - <(expect_bytes); then
  echo "divergence-confirmation: pre-fix matches .ascii.expected (unexpected!)"
else
  echo "divergence-confirmation: pre-fix diverges from .ascii.expected (confirmed)"
fi
echo "--- trace diff (pre fix^ vs post fix), size-attr.txt:"
step; diff "$RUNS/awd-pre.trace" "$RUNS/awd-post.trace" || true
# Oracle-side: same input the E2 dump was taken from (so-flowchart-size probe).
step; awk '/pod_probe_body..so-flowchart-size../{f=1;next} f&&/^EXAMPLE$/{exit} f' \
  tests/repro/probes.pl > "$RUNS/so-input.txt"
trace "$AWD_PRE" "$RUNS/so-input.txt" layout.place "$RUNS/awd-so-pre.trace"
trace "$AWD_POST" "$RUNS/so-input.txt" layout.place "$RUNS/awd-so-post.trace"
step; align_dump tests/repro/dumps/so-flowchart-size.dump.txt > "$RUNS/awd-oracle-aligned.trace"
echo "--- aligned oracle dump node lines (seed 0, invocation 1):"
cat "$RUNS/awd-oracle-aligned.trace"
echo "--- trace diff (pre fix^ vs aligned oracle dump), so-flowchart-size input:"
step; diff "$RUNS/awd-oracle-aligned.trace" "$RUNS/awd-so-pre.trace" || true
echo "--- trace diff (post fix vs aligned oracle dump), so-flowchart-size input:"
step; diff "$RUNS/awd-oracle-aligned.trace" "$RUNS/awd-so-post.trace" || true

echo
echo "== case gently-4ga (stage: render.route, fix $GA_FIX) =="
GA_PRE=$(build_at "$GA_FIX^" ga-pre)
GA_POST=$(build_at "$GA_FIX" ga-post)
step; printf '[ A ] -> [ B ], [ C ], [ D ]\n' > "$RUNS/fan.txt"
trace "$GA_PRE" "$RUNS/fan.txt" render.route "$RUNS/ga-pre.trace"
trace "$GA_POST" "$RUNS/fan.txt" render.route "$RUNS/ga-post.trace"
echo "--- pre-fix render error (symptom confirmation):"
step; "$GA_PRE" "$RUNS/fan.txt" --as ascii 2>&1 >/dev/null | head -2 || true
echo "--- trace diff (pre fix^ vs post fix), fan:"
step; diff "$RUNS/ga-pre.trace" "$RUNS/ga-post.trace" || true

echo
T1=$(date +%s)
echo "== cost =="
echo "wall-clock: $((T1-T0))s (cold-ish; shared target dir /tmp/e1-target)"
echo "command count: $CMDS (driver steps; session-equivalents: 1 subagent run)"
