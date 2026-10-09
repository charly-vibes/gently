# Design: ge.txt_render completion (c1–c4, p1–p4)

## Decision

Deepen the tracer-era txt serializer to the full ge.txt_render spec by
adapting upstream `AsTxt.pm` behavior, with tests aligned to the already-
landed contract naming:

1. **Contract naming is already correct** — `scenarios::ge_txt_render::p1`
   … `p4` unit tests exist in `scenarios.rs` `mod ge_txt_render`. The RED
   state exists as-is (2 fail / 2 pass today: p1 class-sections and p3
   round-trip fail on the tracer-era serializer, p2/p4 pass on the
   supported feature subset).
2. **The sanctioned fix for contract-test correspondence** is editing
   `.espectacular/ge-txt_render/p1.toml` … `p4.toml`: change
   `flags = "scenarios::ge-txt_render::pN"` (hyphen — unmatchable) to
   `flags = "scenarios::ge_txt_render::pN"` (underscore). This is in
   scope for the ge.txt_render bead (gently-3hv), not another spec's
   contract.
3. **Upstream corpus is authoritative**: `AsTxt.pm` (operator/style
   mapping, class-section emission order, instance-attribute format)
   and its test fixtures. txt form must match upstream; c4/p4 fixtures
   land under `tests/fixtures/graph-easy/` pinned to one upstream
   revision with a documented re-recording policy.
4. **Round-trip is bound to the existing parser feature set** — the
   parser API (`crates/gently-core/src/parse/`) is another spec's binding
   contract; c3/p3 tests must target only the supported feature subset.
   If a feature set gap appears, document the deviation and coordinate
   via the orchestrator rather than expanding the parser contract.
5. **RED→GREEN→REFACTOR in separate commits** (ticket boilerplate).

## Risks

- Class-section emission touches only output; the tracer pipeline
  (parse → layout → render txt/ascii → CLI) depends on `txt::render` —
  public tracer behavior (byte-identical oracle) must stay green.
- Fixture re-recording is a deliberate, separately reviewed change, never
  a side effect of code edits.
- Clippy `-D warnings` on internal refactors; no behavior change on the
  REFACTOR commit.
