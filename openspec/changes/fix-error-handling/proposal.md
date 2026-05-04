## Why

`deploy` panics with a raw Rust backtrace when initialization fails (e.g. no workspace directory), while `undeploy` surfaces the same failure as a formatted error box via `display_error()`. This inconsistency exposes internal implementation details to users and bypasses the CLI's established error-presentation contract.

## What Changes

- `Templater::new()` is changed to return `Result<Templater>` instead of panicking on initialization failure.
- `get_templater()` is changed from returning `&'static Templater` to returning `Result<&'static Templater>`, backed by `OnceLock<Templater>` instead of `LazyLock<Templater>`.
- The `.expect("failed to load global variables")` call inside the `LazyLock` initializer is removed; errors now propagate via `?`.
- All call sites of `get_templater()` are updated to handle the `Result` (add `?` or `.context(...)`).
- An e2e test is added asserting that running `deploy` with no workspace directory exits with code 1 and shows a formatted error box on stderr (not raw panic text).

## Capabilities

### New Capabilities

- `error-display-consistency`: Deploy and undeploy must both surface initialization and runtime errors through `display_error()`, producing a consistent formatted error box rather than a raw panic.

### Modified Capabilities

<!-- No existing spec-level behavior is changing — this is a bug fix bringing deploy into conformance with the CLI's existing error-presentation behavior. -->

## Impact

- `src/templates/templater.rs` — primary change site; `LazyLock` replaced with `OnceLock`, `Templater::new` made fallible.
- All callers of `get_templater()` in `src/` — must be found with grep and updated to propagate the `Result`.
- `tests/e2e/` — new test for formatted error output on deploy failure.
- No new dependencies. No public API changes. No breaking changes for end users (error output improves).
