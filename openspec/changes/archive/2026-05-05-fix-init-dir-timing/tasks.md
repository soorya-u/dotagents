## 1. Source Fix

- [x] 1.1 In `src/cli/init.rs`, remove `fs::create_dir_all(&workspace)` from line 113 (before the `try_exists()` call)
- [x] 1.2 In `src/cli/init.rs`, add `fs::create_dir_all(&workspace)` after the TUI wizard block (after line 126, before the `if dir_exists` check at line 128)
- [x] 1.3 Run `mise check` and confirm it exits 0 (no fmt or clippy errors)

## 2. Unit / Integration Verification

- [x] 2.1 Run `mise run tests:unit` and confirm existing unit tests still pass
- [x] 2.2 Run `mise run tests:integration` and confirm integration tests (flag-driven paths) still pass

## 3. TUI Discovery

- [x] 3.1 Start `tui-devtools` as a daemon from a mise shell
- [x] 3.2 Drive the init wizard cancellation flow through a real PTY in a fresh temp directory and record exact terminal output (prompt text, symbols, spacing)
- [x] 3.3 Confirm that after cancellation the temp directory contains no new subdirectories

## 4. E2E Tests

- [x] 4.1 In `tests/e2e/init.test.ts`, add a test that runs `dotagents init` in a fresh temp workspace, cancels the wizard (presses Escape), and asserts no directory created
- [x] 4.2 In `tests/e2e/init.test.ts`, confirm happy path T01 test covers successful init with all scaffold files
- [x] 4.3 Run `mise run tests:e2e` and confirm all e2e tests pass

## 5. Final Verification

- [x] 5.1 Run `mise check` — must exit 0
- [x] 5.2 Run `mise tests` — must exit 0
