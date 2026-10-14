# Oracle→trace normalization table (D2 — committed, tested artifact)

> The oracle→Dec normalization is an explicit table, not prose (D2 /
> CORR-001). The pairwise faithfulness criterion is restated and gated on
> E3 (gently-ugy, PASS 2026-10-14: computable on the five committed
> `_prepare_layout` dumps with zero additional upstream hooks).
>
> Scope: the E2 dump grammar — `_prepare_layout` stage dumps from
> `tests/repro/probes.pl dump` (pin header, `-- seed N` blocks,
> `== invocation` blocks, `band y=` lines, `name @ (x,y) WxH [cx=] [cy=]`
> lines). The table maps every dump construct onto a normalized
> `layout.place` trace event.
>
> Each row is enforced by a named subtest in
> `crates/gently-core/tests/e3_faithfulness.rs` (`row_nN_*`), so a table
> edit without a matching normalizer change fails the corpus test.

| row | oracle construct | normalized event | enforcement |
|-----|------------------|------------------|-------------|
| N1 | `# dump:` header line | pin metadata only — **not** a trace event | dropped by the block splitter (`row_n1_header_is_metadata`) |
| N2 | `-- seed N` marker | block scope key (raw image tagging only) — **not** a trace event | seeds never appear in normalized event keys (`row_n2_seed_not_an_event`) |
| N3 | `== invocation K (caseM, format: F)` | scope keys `inv=K case=M` on every event in that block | keys carry inv + case (`row_n3_invocation_scope_keys`) |
| N4 | `band y=Y nodes=A edge-cells=B group-cells=C` | `layout.place[inv=K case=M band y=Y nodes=A edge-cells=B group-cells=C]` | bands become band events with verbatim counts (`row_n4_band_event`) |
| N5 | `name @ (X,Y) WxH` | `layout.place[inv=K case=M node=name @ (X,Y) WxH cx=1 cy=1]` — spans default to 1 when absent | span defaults (`row_n5_node_span_defaults`) |
| N6 | `name @ (X,Y) WxH cx=C cy=H` | same event with cx=C cy=H | explicit spans carried verbatim (`row_n6_node_explicit_spans`) |
| N7 | any line ordering | canonical emission sorted by (inv, case, band y, name); ordering never leaks into the trace | order-insensitivity (`row_n7_ordering_never_leaks`, plus the E3 clause-2 test) |
| N8 | events keyed on (inv, case, band y) / (…, name) | duplicate oracle states dedup; distinct states stay distinct | dedup/identity (`row_n8_key_dedup`) |

## Faithfulness criterion (restated, pairwise — tested as E3 clauses)

1. For every pair of corpus fixtures whose oracle dumps differ at stage
   S, the normalized traces must differ in S's image (soundness —
   normalization erases no real divergence; asserted pairwise over the
   5 fixtures AND pairwise across seeds within a fixture).
2. Per fixture, canonically-ordered state snapshots normalize to
   identical traces regardless of dump ordering.

## Discrimination evidence (mutation, 2026-10-14)

Dropping node events from the normalizer trips clause 1 at seed
granularity (`fan-layout: seeds differ at layout.place but normalization
collapsed them`) — the criterion is not vacuous.
