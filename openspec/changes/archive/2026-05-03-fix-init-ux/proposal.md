## Why

`dotagents init` has three UX defects that surface on every run: the wizard intro repeats the command the user just typed, skipped-file noise floods the output when features are deselected, and the features the user selects in the wizard are never actually written to `config.toml` — the file always comes out with all four features hardcoded regardless of input.

## What Changes

- Replace `intro("dotagents · init")` with a short descriptive phrase that does not mirror the typed command.
- Demote `info!("Skipping {}", file.path.display())` to `debug!` so skip messages only appear with `-v`; no change to skip logic itself.
- Add `update_config_features(config_path, features)` — mirrors the existing `update_config_targets` — and call it from `initialize_agents_dir` after the wizard completes, writing the selected feature list to both `config.toml` and `local.config.toml`.
- When `--features none` is used, write `features = []`.
- The `variables` key remains hardcoded in the generated config (not user-configurable at init time).

## Capabilities

### New Capabilities

*(none — all changes are corrections to existing init behaviour)*

### Modified Capabilities

- `init-wizard`: Init wizard now correctly persists user's feature selection into both config files; skip messages are no longer visible at default verbosity; intro text no longer repeats the command name.

## Impact

- `src/cli/ui/init.rs` — change `intro()` text (line 11).
- `src/cli/init.rs` — demote `info!("Skipping …")` to `debug!` (line 190); add `update_config_features()` function; call it alongside `update_config_targets` in the `tui_mode` block and in the non-TUI headless path when `opts.features` is `Some`.
- Tests: update unit tests that assert on written config content; add unit tests for `update_config_features`.
