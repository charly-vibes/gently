# Graph-Easy upstream architecture survey

Grounding for the `specs/ge-*` Specodelic corpus. Sources: upstream README,
`lib/Graph/Easy.pm`, `Parser.pm`, `As_txt.pm`, `Layout.pm` (ironcamel/Graph-Easy @ master).

## What Graph::Easy does

Build graphs (nodes/edges, directed or not, plus groups), lay them out on a
flat surface, and render as ASCII art, HTML, boxart, or Graphviz DOT. Graphs
come from Perl API calls, the own text format, or Graphviz DOT input.

## Input: Graph::Easy text format (`Graph::Easy::Parser`)

- Nodes in `[brackets]`; `[ ]` creates an anonymous node (not referencable later).
- Edge operators: `->` solid, `=>` double, `.>` dotted, `~>` wave; plus
  `- >` dashed, `.->` dot-dash, `..->` dot-dot-dash, `= >` double-dash.
  Repeated patterns must repeat the full unit (`..-..-..->` valid;
  `..-..-..>` invalid). Arrow-less edges (`---`, `.-.-`) need ≥2 repetitions;
  single-char shortcuts (`-`, `~`) are invalid.
- Directions: `-->` (arrow at target), `<-->` (both), `--` (none).
- Inline labels: `-- foo -->` (both flanks must match; arrow-less edges
  cannot take inline labels).
- `{ attr: value; }` blocks attach to nearest preceding object; class
  sections `graph/node/edge/group { ... }` apply to whole classes.
- Groups: `( Name: ... )` blocks and anonymous `( [ A ] )`; group stack
  supports nesting. Comments start with `#`.
- Chaining: `A -> B -> C` makes one edge per adjacent pair.

## Output

- `as_ascii` (As_ascii.pm): box-drawing ASCII art, the signature output.
- `as_boxart` (As_boxart.pm, separate dist upstream? in-repo As_boxart exists): Unicode.
- `as_html` / `as_html_file`: HTML table + CSS (default output format is html in the API, script default ascii).
- `as_graphviz`: DOT output; dot/neato do the layout.
- `as_txt` (As_txt.pm): canonical text — emits class attribute sections
  (sorted) then nodes/edges with attributes. **This is the form used for
  expected results in the upstream test corpus** => golden-test lingua
  franca for the port.

## Layout (`Graph::Easy::Layout` + submodules)

Staged: rank assignment (DAG-ish ranking by flow), ordering within ranks
(crossing minimization), positioning, orthogonal edge path routing (ports,
edge labels on paths). Flow: down/up/left/right. Output is a cell grid.

## Port strategy implied for gently

- Model first, then parsers, layout, renderers, CLI last.
- Differential/golden testing against upstream `t/` fixtures via txt+ascii.
- SVG was a separate upstream dist — out of scope for v1 (extension_point later).

