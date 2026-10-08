# gently

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)

A Rust port of Perl's [Graph::Easy](https://github.com/ironcamel/Graph-Easy):
build graphs (nodes/edges/groups), lay them out deterministically on a
flat surface, and render them as ASCII art, boxart, HTML, or Graphviz
DOT — plus parse graphs from the Graph::Easy text format and from DOT.

**Status: specification phase.** The capability corpus lives in
`specs/` (Specodelic four-layer format); no implementation yet.

```bash
specodelic lint          # all specs must be lint-clean
specodelic compile specs # regenerate artifacts under specodelic/
specodelic model-check specs
```
