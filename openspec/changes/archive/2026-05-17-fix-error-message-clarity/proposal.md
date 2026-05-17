## Why

Error output is confusing in two ways: (1) `display_error()` prepends `"Failed to "` unconditionally, but many `.context()` strings also start with `"Failed to"` — producing `"Failed to Failed to resolve workspace directory"`; (2) inner `"Caused by:"` entries use bare verb phrases like `"resolve workspace directory"` which read as nonsense without a prefix. Separately, subcommand dispatch is buried inside `run_skills()` / `run_commands()` helper functions, making the CLI command tree invisible in `runner.rs`.

## What Changes

- `display_error()` in `src/utils/error.rs` keeps its `"Failed to "` prefix for the first message only (no change needed — it already does this); outermost context strings in `runner.rs` are changed to verb-only phrases (`"complete 'skills add' command"`) so `display_error` prepends exactly once
- All inner `.context()` strings throughout the codebase are changed from `"Failed to X"` to `"unable to X"` so `"Caused by: unable to resolve workspace directory"` reads naturally
- `run_skills()` and `run_commands()` dispatch helpers are removed; their match arms are inlined into `runner.rs::run()` with per-subcommand `.context()` strings
- `init`, `deploy`, and `undeploy` actions in `runner.rs` gain `.context()` wrappers so `bail!()` root causes are never the first message seen by `display_error`
- Unit tests in `src/utils/error.rs` verify the double-prefix regression does not recur

## Capabilities

### New Capabilities

### Modified Capabilities
- `error-display-consistency`: Error chain formatting now uses `"unable to X"` for inner causes and single `"Failed to "` prefix from `display_error` for the outermost message

## Impact

- `src/utils/error.rs` — unit tests for error chain formatting
- `src/cli/runner.rs` — subcommand dispatch inlined; all actions gain `.context()`
- `src/cli/skills.rs` — `run_skills()` removed; inner contexts changed to `"unable to X"`
- `src/cli/commands.rs` — `run_commands()` removed; inner contexts changed to `"unable to X"`
- `src/cli/init.rs`, `src/cli/deploy.rs`, `src/cli/undeploy.rs` — inner contexts changed to `"unable to X"`
- All other files using `.context("Failed to …")` — search-and-replace to `"unable to …"`
