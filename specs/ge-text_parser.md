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

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | A bracketed token `[ Name ]` creates a named node; the bare token `[ ]` creates an anonymous node that cannot be referenced again after creation. | [[ge.text_parser]] | |
| c2 | invariant | The edge operators `->`, `=>`, `.>`, `~>`, `- >`, `.->`, `..->`, `= >` map to styles solid, double, dotted, wave, dashed, dot-dash, dot-dot-dash, and double-dash; the styles bold, wide, and broad are settable only via the `style` attribute. | [[ge.text_parser]] | |
| c3 | invariant | A chain `A -> B -> C` creates exactly one edge per adjacent pair with the correct endpoints, sharing node objects for repeated names. | [[ge.text_parser]] | |
| c4 | invariant | An inline edge label `-- label -->` sets the edge's label only when both flanking patterns match and the edge has an arrow; an arrow-less edge with an inline label is a parse error. | [[ge.text_parser]] | |
| c5 | invariant | An attribute block `{ key: value; }` applies to the nearest preceding object (node, edge, or group), and class sections `graph { ... }`, `node { ... }`, `edge { ... }`, `group { ... }` apply attributes to the whole class. | [[ge.text_parser]] | |
| c6 | invariant | A group block `( Name: ... )` or an anonymous group `( [ A ] )` produces a group containing exactly the nodes declared inside it, and later `[ Name ]` references reuse the existing node. | [[ge.text_parser]] | |
| c7 | invariant | Comment lines beginning with `#` are ignored everywhere outside of quoted strings. | [[ge.text_parser]] | [[ge.text_parser.c8]] |
| c8 | effect | A malformed input line produces a typed parse error naming the source line number and the reason. | [[ge.text_parser]] | |
| c9 | invariant | Repeated edge patterns must repeat their unit fully (e.g. `..-..-..->` is valid dot-dot-dash while `..-..-..>` is a parse error); arrow-less patterns (`---`, `.-.-`, `= =`) require at least two full repetitions, and single-character arrow-less patterns (`-`, `~`, `.-`) are parse errors. | [[ge.text_parser]] | |

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
| p1 | unit | [[ge.text_parser.c1]] | arbitrary node tokens, named and anonymous | every named bracket token yields a model node with that name; every bare `[ ]` yields a node that is never reused by a later reference |
| p2 | unit | [[ge.text_parser.c2]] | arbitrary operator patterns from the style grammar | each valid operator maps to the documented style; style-only styles are accepted via the attribute and rejected as operators |
| p9 | unit | [[ge.text_parser.c9]] | repeated and arrow-less operator patterns, valid and invalid | each invalid repetition is rejected and each valid repetition parses to the right style |
| p3 | unit | [[ge.text_parser.c3]] | arbitrary node chains of length 1 to N | the parsed model has exactly one edge per adjacent pair and shared endpoints for repeated names |
| p4 | unit | [[ge.text_parser.c4]] | labelled edges with matching, mismatching, and arrow-less flanking patterns | valid labelled arrows set the edge label; mismatched flanks and arrow-less inline labels are rejected |
| p5 | unit | [[ge.text_parser.c5]] | attribute blocks and class sections over all class scopes | each attribute lands on exactly the nearest preceding object or the declared class |
| p6 | unit | [[ge.text_parser.c6]] | group blocks, nested groups, and anonymous groups | group membership in the model equals the nodes declared inside the block |
| p7 | unit | [[ge.text_parser.c7]] | inputs with interleaved comments and quoted strings containing `#` | comments are dropped and in-string `#` characters survive into attribute values |
| p8 | unit | [[ge.text_parser.c8]] | malformed inputs: bad operators, dangling brackets, unknown classes | each error names a line number and a reason and aborts without producing a graph |
