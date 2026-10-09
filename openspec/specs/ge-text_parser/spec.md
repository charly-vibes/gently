---
id: ge.text_parser
kind: intent
statement: "WHEN text in the Graph::Easy format is parsed THE text-parser SHALL build the graph model with nodes, edges, groups, and attributes exactly as the format grammar defines them."
---

# text-parser

Parses the human-readable Graph::Easy text format (nodes in `[brackets]`,
styled edge operators, `{ attribute: value; }` blocks, `( Groups )`,
class sections) into [[ge.graph_model]]. This is the format Graph::Easy's
own test corpus is written in, so it is also gently's golden-fixture
format.

## Purpose

Parses the human-readable Graph::Easy text format (nodes in `[brackets]`,
styled edge operators, `{ attribute: value; }` blocks, `( Groups )`,
class sections) into [[ge.graph_model]]. This is the format Graph::Easy's
own test corpus is written in, so it is also gently's golden-fixture
format.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | A bracketed token `[ Name ]` creates a named node, and later `[ Name ]` tokens reuse the existing node; the bare token `[ ]` creates an anonymous node that the oracle names `#N` (N an odd counter), which is findable and reusable by that generated name — in text only written escaped (`[ \#N ]`), since an unescaped `#` begins a comment (c7). | [[ge.text_parser]] | |
| c2 | invariant | The edge operators `->`, `=>`, `.>`, `~>`, `- >`, `.->`, `..->`, `= >` map to styles solid, double, dotted, wave, dashed, dot-dash, dot-dot-dash, and double-dash; the styles bold, wide, and broad are settable only via the `style` attribute. A `<` prefix before any of these operators makes the edge bidirectional (`<->` solid, `<=>` double, `<.>` dotted, `<~>` wave, `<- >` dashed, `<.->` dot-dash, `<..->` dot-dot-dash, `<= >` double-dash); the `<` requires a closing `>`, so a lone left arrow is a parse error; the operator further requires a node token on both sides — a missing left node (`--> [ b ]`) or missing right node (`[ a ] -->`) is a parse error. | [[ge.text_parser]] | |
| c3 | invariant | A chain `A -> B -> C` creates exactly one edge per adjacent pair with the correct endpoints, sharing node objects for repeated names. | [[ge.text_parser]] | |
| c4 | invariant | An inline edge label `-- label -->` sets the edge's label only when both flanking patterns match and the edge has an arrow; an arrow-less edge with an inline label is a parse error. | [[ge.text_parser]] | |
| c5 | invariant | An attribute block `{ key: value; }` applies to the nearest preceding object (node, edge, or group), and class sections `graph { ... }`, `node { ... }`, `edge { ... }`, `group { ... }` apply attributes to the whole class. | [[ge.text_parser]] | |
| c6 | invariant | A group block `( Name: ... )` produces a group whose name includes the colon (the oracle names it literally `G:`); a node declared inside belongs to exactly one group — referencing the node again inside a later group moves it there (the earlier group empties). Nested groups: a node declared in an inner group belongs only to that inner group, and the outer group contains only its directly declared nodes. An anonymous group `( [ A ] )` is named `Group #N` by the oracle. | [[ge.text_parser]] | |
| c7 | invariant | An unescaped `#` starts a comment running to the end of the line, everywhere — mid-line and inside quoted attribute values — so `{ label: x # y }` is a parse error and `\#` escapes it; a line that is only a comment is dropped. As a special case, a 3- or 6-digit hex colour token (`#rgb`/`#rrggbb`) immediately following an attribute separator is auto-escaped and accepted. | [[ge.text_parser]] | [[ge.text_parser.c8]] |
| c8 | effect | A malformed input line produces a typed parse error naming the source line number and the reason. | [[ge.text_parser]] | |
| c9 | invariant | A directed edge pattern is one or more unit tokens (each of `= `, `=`, `- `, `-`, `..-`, `.-`, `.`, `~`) followed by `>`, and the edge's style is determined solely by the last unit token — repetitions and mixed units are accepted, so `..-..-..->` is dot-dot-dash while `..-..-..>` is a valid dotted edge. Arrow-less patterns: `.-` and `..-` are valid with a single repetition (dot-dash, dot-dot-dash), while the plain units (`=`, `= `, `-`, `- `, `.`, `~`) require at least two repetitions, and single-character arrow-less patterns are parse errors. | [[ge.text_parser]] | |
| c10 | invariant | An attribute value is finalized with the upstream two-layer unquoting: after the parser-layer unescape and whitespace collapse, the store layer strips one pair of surrounding quotes — either `"` or `'`, mixed ends allowed, greedy first-and-last — then unescapes `\#`, `\"`, `\'`, `\;`, and `\\` to their bare characters; a value whose opening quote is never closed is accepted verbatim (quotes kept) and its first unescaped `;` still terminates the declaration, while a `;` inside a properly closed `"..."` value does not split and mid-value quotes are kept. (Probe evidence: tests/repro/claims/attr-quote-value.observed.) | [[ge.text_parser]] | |

## Model

### States

