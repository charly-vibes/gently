---
date: 2026-10-13
project: graph-easy-port
phase: research
---

# Session Handoff

## What Was Done

<!-- Summary of completed work -->

## Key Decisions

<!-- Decisions made and rationale -->

## Gotchas & Surprises

<!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

## What Took Longer Than Expected

<!-- Steps that needed multiple attempts. Commands that failed before the right one. -->

## Open Questions

<!-- Unresolved questions -->

## Next Steps

<!-- Prioritized list of what to do next -->

## Context

### git_status

```
 M .wai/pipeline-runs/epic-orchestrator-2026-10-13-gently-a0k-dot-parser-graph-scope-attribute-statements.yml
 M .whisper/branches/main/notes.md
 M .whisper/branches/main/notes.usage.jsonl
```

### open_issues

```
○ gently-527 P2 [bug] so-flowchart-size: literal \n in edge label printed raw instead of two-row label split (SO 47323345)
○ gently-58m P2 [bug] gently cannot parse anonymous-group syntax '( [ Bonn ] ) -> [ Berlin ]' from Parser.pm POD
○ gently-5vf P2 [bug] so-graph-convert: vertical stem column off by one (2-space vs 3-space indent) on SO 71869025 URI tree
○ gently-7rx P2 [bug] gently renders node groups without the dashed border box; oracle draws '+ - - -+' border with group title
○ gently-89r P2 gently: start:/end: port attributes ignored in layout
○ gently-8xd P2 gently: trailing node attribute block after node name silently dropped ({ label: "Huge"; })
○ gently-b0z P2 [bug] gently renders anonymous nodes as boxed '#N'; oracle renders them boxless/unnamed in as_ascii
○ gently-bvb P2 [bug] gently computes ASCII-art label width in Unicode chars; oracle counts UTF-8 bytes ('Köln' box is 1 col wider in oracle)
○ gently-byq P2 [bug] so-start-port-index: indexed start/end port attrs {start: front,N} ignored (SO 47323345 community workaround)
○ gently-cbk P2 gently: shape:edge nodes — render error "edge path cells are not orthogonally adjacent"
○ gently-emd P2 gently: textwrap: auto / textwrap edge attribute ignored (labels not wrapped)
○ gently-fb1 P2 [bug] dot_parser: bare scope nested in a named subgraph joins the enclosing group in the oracle; gently keeps its nodes ungrouped
○ gently-klf P2 [bug] gently triangle layout diverges from oracle by 2 columns on the back-edge ([ Bonn ] -> [ Berlin ] -> [ Hamburg ] + [ Bonn ] -> [ Hamburg ])
○ gently-m4i P2 gently: comma-chained node lists ([ A ], [ B ] --> [ C ]) — parser error
○ gently-y7i P2 gently: \l \r \c label alignment escapes rendered raw
○ gently-yhg P2 gently: minlen edge attribute ignored in layout

--------------------------------------------------------------------------------
Total: 16 issues (16 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```

