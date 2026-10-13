---
tags: [pipeline-run:epic-orchestrator-2026-10-13-gently-pua-shape-edge-ranking, pipeline-step:gates]
---

GATES: cargo test -p gently-core baseline green — 128 scenarios + 59 lib + 0 fail, 1 ignored (cbk RED scaffold #[ignore]d oracle bytes); p2_edge_node error-pin test green. Full gates (clippy/ah check/just gates/oracle-verify) were green at origin 15fa635; subagent reruns all at completion. Baseline attributable.
