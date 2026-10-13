# gently-cbk GREEN — the shape:edge oracle rules (probed, pinned)

Probed against the pinned oracle (Graph::Easy v0.69 @ ededa3d7,
PERL_HASH_SEED=0) for tests/repro/claims/bg-edge-node.observed. GREEN
landed in 05d5ddb; this note records the cell-level truth the render was
built from, so a future shape:edge fixture starts here instead of
re-probing.

## The cell grid after `_prepare_layout('ascii')`

Grid cols: c0=0(w8) c1=8(w9) c2=17(w9) c3=26(w5) c4=31(w7); rows r0..r4,
every band 3 lines. Column c2 is 9 wide because train (5-char label)
outranks car (3) in the same column — the edge cell renders FULL column
width, not its own box width.

## The shape:edge node cell

`Node::as_ascii` builds a FRESH `Graph::Easy::Edge->new()` and renders an
`Edge::Cell` with type `EDGE_HOR | EDGE_LABEL_CELL`, w/h = the node's,
label = the node's. `_draw_hor`/`_insert_label` then give:

- solid dash line `'-' x w` across the FULL cell width on fb row `h-2`
  (the band's middle line), no borders/sides;
- label bottom-aligned in the `h-2` field at byte offset 1, `w-2` wide,
  LEFT-aligned — the fresh edge's `align` attribute defaults `left`
  (Attributes.pm: `{ default => 'center', group => 'left', edge => 'left' }`);
  the NODE's align (center) is never consulted.

## The endpoint flag stripping (Scout.pm, two DIFFERENT rules)

Short cells (single gap cell between adjacent nodes, `_find_path`'s
straight branch) carry whole flags from the SHORT_* type:

- `EDGE_SHORT_E = HOR|END_E|START_W`, `SHORT_W = HOR|END_W|START_E`,
  `SHORT_N/S` the vertical pair, `SHORT_UN_EW = HOR|START_W|START_E`.
- target shape:edge: strips `START_E` when dx>0, `START_W` when dx<0,
  NOTHING when dx=0 — i.e. directed runs NEVER lose their arrowhead
  (rent's `>` rides into the car edge cell), undirected runs lose one
  START (Bonn→train keeps `START_W` only → 8 dashes at cell-x 1 of 9).
- source shape:edge: strips `EDGE_START_MASK` wholesale (train→Ulm and
  car→Hof lose `START_W` → `--->` at cell-x 0, flush off the border).

Multi-cell paths (`_find_path_astar`, line ~913) zero the flag ARRAYS
instead: target shape:edge → end_flags all 0 (no END flag on the last
cell — Ulm→car's horizontal has no head and no connection glyph at all;
the merged full-width dash cell just ENDS at car's west edge, x17, with
no stub and no corner into car's cell); source shape:edge → start_flags
all 0.

## The VER short cell (buy)

`SHORT_N = VER|END_N|START_S` — the target strip is a no-op (dx=0), so
the `^` SURVIVES into train's south side. `_draw_ver` ignores START flags
entirely ("starting points are currently invisible"). `_printfb_ver`
prints the line top-down from fb row y (chop-from-end lands the LAST char
at the HIGHEST row — the line reads in order). Label via
`_insert_label(xs=4, ys=1, ws=4, hs=2, 'middle')` → left-aligned at byte
offset 4 on the gap band's middle line (`| buy`).

## The west-going multi-band A* shape

Ulm(4,2) → car(2,0) routes [4,1],[3,1],[2,1]: it bends out of Ulm's
NORTH first (corner S_W at [4,1]), runs west along the gap row adjacent
to the target band ([3,1] + [2,1] MERGE into one 14-wide multicelled
cell — cols 2+3 — rendering one full-width dash run; [3,1] renders as
`Edge::Cell::Empty` = ''), and the horizontal simply stops at the target
column's west edge because the end flags were stripped. gently's old
west-going route walked the source band first and drove THROUGH train's
node cell — the source of the original orthogonality error and the
garbage render.

## Trailing floor

`as_ascii` trims ALL trailing empty lines (`s/\n+\z/\n/`), so the anon
band (rows 12-14, all blank) renders nothing. NOTE: the port's floor rule
diverges from the oracle on two PRE-EXISTING pins — border:none keeps
`\n x\n\n` (oracle: `\n x\n`) and shape:invisible keeps `\n\n\n` (oracle:
`\n`). gently-cbk only excluded anon nodes from the floor (p6/border pins
stay green); the visible-node trailing-blank divergence predates this
ticket and stays as pinned.

## Probe scripts

`/var/tmp/probe_edge*.pl`, `probe_none.pl`, `probe_inv.pl` (scratch, not
committed) dump `$g->{cells}`, `_prepare_layout` rows/cols, per-cell
`as_ascii`, and the edge_type decode. The oy2 lesson holds: trace by
patching/reading the reopened `package Graph::Easy::Edge::Cell` inside
As_ascii.pm (`_draw_hor`/`_draw_ver`/`_insert_label`), and
`Edge::as_ascii` (Edge.pm:483) dispatches through the cell's overridden
`_draw_label`.
