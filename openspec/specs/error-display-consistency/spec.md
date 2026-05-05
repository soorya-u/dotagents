# Error Display Consistency

## Purpose

Ensure all CLI subcommands surface initialization and runtime errors through a consistent `display_error()` path, producing a formatted error box on stderr instead of raw Rust panic text.

## Requirements

### Requirement: Deploy surfaces errors through display_error
When the `deploy` subcommand fails during initialization (e.g., no workspace directory, failed variable loading), it SHALL exit with code 1 and print a formatted error box to stderr via `display_error()`. It MUST NOT produce a raw Rust panic message.

#### Scenario: Deploy with no workspace directory prints formatted error
- **WHEN** the user runs `dotagents deploy` from a directory that has no `.dotagents` ancestor
- **THEN** the process exits with code 1
- **THEN** stderr contains a formatted error box (e.g., text matching `■` or `Fatal error`)
- **THEN** stderr does NOT contain the text "panicked at"

#### Scenario: Deploy error output matches undeploy error output format
- **WHEN** both `deploy` and `undeploy` fail due to a missing workspace directory
- **THEN** both commands exit with code 1
- **THEN** both commands produce error output using the same `display_error()` formatted box structure
- **THEN** neither command emits raw Rust panic text

### Requirement: get_templater returns Result
The `get_templater()` function SHALL return `Result<&'static Templater>` so that callers can propagate initialization failures with the `?` operator into their own `Result`-returning functions, allowing errors to reach `main.rs` and flow through `display_error()`.

#### Scenario: Templater initialization failure propagates to caller
- **WHEN** `Templater::new()` returns an `Err` (e.g., workspace directory cannot be resolved)
- **THEN** `get_templater()` returns that same `Err`
- **THEN** the calling function propagates it with `?`
- **THEN** `main.rs` receives the error and calls `display_error()` before exiting with code 1

#### Scenario: Templater initialization success behaves as before
- **WHEN** `Templater::new()` succeeds (workspace directory exists, variables load correctly)
- **THEN** `get_templater()` returns `Ok(&'static Templater)`
- **THEN** callers receive the same usable `Templater` reference as before
