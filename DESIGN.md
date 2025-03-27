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
        "event": "<ok|failed|ignored>",
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
- Requires every plugin to cooperate
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
- Escapes aren't explicitly communicated which makes communal parsing more difficult
- lexopt builds in specific option-value semantics

And in general we will be putting the parser in the libtest-next's API and it will be a fundamental point of extension.
Having complete control helps ensure the full experience is cohesive.

### Decision: `Short(&str)`

`lexopt` and `clap` / `clap_lex` treat shorts as a `char` which gives a level of type safety to parsing.
However, with a minimal API, providing `&str` provides span information "for free".

If someone were to make an API for pluggable lexers,
support for multi-character shorts is something people may want to opt-in to (it has been requested of clap).

Performance isn't the top priority, so remoing `&str` -> `char` conversions isn't necessarily viewed as a benefit.
This also makes `match` need to work off of `&str` instead of `char`.
Unsure which of those would be slower and how the different characteristics match up.
