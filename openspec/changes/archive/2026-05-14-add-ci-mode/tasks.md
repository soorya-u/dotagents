## 1. CI Mode State

- [x] 1.1 Add a `static CI_MODE: OnceLock<bool>` to `src/utils/tty.rs`
- [x] 1.2 Add a `pub(crate) fn set_ci_mode(enabled: bool)` initializer that calls `CI_MODE.set(enabled)`
- [x] 1.3 Update `is_tty()` to return `false` immediately when `CI_MODE` is set to `true`

## 2. CLI Flag

- [x] 2.1 Add `ci: bool` field to `Options` in `src/cli/options.rs` with `#[clap(long, global = true)]`

## 3. Initialization

- [x] 3.1 In `src/cli/runner.rs::run()`, resolve CI mode: `opts.ci || matches env DOTAGENTS_CI as truthy (true/1/yes, case-insensitive)`
- [x] 3.2 Call `set_ci_mode(ci_mode)` before dispatching to any subcommand

## 4. Fix Direct is_terminal() Call Sites

- [x] 4.1 Replace `std::io::stdin().is_terminal()` in `is_tui_mode()` (`src/cli/init.rs:53`) with `is_tty()`
- [x] 4.2 Replace all direct `std::io::stdin().is_terminal()` / `std::io::stdout().is_terminal()` calls in `src/cli/config.rs` with `is_tty()`

## 5. Unit Tests

- [x] 5.1 Add unit test in `src/utils/tty.rs`: `is_tty_returns_false_when_ci_mode_set` — set CI mode and assert `is_tty()` returns `false`
- [x] 5.2 Add unit test in `src/cli/init.rs`: `is_tui_mode_false_when_ci_mode_set` — set CI mode and assert `is_tui_mode()` returns `false`

## 6. E2E Tests

- [x] 6.1 Add e2e test in `tests/e2e/`: `dotagents --ci deploy` exits 0 and writes files without prompting (use a pre-configured workspace)
- [x] 6.2 Add e2e test: `DOTAGENTS_CI=true dotagents deploy` behaves identically to `--ci`
- [x] 6.3 Add e2e test: `dotagents --ci init` completes without wizard (use a temp workspace, assert `.dotagents*/config.toml` written)

## 7. Verification

- [x] 7.1 Run `mise check` (cargo fmt + clippy) and fix any issues
- [x] 7.2 Run `mise tests` (unit + integration + e2e) and fix any failures
