---
id: ge.dot_parser
kind: intent
statement: "WHERE the dot-parser feature is included THE dot-parser SHALL map a Graphviz DOT graph onto the graph model, covering node statements, edge statements with attribute lists, and subgraph clusters."
---

# dot-parser

An optional feature that reads Graphviz DOT (the input side
Graph::Easy provides via `Graph::Easy::Parser::Graphviz`) and feeds
[[ge.graph_model]]. Scope: the common DOT subset — digraph/graph
headers, node and edge statements, attribute lists, and subgraph
clusters; advanced DOT (records, HTML labels, ports) is out of scope.

## Purpose

An optional feature that reads Graphviz DOT (the input side
Graph::Easy provides via `Graph::Easy::Parser::Graphviz`) and feeds
[[ge.graph_model]]. Scope: the common DOT subset — digraph/graph
headers, node and edge statements, attribute lists, and subgraph
clusters; advanced DOT (records, HTML labels, ports) is out of scope.

## Constraints

| id | kind | expr | traces_to | observes |
|----|------|------|-----------|----------|
| c1 | invariant | A `digraph` header maps to a directed model graph and a `graph` header maps to an undirected one. | [[ge.dot_parser]] | |
| c2 | invariant | An edge chain `a -> b -> c;` creates one directed edge per arrow, and `a -- b;` in a plain graph creates an undirected edge. | [[ge.dot_parser]] | |
| c3 | invariant | An attribute list `[key=val, key2=val2]` attached to a node or edge maps onto the model attributes of that object, with quoted values unescaped once. | [[ge.dot_parser]] | [[ge.dot_parser.c5]] |
| c4 | invariant | A subgraph block named `cluster_*` maps to a model group with that name containing the nodes declared inside it. | [[ge.dot_parser]] | |
| c5 | effect | An unsupported DOT construct produces a typed error naming the construct and its source position. | [[ge.dot_parser]] | |

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
| p1 | unit | [[ge.dot_parser.c1]] | DOT headers of both kinds | header kind matches model direction exactly |
| p2 | unit | [[ge.dot_parser.c2]] | DOT edge chains and single edge statements | one model edge per arrow with correct endpoints and direction |
| p3 | unit | [[ge.dot_parser.c3]] | attribute lists with quoted, escaped, and bare values | attribute values land unescaped exactly once on the right object |
| p4 | unit | [[ge.dot_parser.c4]] | nested subgraphs with and without cluster names | named clusters become groups; anonymous subgraphs keep their nodes ungrouped |
| p5 | unit | [[ge.dot_parser.c5]] | DOT snippets using out-of-scope constructs (records, HTML labels, ports) | each construct is reported with its name and position, and no partial graph is returned |

## Requirements

### Requirement: Property coverage mirror

Every property row SHALL be verified by exactly one dedicated scenario;
`ah sync` SHALL derive one contract per VERIFIES link.

#### Scenario: p1

- **WHEN** DOT headers of both kinds
- **THEN** header kind matches model direction exactly
- **VERIFIES** [[ge.dot_parser.p1]]

#### Scenario: p2

- **WHEN** DOT edge chains and single edge statements
- **THEN** one model edge per arrow with correct endpoints and direction
- **VERIFIES** [[ge.dot_parser.p2]]

#### Scenario: p3

- **WHEN** attribute lists with quoted, escaped, and bare values
- **THEN** attribute values land unescaped exactly once on the right object
- **VERIFIES** [[ge.dot_parser.p3]]

#### Scenario: p4

- **WHEN** nested subgraphs with and without cluster names
- **THEN** named clusters become groups; anonymous subgraphs keep their nodes ungrouped
- **VERIFIES** [[ge.dot_parser.p4]]

#### Scenario: p5

- **WHEN** DOT snippets using out-of-scope constructs (records, HTML labels, ports)
- **THEN** each construct is reported with its name and position, and no partial graph is returned
- **VERIFIES** [[ge.dot_parser.p5]]

