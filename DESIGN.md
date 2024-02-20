# Design decisions

## json format

Goals:
- Allow a runner, like cargo-test, to take over UX concerns, making the experience richer and removing burdens from custom test harness writers
- Allow a runner, like cargo-test, to run binaries in parallel
- Evolve with users to handle their custom test harnesses

Care abouts
- Minimize the burden on custom test harness authors
- Recognize we can't see the future and allow adaptation

See also [eRFC 3558](https://github.com/rust-lang/rfcs/pull/3558)

### Prior Art

[libtest's existing format](https://github.com/rust-lang/rust/blob/master/library/test/src/formatters/json.rs) (as ndjson):
```json
[
    {
        "type": "suite",
        "event": "discovery"
    },
    {
        "type": "<test|bench>",
        "event": "discovered",
        "name": "",
        "ignore": false,
        "ignore_message": "",
        "source_path": "",
        "start_line": 0,
        "start_col": 0,
        "end_line": 0,
        "end_col": 0
    },
    {
        "type": "suite",
        "event": "completed",
        "tests": 0,
        "benches": 0,
        "total": 0,
        "ignored": 0
    },
    {
        "type": "suite",
        "event": "started",
        "test_count": 0,
        "shuffle_seed": 0  # or not-present (unstable)
    },
    {
        "type": "test",
        "event": "started",
        "name": "",
    },
    {
        "type": "test",
        "event": "<ok|failed|igored>",
        "name": "",
        "exec_time": 0.0,  # or not-present (unstable)
        "stdout": "",  # or not-present
        "message": "", # present only for `failed`, `ignored`
        "reason": "time limit exceeded",  # present only for `failed`
    },
    {
        "type": "bench",  # (unstable)
        "name": "",
        "median": 0,
        "deviation": 0,
        "mib_per_second": 0  # or not-present
    },
    {
        "type": "test",
        "event": "timeout",
        "name": ""
    },
    {
        "type": "suite",
        "event": "<ok|failed>",
        "passed": 0,
        "failed": 0,
        "ignored": 0,
        "measured": 0,
        "filtered_out": 0,
        "exec_time": 0  # (unstable)
    }
]
```
- The event type is split between `event` and `type`
  - This becomes even more complicated when `event` is also used to convey "status"
- Ambiguous when multiple streams of these get merged, like if we had `cargo test --message-format=json` support for this
- Carries presentation-layer concerns like count
- Line/column is a presentation-layer way of tracking location within a file (vs byte offsets)
- Does not support runtime ignoring (can't report a test is ignored outside of discovery)
- Does not directly support runtime test case (`name` is assumed to be same between discovery and running)
- bench (unstable)
  - Doesn't even have `event`
  - No "started" event
  - `mib_per_second` is too application-specific
  - Does not convey units
  - No extension point for special reporters

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
