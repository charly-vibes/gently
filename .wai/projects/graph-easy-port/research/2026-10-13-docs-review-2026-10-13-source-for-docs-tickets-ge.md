---
tags: [pipeline-run:epic-orchestrator-2026-10-13-gently-cbk-shape-edge-edge-cell-render, pipeline-step:claim]
---

DOCS-REVIEW 2026-10-13 (source for docs tickets gently-tyv/lkf/cof/kcq/zf5 @ 9539327): 5-pass review of README.md + examples/README.md + docs/known-hard-cases.md — verdict Needs Revision, 0 Critical / 1 High / 3 Medium / 4 Low. Findings: SCAN-001 High (known-hard-cases prose walls, no visual oracle-vs-gently evidence) -> gently-tyv; INTENT-001 + SCAN-002 (README gates before quickstart, no formats table) -> gently-lkf; EPPO-001/002 (Specodelic/beads/wai undefined) -> gently-cof; ACC-001 (examples README missing build step) -> gently-kcq; INTENT-002 (no how-to guides) -> gently-zf5. Runtime-verified: quickstart bytes exact, --as bogus message matches README claim. Gallery request (CLI+SVG) -> gently-irk P1 + enabler gently-4ig P2 (SVG is NOT upstream v0.69 — no As_svg module; visual path = library graphviz render -> dot -Tsvg).