- `clean`
- `accepted`
- `parse_failed` emits: [[ge.text_parser.c8]]

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | clean | accepted | [[ge.text_parser.c1]] |
| t2 | clean | parse_failed | [[ge.text_parser.c1]] |
| t3 | parse_failed | clean | [[ge.text_parser.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.text_parser.c1]] | arbitrary node tokens, named and anonymous | every named bracket token yields a model node with that name, reused by later references; every bare `[ ]` yields an anonymous node named `#N` (odd counter) that `node('#N')` finds and that a later escaped `[ \#N ]` reference reuses as the same node |
| p2 | unit | [[ge.text_parser.c2]] | arbitrary operator patterns from the style grammar, directed and bidirectional, with and without both endpoint nodes | each valid operator maps to the documented style; `<`-prefixed operators map to the same style with the edge marked bidirectional; a `<` without a closing `>` and an operator with a missing endpoint node are rejected; style-only styles are accepted via the attribute and rejected as operators |
| p9 | unit | [[ge.text_parser.c9]] | directed and arrow-less unit-token patterns, single and repeated | the style follows the last unit token; single-unit arrow-less `.-`/`..-` are accepted; plain units are rejected below two repetitions |
| p3 | unit | [[ge.text_parser.c3]] | arbitrary node chains of length 1 to N | the parsed model has exactly one edge per adjacent pair and shared endpoints for repeated names |
| p4 | unit | [[ge.text_parser.c4]] | labelled edges with matching, mismatching, and arrow-less flanking patterns | valid labelled arrows set the edge label; mismatched flanks and arrow-less inline labels are rejected |
| p5 | unit | [[ge.text_parser.c5]] | attribute blocks and class sections over all class scopes | each attribute lands on exactly the nearest preceding object or the declared class |
| p6 | unit | [[ge.text_parser.c6]] | group blocks, nested groups, anonymous groups, and multi-group references | the group name includes the colon; a node's membership equals the directly declared group it was last declared in; nested inner nodes belong only to the inner group; anonymous groups are named `Group #N` |
| p7 | unit | [[ge.text_parser.c7]] | inputs with interleaved comments, quoted strings containing `#`, and hex colour values | an unescaped `#` truncates the line to a parse error even inside quotes; `\#` survives into attribute values; a hex colour token after an attribute separator is accepted |
| p8 | unit | [[ge.text_parser.c8]] | malformed inputs: bad operators, dangling brackets, unknown classes | each error names a line number and a reason and aborts without producing a graph |
| p10 | unit | [[ge.text_parser.c10]] | attribute values in double-, single-, mixed-, and unterminated-quoted form, with mid-value quotes and escaped `\"`, `\'`, `\;`, `\\` sequences | single-quoted and mixed-end values lose their outer quotes; an unterminated quoted value keeps its quotes and ends at the first unescaped `;`; `\"`, `\'`, `\;`, and `\\` become their bare characters; mid-value quotes and `;` inside properly closed quotes survive |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** arbitrary node tokens, named and anonymous
- **THEN** every named bracket token yields a model node with that name, reused by later references; every bare `[ ]` yields an anonymous node named `#N` (odd counter) that `node('#N')` finds and that a later escaped `[ \#N ]` reference reuses as the same node
- **VERIFIES** [[ge.text_parser.p1]]

#### Scenario: p2

- **WHEN** arbitrary operator patterns from the style grammar, directed and bidirectional, with and without both endpoint nodes
- **THEN** each valid operator maps to the documented style; `<`-prefixed operators map to the same style with the edge marked bidirectional; a `<` without a closing `>` and an operator with a missing endpoint node are rejected; style-only styles are accepted via the attribute and rejected as operators
- **VERIFIES** [[ge.text_parser.p2]]

#### Scenario: p9

- **WHEN** directed and arrow-less unit-token patterns, single and repeated
- **THEN** the style follows the last unit token; single-unit arrow-less `.-`/`..-` are accepted; plain units are rejected below two repetitions
- **VERIFIES** [[ge.text_parser.p9]]

#### Scenario: p3

- **WHEN** arbitrary node chains of length 1 to N
- **THEN** the parsed model has exactly one edge per adjacent pair and shared endpoints for repeated names
- **VERIFIES** [[ge.text_parser.p3]]

#### Scenario: p4

- **WHEN** labelled edges with matching, mismatching, and arrow-less flanking patterns
- **THEN** valid labelled arrows set the edge label; mismatched flanks and arrow-less inline labels are rejected
- **VERIFIES** [[ge.text_parser.p4]]

#### Scenario: p5

- **WHEN** attribute blocks and class sections over all class scopes
- **THEN** each attribute lands on exactly the nearest preceding object or the declared class
- **VERIFIES** [[ge.text_parser.p5]]

#### Scenario: p6

- **WHEN** group blocks, nested groups, anonymous groups, and multi-group references
- **THEN** the group name includes the colon; a node's membership equals the directly declared group it was last declared in; nested inner nodes belong only to the inner group; anonymous groups are named `Group #N`
- **VERIFIES** [[ge.text_parser.p6]]

#### Scenario: p7

- **WHEN** inputs with interleaved comments, quoted strings containing `#`, and hex colour values
- **THEN** an unescaped `#` truncates the line to a parse error even inside quotes; `\#` survives into attribute values; a hex colour token after an attribute separator is accepted
- **VERIFIES** [[ge.text_parser.p7]]

#### Scenario: p8

- **WHEN** malformed inputs: bad operators, dangling brackets, unknown classes
- **THEN** each error names a line number and a reason and aborts without producing a graph
- **VERIFIES** [[ge.text_parser.p8]]

#### Scenario: p10

- **WHEN** attribute values in double-, single-, mixed-, and unterminated-quoted form, with mid-value quotes and escaped `\"`, `\'`, `\;`, `\\` sequences
- **THEN** single-quoted and mixed-end values lose their outer quotes; an unterminated quoted value keeps its quotes and ends at the first unescaped `;`; `\"`, `\'`, `\;`, and `\\` become their bare characters; mid-value quotes and `;` inside properly closed quotes survive
- **VERIFIES** [[ge.text_parser.p10]]

