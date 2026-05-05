## Why

`dotagents init` creates the workspace directory on disk before the TUI wizard runs, so cancelling the wizard leaves a spurious directory behind. Directory creation should be deferred until the user completes the wizard and confirms they want to proceed.

## What Changes

- Move `fs::create_dir_all(&workspace)` in `src/cli/init.rs` from before the `try_exists()` check to after the TUI wizard block, so no filesystem writes occur if the user cancels.
- Add an e2e test that asserts cancelling the init wizard leaves no directory on disk.

## Capabilities

### New Capabilities

- `init-dir-timing`: Behavioral invariant that `dotagents init` produces no filesystem side-effects when the user cancels the TUI wizard.

### Modified Capabilities

- `init-wizard`: The requirement that no filesystem writes occur on wizard cancellation is a spec-level behavioral change for the existing init wizard capability.

## Impact

- `src/cli/init.rs` — one-line move of `fs::create_dir_all` call
- `tests/e2e/init.test.ts` — new test case covering cancellation with no leftover directory
- No API changes, no new dependencies, no breaking changes
