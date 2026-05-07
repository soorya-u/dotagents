## 1. CLI plumbing

- [ ] 1.1 Add `Config` variant to `Action` enum and `ConfigAction` enum in `src/cli/options.rs`
- [ ] 1.2 Define `ConfigTarget` enum: `App` (default), `Global`, `Local`
- [ ] 1.3 Create `src/cli/config.rs` module with `handle` function
- [ ] 1.4 Wire `Action::Config` dispatch in `src/cli/runner.rs`
- [ ] 1.5 Add `--json` flag to config command
- [ ] 1.6 Add `--edit` flag (only valid for `global`/`local` targets)

## 2. Core implementation

- [ ] 2.1 Implement workspace/config loading for config command (reuse existing `get_workspace_dir` and config loading)
- [ ] 2.2 Implement CLI text output: display active features and providers for `app` target
- [ ] 2.3 Implement CLI text output: display raw config content for `global`/`local` targets
- [ ] 2.4 Implement `--json` output using existing serde serialization for all three targets
- [ ] 2.5 Implement `--edit` validation: reject `--edit` on `app`, reject `--edit` in non-TTY
- [ ] 2.6 Implement TUI viewer mode (cliclack interactive display of config sections)
- [ ] 2.7 Implement TUI editor mode for `global --edit`: multiselect features + provider selection + write to config.toml
- [ ] 2.8 Implement TUI editor mode for `local --edit`: same as global but writes to local.config.toml

## 3. Display-friendly config serialization

- [ ] 3.1 Add `AppConfig::to_display_json()` method returning a flat `{ features: [...], providers: [...] }` structure
- [ ] 3.2 Implement provider detail extraction from `AppConfig` (name, enabled features, per-feature settings)

## 4. Unit tests

- [ ] 4.1 Test config target parsing (app/global/local from CLI args)
- [ ] 4.2 Test `--edit` rejection on `app` target
- [ ] 4.3 Test `--edit` rejection in non-TTY mode
- [ ] 4.4 Test `AppConfig::to_display_json()` output format

## 5. E2E tests

- [ ] 5.1 tui-devtools discovery pass for `config` TUI viewer flow
- [ ] 5.2 tui-devtools discovery pass for `config global --edit` TUI editor flow
- [ ] 5.3 Add e2e test: `dotagents config` CLI output in non-TTY
- [ ] 5.4 Add e2e test: `dotagents config --json` valid JSON output
- [ ] 5.5 Add e2e test: `dotagents config global --json` valid JSON output
- [ ] 5.6 Add e2e test: `dotagents config app --edit` errors correctly
- [ ] 5.7 Add e2e test: `dotagents config global --edit` in TTY mode
- [ ] 5.8 Add e2e test: `dotagents config` TUI viewer mode

## 6. Verification

- [ ] 6.1 Run `mise check` and fix any format/lint issues
- [ ] 6.2 Run `mise tests` and fix any failures
