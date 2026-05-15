## Why

Manual testing of v0.1.0 (commit `793b8391`) revealed 8 gaps across the init, config, and providers e2e suites. These include untested Clap validation errors for init, missing config file graceful handling, and providers TUI coverage. Two providers cases (`--quiet`, `--verbose`) require implementation changes alongside tests, as the flags are currently non-functional for the `providers` command.

## What Changes

- Add e2e tests to `tests/e2e/init.test.ts` covering:
  - **TC-INIT-07**: `--features none,commands` rejected with exclusive-combination error
  - **TC-INIT-ERR-01**: Invalid `--features` value (e.g., `--features bogus`) exits 2 with valid values listed
  - **TC-INIT-ERR-02**: Invalid `--template` value (e.g., `--template bogus`) exits 2 with valid values listed
- Add e2e tests to `tests/e2e/config.test.ts` covering:
  - **TC-CFG-08**: Missing `local.config.toml` — `config local` exits 0 with "No local config" message, `--json` returns `{}`
  - **TC-CFG-09**: Missing `config.toml` — `config global` exits 1 with "not found" error
- Add e2e tests to `tests/e2e/providers.test.ts` covering:
  - **TC-PROV-01**: Interactive TUI provider list — select widget renders, navigation works, Enter selects, Escape cancels
- Fix `providers` command to respect `--quiet` and `--verbose` flags, then add tests:
  - **TC-PROV-09**: `--quiet` suppresses provider listing output (requires changing `println!()` to log-framework output or gating behind quiet check)
  - **TC-PROV-10**: `--verbose` adds fetch/cache diagnostics to provider listing output (requires adding `debug!()` calls to visible output path)

## Capabilities

### New Capabilities
- `init-validation-e2e`: E2e tests for init flag validation errors (exclusive features, invalid values)
- `config-missing-files-e2e`: E2e tests for graceful handling of missing config files
- `providers-tui-e2e`: E2e test for the interactive provider list TUI widget
- `providers-flag-fixes-e2e`: Implementation fixes and e2e tests for `--quiet` and `--verbose` on the providers command

### Modified Capabilities
- `providers-list`: `--quiet` and `--verbose` flags need implementation changes to become functional for this command

## Impact

- `tests/e2e/init.test.ts` — new test cases for validation errors
- `tests/e2e/config.test.ts` — new test cases for missing config files
- `tests/e2e/providers.test.ts` — new TUI test and flag tests
- `src/cli/providers.rs` — implementation changes for `--quiet`/`--verbose` flag handling (currently uses raw `println!()` which bypasses log framework)
- Reference: `docs/v0.1.0-testing/results/01-init.md`, `docs/v0.1.0-testing/results/06-providers.md`, `docs/v0.1.0-testing/results/07-config.md`
