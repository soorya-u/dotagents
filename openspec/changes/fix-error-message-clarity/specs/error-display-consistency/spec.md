## MODIFIED Requirements

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

## MODIFIED Requirements

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
