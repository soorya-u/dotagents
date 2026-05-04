## 1. Intro text fix

- [x] 1.1 In `src/cli/ui/init.rs` line 11, change `intro("dotagents · init")` to `intro("dotagents")`

## 2. Silence skip logging

- [x] 2.1 In `src/cli/init.rs` line 190, change `info!("Skipping {}", file.path.display())` to `debug!("Skipping {}", file.path.display())`

## 3. Feature persistence

- [x] 3.1 Add `update_config_features(config_path: &Path, features: &[Feature]) -> Result<()>` to `src/cli/init.rs` — mirrors `update_config_targets`: parse the TOML file, replace the `features` array with the serialised feature strings, write back
- [x] 3.2 Map `Feature::None` to an empty array in `update_config_features`; all other variants map to their string form
- [x] 3.3 In `initialize_agents_dir`, call `update_config_features` for both `config.toml` and `local.config.toml` inside the `if tui_mode` block (alongside the existing `update_config_targets` calls)
- [x] 3.4 Also call `update_config_features` in the non-TUI headless path when `opts.features` is `Some` (outside the `tui_mode` guard)

## 4. Tests

- [x] 4.1 Add unit test `update_config_features_sets_features_array` — writes mock config, calls `update_config_features` with a subset, asserts the written file contains only those features
- [x] 4.2 Add unit test `update_config_features_writes_empty_array_for_none` — asserts `features = []` when `Feature::None` is passed
- [x] 4.3 Add unit test `update_config_features_replaces_existing_features` — asserts previous feature list is fully replaced
- [x] 4.4 Add unit test `update_config_features_errors_on_missing_file`
- [x] 4.5 Run `tui-devtools` against the full init wizard flow and verify: no skip bullets at default verbosity, intro text is `dotagents`, features match selection in generated config
- [x] 4.6 Add e2e test asserting `config.toml` features match the `--features` flag in headless mode
- [x] 4.7 Run `mise check && mise tests` and fix any failures
