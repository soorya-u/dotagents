## 1. Audit context strings

- [x] 1.1 Run `grep -rn 'context("Failed to' src/` to list all context strings starting with "Failed to"
- [x] 1.2 Replace all inner `.context("Failed to X")` strings with `.context("unable to X")` across the entire `src/` tree

## 2. Fix runner.rs dispatch

- [x] 2.1 In `src/cli/runner.rs`, inline the `Action::Skills` match arm to dispatch `SkillsAction` variants directly, with per-subcommand `.context("complete 'skills X' command")`
- [x] 2.2 In `src/cli/runner.rs`, inline the `Action::Commands` match arm to dispatch `CommandsAction` variants directly, with per-subcommand `.context("complete 'commands X' command")`
- [x] 2.3 Remove `run_skills()` from `src/cli/skills.rs` and `run_commands()` from `src/cli/commands.rs`
- [x] 2.4 Add `.context("complete 'init' command")` to the `Action::Init` arm in `runner.rs`
- [x] 2.5 Add `.context("complete 'deploy' command")` to the `Action::Deploy` arm in `runner.rs`
- [x] 2.6 Add `.context("complete 'undeploy' command")` to the `Action::Undeploy` arm in `runner.rs`
- [x] 2.7 Add `.context("complete 'config' command")` to the `Action::Config` arm in `runner.rs`

## 3. Unit tests

- [x] 3.1 Add unit test in `src/utils/error.rs`: single-message error chain produces `"Failed to <msg>"` with no double prefix
- [x] 3.2 Add unit test in `src/utils/error.rs`: two-level error chain produces `"Failed to <outer>"` and `"Caused by:\n    <inner>"`
- [x] 3.3 Update any existing unit/e2e tests that assert on exact error message text to use the new `"unable to"` / `"complete '…' command"` phrasing

## 4. Verification

- [x] 4.1 Run `mise check` (fmt + clippy) — must exit 0
- [x] 4.2 Run `mise tests` (unit + integration + e2e) — must exit 0
