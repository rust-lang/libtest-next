# Design decisions

## lexarg

Goal: provide an API-stable CLI parser for inclusion in APIs for plugin-specific CLI args

### Decision: level of abstraction

Potential design directions
- High-level argument definitions that get aggregated
  - e.g. like https://crates.io/crates/gflags
- Low-level, cooperative parsing
  - e.g. like https://crates.io/crates/lexopt

`lexopt`-like API was selected as it was assumed to have the most potential for
meeting future needs because parsing control is handed to the plugin.

This comes at the cost of:
- Requires everyone plugin to cooperate
- More manual help construction

### Decision: iteraton model

Potential design directions
- `lexopt` exposes a single iterator type that walks over both longs and shorts.
- `clap_lex` exposes an iterator type that walks over each argument with an inner iterator when walking over short flags

`lexopt`-like API was selected.  While `clap_lex` is the more powerful API,
this makes delegating to plugins in a cooperative way more challenging.

### Decision: reuse lexopt vs build something new

In reviewing lexopt's API:
- Error handling is included in the API in a way that might make evolution difficult

TODO: there were other points that felt off to me about lexopt's API wrt API stability but I do not recall what they are
