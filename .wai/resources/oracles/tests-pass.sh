#!/usr/bin/env bash
# Oracle: tests-pass — the workspace test suite must be green.
set -euo pipefail
cd "$(dirname "$0")/../../.."
exec cargo test --workspace
