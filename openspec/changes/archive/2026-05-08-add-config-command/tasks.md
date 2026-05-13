## 1. CLI plumbing

- [x] 1.1 Add `Config` variant to `Action` enum and `ConfigAction` enum in `src/cli/options.rs`
- [x] 1.2 Define `ConfigTarget` enum: `App` (default), `Global`, `Local`
- [x] 1.3 Create `src/cli/config.rs` module with `handle` function
- [x] 1.4 Wire `Action::Config` dispatch in `src/cli/runner.rs`
- [x] 1.5 Add `--json` flag to config command
- [x] 1.6 Add `--edit` flag (only valid for `global`/`local` targets)

## 2. Core implementation

- [x] 2.1 Implement workspace/config loading for config command (reuse existing `get_workspace_dir` and config loading)
- [x] 2.2 Implement CLI text output: display active features and providers for `app` target
- [x] 2.3 Implement CLI text output: display raw config content for `global`/`local` targets
- [x] 2.4 Implement `--json` output using existing serde serialization for all three targets
- [x] 2.5 Implement `--edit` validation: reject `--edit` on `app`, reject `--edit` in non-TTY
- [x] 2.6 Implement TUI viewer mode (cliclack interactive display of config sections)
- [x] 2.7 Implement TUI editor mode for `global --edit`: multiselect features + provider selection + write to config.toml
- [x] 2.8 Implement TUI editor mode for `local --edit`: same as global but writes to local.config.toml

## 3. Display-friendly config serialization

- [x] 3.1 Add `AppConfig::to_display_json()` method returning a flat `{ features: [...], providers: [...] }` structure
- [x] 3.2 Implement provider detail extraction from `AppConfig` (name, enabled features, per-feature settings)

## 4. Unit tests

- [x] 4.1 Test config target parsing (app/global/local from CLI args)
- [x] 4.2 Test `--edit` rejection on `app` target
- [x] 4.3 Test `--edit` rejection in non-TTY mode
- [x] 4.4 Test `AppConfig::to_display_json()` output format

## 5. E2E tests

- [x] 5.3 Add e2e test: `dotagents config` CLI output in non-TTY
- [x] 5.4 Add e2e test: `dotagents config --json` valid JSON output
- [x] 5.5 Add e2e test: `dotagents config global --json` valid JSON output
- [x] 5.6 Add e2e test: `dotagents config app --edit` errors correctly
- [x] 5.7 Add e2e test: `dotagents config global --edit` in TTY mode (T-CG01)
- [x] 5.8 Add e2e test: `dotagents config` TUI viewer mode (T-CA01)
- [x] 5.9 Add e2e test: `dotagents config global` TUI viewer mode (T-CG02)
- [x] 5.10 Add e2e test: `dotagents config local` TUI viewer mode (T-CL02)
- [x] 5.11 Add e2e test: `dotagents config local --edit` in TTY mode (T-CL01)

## 6. Verification

- [x] 6.1 Run `mise check` and fix any format/lint issues
- [x] 6.2 Run `mise tests` and fix any failures
