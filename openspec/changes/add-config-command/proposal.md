## Why

Dotagents uses a layered config system (global `config.toml`, local `local.config.toml`, merged `AppConfig`) but users have no way to inspect their resolved runtime configuration. After init and any edits, users are left guessing which features are active, which providers are targeted, and how local overrides have merged with global settings. A `config` command closes this visibility gap.

## What Changes

- Add a `dotagents config` subcommand (defaulting to `app` — the merged runtime config)
- Support `dotagents config app|global|local` to inspect each configuration layer individually
- CLI mode: display active features and targeted providers with their settings
- `--json` flag outputs the selected config as structured JSON
- TUI mode: interactive viewer for `app`/`global`/`local` configs with detail navigation
- `--edit` flag (only on `global` or `local`): opens an interactive TUI editor to add, remove, or modify providers and features; edits are persisted to the respective config file
- `--edit` on `app` is rejected with an error (app is derived, not directly editable)

## Capabilities

### New Capabilities
- `config-inspect`: viewing and interactively editing the workspace configuration (global, local, and merged app config)

### Modified Capabilities
<!-- None -->

## Impact

- `src/cli/config.rs` — new module implementing the `config` subcommand and TUI editor
- `src/cli/options.rs` — new `Action::Config` variant with `ConfigAction` enum
- `src/schema/config/` — may need minor additions for display-friendly config serialization
- `tests/e2e/` — new e2e tests for CLI and TUI paths
