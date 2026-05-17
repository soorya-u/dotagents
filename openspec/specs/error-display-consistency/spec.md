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

### Requirement: Error chain uses single "Failed to" prefix and readable inner causes
The `display_error()` function SHALL prepend `"Failed to "` exactly once (for the outermost message). All `.context()` strings owned by `dotagents` code SHALL use `"unable to X"` phrasing for inner causes and a bare verb phrase (e.g., `"complete 'skills add' command"`) for the outermost wrapper added in `runner.rs`. No `.context()` string SHALL start with `"Failed to"`.

#### Scenario: Single outermost prefix
- **WHEN** a command fails with error chain `[bail("No .dotagents directory found"), context("unable to resolve workspace directory"), context("complete 'skills add' command")]`
- **THEN** stderr contains `"Failed to complete 'skills add' command"`
- **THEN** stderr contains `"unable to resolve workspace directory"` under `"Caused by:"`
- **THEN** stderr does NOT contain `"Failed to Failed to"`

#### Scenario: Inner context strings use "unable to" prefix
- **WHEN** any internal function adds a `.context()` to an error
- **THEN** that context string starts with `"unable to"` (not `"Failed to"`)

#### Scenario: Root-cause bail messages appear unchanged
- **WHEN** a `bail!("Configuration already exists")` is the root cause
- **THEN** that message appears in the `"Caused by:"` section unchanged, without any prefix added

### Requirement: runner.rs surfaces full subcommand dispatch
`runner.rs::run()` SHALL contain the full dispatch tree for all commands and subcommands. Helper functions `run_skills()` and `run_commands()` SHALL be removed. Each dispatch arm SHALL wrap its call with a `.context("complete '<subcommand>' command")` string.

#### Scenario: skills subcommand errors include subcommand context
- **WHEN** `skills add` fails with any internal error
- **THEN** the outermost context is `"complete 'skills add' command"`
- **THEN** `display_error` prints `"Failed to complete 'skills add' command"`

#### Scenario: init/deploy/undeploy errors include action context
- **WHEN** `deploy` fails (e.g., missing workspace)
- **THEN** the outermost context from `runner.rs` is `"complete 'deploy' command"`
- **THEN** `display_error` prints `"Failed to complete 'deploy' command"`
