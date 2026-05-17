## Why

The `init` command has no way to specify provider targets non-interactively (`--targets` flag is missing), and provider display is inconsistent across commands — `init`/`config` show bare slugs while `providers ls` shows rich details. Additionally, the `Feature::None` sentinel was a pre-`--ci` workaround that is now unnecessary and adds complexity.

## What Changes

- Add `--targets` flag to `init` command, following the same `value_delimiter = ','` pattern as `--features`
- **BREAKING**: Remove `Feature::None` variant and `--features none` support — use `--ci` instead for headless no-feature init
- **BREAKING**: Change `is_tui_mode()` from checking individual flag presence to just `is_tui_enabled()` — wizard now runs in TUI and skips only the prompts whose flags were provided
- **BREAKING**: When `--features` is absent in non-TUI mode, default to no features (was: all features)
- Change `init` and `config --edit` provider display from bare slug to `Provider Name [provider-slug]`
- Change `providers ls` TUI to show only `provider-slug` in the select list, with `Provider Name (url)` shown as outro on selection

## Capabilities

### New Capabilities
- `init-targets-flag`: The `--targets` flag for non-interactive provider target selection during init

### Modified Capabilities
- `init-features-flag`: Remove `Feature::None` sentinel; `--features` absence in non-TUI defaults to no features instead of all
- `init-wizard`: Wizard runs whenever TUI is enabled; individual prompts are skipped when their corresponding flag is provided (instead of any flag skipping the entire wizard)
- `ci-mode`: `--ci init` without `--features` now scaffolds no features (was: all features)
- `providers-list`: TUI select shows slug only; name+url shown as outro on selection
- `provider-display-in-init-config`: `init` wizard and `config --edit` show `Provider Name [provider-slug]` format

## Impact

- **Files**: `src/cli/options.rs`, `src/cli/init.rs`, `src/cli/ui/init.rs`, `src/cli/providers.rs`, `src/cli/config.rs`
- **Tests**: Existing tests for `Feature::None`, `validate_features()`, `has_feature()`, and `is_tui_mode()` need updating
- **CI pipelines**: Any pipeline using `dotagents --ci init` without explicit `--features` will get no features instead of all (no current users affected)
