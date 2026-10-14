---
id: ge.dot_parser
kind: intent
statement: "WHERE the dot-parser feature is included THE dot-parser SHALL map a Graphviz DOT graph onto the graph model, covering node statements, edge statements with attribute lists, subgraph blocks, and record and HTML-like labels with port references."
---

# dot-parser

An optional feature that reads Graphviz DOT (the input side
Graph::Easy provides via `Graph::Easy::Parser::Graphviz`) and feeds
[[ge.graph_model]]. Scope: digraph/graph headers, node and edge
statements, attribute lists and graph-scope attribute statements, and
subgraph blocks — records, HTML-like
labels, and port references included (they parse into numbered part
nodes, per the oracle). The oracle's own limitations carry over:
record labels with nested braces split by the flat rule rather than
Graphviz's nesting semantics, and the header does not fix edge
direction — the edge operator does.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | A `graph` header sets the model graph's `type` attribute to `undirected` and a `digraph` header leaves the model graph at its directed default; a named header sets the model graph's `title` to its name verbatim and an unnamed header sets it to `unnamed`; neither header fixes the direction of individual edges. | [[ge.dot_parser]] | |
| c2 | invariant | An edge chain `a -> b -> c;` creates one model edge per arrow, and each edge's direction follows its operator — `->` yields a directed edge and `--` yields an undirected edge — independently of the graph header. | [[ge.dot_parser]] | |
| c3 | invariant | An attribute list `[key=val, key2=val2]` attached to a node or edge maps onto the model attributes of that object, with quoted values unescaped once; a graph-scope attribute statement `key=value` (quoted or bare value) applies to the model graph — or to the innermost enclosing named group — with `rankdir` remapped to `flow` per the oracle's direction map (LR→east, RL→west, TB→south, BT→north, unrecognized→east); every other key lands verbatim, the parser neither validates names nor remaps beyond `rankdir`. | [[ge.dot_parser]] | [[ge.dot_parser.c5]] |
| c4 | invariant | The parser links edges through a node stack that survives scope boundaries: a node statement in a group scope (the graph body or a named subgraph) replaces the stack, a node statement in a bare `{ }` scope pushes onto it, and an edge operator links every node on the stack to its right endpoint — so a bare scope or an inline named subgraph used as an edge-chain endpoint links every node on the surviving stack (leaked earlier entries first) to the chain target, while a plain `}` resets the stack, a scope with no inner edge links nothing, and an operator over an empty stack fails as not recognized. A subgraph block with a name maps to a model group whose name is the subgraph's name verbatim — except for the graphviz cluster convention (gently-j2i): an id `cluster_<name>` maps to group `<name>` and an id `cluster<N>` (bare digits after `cluster`) maps to an anonymous group — containing only the nodes declared directly inside it. | [[ge.dot_parser]] | |
| c5 | effect | A record label — a `shape=record` node whose label contains a vertical bar — or an HTML-like table label parses into numbered part nodes named `name.N`, with port markers stripped and edges reattached to the referenced part; a construct the parser cannot tokenize — such as the `subgraph` keyword without a name or a malformed HTML-like label — and a port reference with no matching part fail with a typed error, and no partial graph is returned. | [[ge.dot_parser]] | |
| c6 | invariant | The parser seeds the model graph with the upstream Graphviz parser's defaults (Graph::Easy::Parser::Graphviz::reset — "graphviz has different defaults as Graph::Easy"): the graph carries `flow: south` and `colorscheme: x11` and the edge base class carries `arrowstyle: filled` before the first statement is read, so an explicit `rankdir` (remapped per [[ge.dot_parser.c3]]), `flow`, or `colorscheme` statement overrides them; the group base class carries `align: center` and `fill: inherit` once a group is created — the upstream post-parse attribute table surfaces the group defaults only when a group exists (gently-cij; probe evidence tests/repro/claims/cli-input-from.observed). | [[ge.dot_parser]] | |

## Model

### States

- `reading`
- `accepted`
- `dot_rejected` emits: [[ge.dot_parser.c5]]

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | reading | accepted | [[ge.dot_parser.c1]] [[ge.dot_parser.c6]] |
| t2 | reading | dot_rejected | [[ge.dot_parser.c1]] [[ge.dot_parser.c6]] |
| t3 | accepted | reading | [[ge.dot_parser.c3]] |


## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.dot_parser.c1]] | DOT headers of both kinds, named and unnamed | a `graph` header yields a model graph with `type: undirected` set and a `digraph` header yields one with `type` unset; a named header yields `title` verbatim and an unnamed header yields `unnamed` |
| p2 | unit | [[ge.dot_parser.c2]] | DOT edge chains under all four header×operator combinations | one model edge per arrow, directed exactly when the operator is `->` and undirected when it is `--`, independently of the header |
| p3 | unit | [[ge.dot_parser.c3]] | attribute lists with quoted, escaped, and bare values, and graph-scope attribute statements (quoted and bare values, all four rankdir directions, unrecognized values, bare-scope nesting, repeated statements) | attribute values land unescaped exactly once on the right object; a graph-scope statement lands on the model graph with `rankdir` remapped to `flow` per the oracle's map, the last statement winning |
| p4 | unit | [[ge.dot_parser.c4]] | named, nested, bare-scope, and cluster-convention subgraphs; bare scopes and named subgraphs as edge-chain endpoints; leaked stack entries, scopes with no inner edge, and empty-stack operators | a named subgraph becomes a group with the name verbatim containing only its directly declared nodes (nested nodes belong to the innermost group), with the graphviz cluster convention pinned — `cluster_x` re-parses as group `x` and a bare `cluster0` as an anonymous group — and a bare `{ }` scope keeps its nodes ungrouped; as an edge-chain endpoint a bare scope or named subgraph links every node on the surviving stack — leaked earlier entries first, no node for a subgraph's name — a scope with no inner edge links nothing and its `}` cuts the leak, and an operator over an empty stack is the oracle's not-recognized error |
| p5 | unit | [[ge.dot_parser.c5]] | record labels with and without ports, HTML-like table labels, and the failing constructs (nameless `subgraph`, malformed HTML-like label, unresolvable port reference) | record and HTML-like table labels split into `name.N` part nodes with port markers stripped and edges reattached to the referenced parts, while the failing constructs each produce the oracle's typed error — tokenizing failures quote the offending input, unresolvable port references name the `base:port` and edge id — and return no partial graph |
| p6 | unit | [[ge.dot_parser.c6]] | digraph and graph headers with and without explicit `rankdir`/`flow`/`colorscheme` statements, with and without a named subgraph | the defaults land exactly as observed — `flow: south`, `colorscheme: x11`, and edge-class `arrowstyle: filled` always; group-class `align: center` and `fill: inherit` only once a group exists; an explicit statement wins over the default it overrides |
