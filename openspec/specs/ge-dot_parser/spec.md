---
id: ge.dot_parser
kind: intent
statement: "WHERE the dot-parser feature is included THE dot-parser SHALL map a Graphviz DOT graph onto the graph model, covering node statements, edge statements with attribute lists, subgraph blocks, and record and HTML-like labels with port references."
---

# dot-parser

An optional feature that reads Graphviz DOT (the input side
Graph::Easy provides via `Graph::Easy::Parser::Graphviz`) and feeds
[[ge.graph_model]]. Scope: digraph/graph headers, node and edge
statements, attribute lists, and subgraph blocks — records, HTML-like
labels, and port references included (they parse into numbered part
nodes, per the oracle). The oracle's own limitations carry over:
record labels with nested braces split by the flat rule rather than
Graphviz's nesting semantics, and the header does not fix edge
direction — the edge operator does.

## Purpose

An optional feature that reads Graphviz DOT (the input side
Graph::Easy provides via `Graph::Easy::Parser::Graphviz`) and feeds
[[ge.graph_model]]. Scope: digraph/graph headers, node and edge
statements, attribute lists, and subgraph blocks — records, HTML-like
labels, and port references included (they parse into numbered part
nodes, per the oracle). The oracle's own limitations carry over:
record labels with nested braces split by the flat rule rather than
Graphviz's nesting semantics, and the header does not fix edge
direction — the edge operator does.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | A `graph` header sets the model graph's `type` attribute to `undirected` and a `digraph` header leaves the model graph at its directed default; neither header fixes the direction of individual edges. | [[ge.dot_parser]] | |
| c2 | invariant | An edge chain `a -> b -> c;` creates one model edge per arrow, and each edge's direction follows its operator — `->` yields a directed edge and `--` yields an undirected edge — independently of the graph header. | [[ge.dot_parser]] | |
| c3 | invariant | An attribute list `[key=val, key2=val2]` attached to a node or edge maps onto the model attributes of that object, with quoted values unescaped once. | [[ge.dot_parser]] | [[ge.dot_parser.c5]] |
| c4 | invariant | A subgraph block with a name maps to a model group whose name is the subgraph's name verbatim, containing only the nodes declared directly inside it. | [[ge.dot_parser]] | |
| c5 | effect | A record label — a `shape=record` node whose label contains a vertical bar — or an HTML-like table label parses into numbered part nodes named `name.N`, with port markers stripped and edges reattached to the referenced part; a construct the parser cannot tokenize — such as the `subgraph` keyword without a name or a malformed HTML-like label — and a port reference with no matching part fail with a typed error, and no partial graph is returned. | [[ge.dot_parser]] | |

## Model

### States

- `reading`
- `accepted`
- `dot_rejected` emits: [[ge.dot_parser.c5]]

### Transitions

| id | from | to | guard |
|----|------|----|-------|
| t1 | reading | accepted | [[ge.dot_parser.c1]] |
| t2 | reading | dot_rejected | [[ge.dot_parser.c1]] |
| t3 | accepted | reading | [[ge.dot_parser.c3]] |

## Properties

| id | kind | derives_from | generator | predicate |
|----|------|--------------|-----------|-----------|
| p1 | unit | [[ge.dot_parser.c1]] | DOT headers of both kinds | a `graph` header yields a model graph with `type: undirected` set and a `digraph` header yields one with `type` unset |
| p2 | unit | [[ge.dot_parser.c2]] | DOT edge chains under all four header×operator combinations | one model edge per arrow, directed exactly when the operator is `->` and undirected when it is `--`, independently of the header |
| p3 | unit | [[ge.dot_parser.c3]] | attribute lists with quoted, escaped, and bare values | attribute values land unescaped exactly once on the right object |
| p4 | unit | [[ge.dot_parser.c4]] | named, nested, and bare-scope subgraphs | a named subgraph becomes a group with the name verbatim containing only its directly declared nodes (nested nodes belong to the innermost group), and a bare `{ }` scope keeps its nodes ungrouped — though the node preceding the scope is still linked by the scope's inner edge chain, per the oracle's surviving left-edge stack |
| p5 | unit | [[ge.dot_parser.c5]] | record labels with and without ports, HTML-like table labels, and the failing constructs (nameless `subgraph`, malformed HTML-like label, unresolvable port reference) | record and HTML-like table labels split into `name.N` part nodes with port markers stripped and edges reattached to the referenced parts, while the failing constructs each produce the oracle's typed error — tokenizing failures quote the offending input, unresolvable port references name the `base:port` and edge id — and return no partial graph |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** DOT headers of both kinds
- **THEN** a `graph` header yields a model graph with `type: undirected` set and a `digraph` header yields one with `type` unset
- **VERIFIES** [[ge.dot_parser.p1]]

#### Scenario: p2

- **WHEN** DOT edge chains under all four header×operator combinations
- **THEN** one model edge per arrow, directed exactly when the operator is `->` and undirected when it is `--`, independently of the header
- **VERIFIES** [[ge.dot_parser.p2]]

#### Scenario: p3

- **WHEN** attribute lists with quoted, escaped, and bare values
- **THEN** attribute values land unescaped exactly once on the right object
- **VERIFIES** [[ge.dot_parser.p3]]

#### Scenario: p4

- **WHEN** named, nested, and bare-scope subgraphs
- **THEN** a named subgraph becomes a group with the name verbatim containing only its directly declared nodes (nested nodes belong to the innermost group), and a bare `{ }` scope keeps its nodes ungrouped — though the node preceding the scope is still linked by the scope's inner edge chain, per the oracle's surviving left-edge stack
- **VERIFIES** [[ge.dot_parser.p4]]

#### Scenario: p5

- **WHEN** record labels with and without ports, HTML-like table labels, and the failing constructs (nameless `subgraph`, malformed HTML-like label, unresolvable port reference)
- **THEN** record and HTML-like table labels split into `name.N` part nodes with port markers stripped and edges reattached to the referenced parts, while the failing constructs each produce the oracle's typed error — tokenizing failures quote the offending input, unresolvable port references name the `base:port` and edge id — and return no partial graph
- **VERIFIES** [[ge.dot_parser.p5]]

