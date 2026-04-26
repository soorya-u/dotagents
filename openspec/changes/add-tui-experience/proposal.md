## Why

`dotagents init` and `dotagents deploy` currently offer no interactive feedback — init silently writes a fixed set of files, the gitignore prompt is a raw `[y/N]` text line, and the `--offline` deploy flag introduced in PR #38 is flag-only with no interactive path. This creates a poor first-run experience and makes it hard for new users to understand what they're setting up or discover available providers.

## What Changes

- **Add `cliclack`** as a dependency for styled terminal prompts (intro/outro framing, multiselect, select, spinner, log steps).
- **Init wizard**: replace silent file writes with an interactive prompt sequence — multiselect for features, select for template choice, confirm for overwrite — with per-file `log::step` feedback.
- **Two init templates**: introduce a "Starter" template (core files only) and a "With Custom Provider" template (adds `templates/mycode/` and provider block in `local.config.toml`).
- **New `--template` flag** on `init` (`starter` | `with-custom-provider`) for non-interactive use.
- **Init target selection**: at the end of the init wizard, fetch `registry.json` (using `Registry::fetch` from PR #38) and present a multiselect of available providers — selected providers are written as `targets = [...]` in `config.toml` (global only, not `local.config.toml`). Registry fetch failure is a soft warning; step is skipped gracefully.
- **Deploy offline prompt**: in TUI mode, before deploy begins, ask "Run in offline mode?" (Yes / No, default No) — maps to the `--offline` flag from PR #38. Flag-based bypass (`--offline`) skips the prompt.
- **Deploy gitignore prompt**: replace raw crossterm keypress code in `src/utils/gitignore.rs` with a cliclack `select` (Yes / No).
- **New `src/cli/ui/` module**: `init.rs` and `deploy.rs` contain all TUI logic; raw crossterm prompt removed from `src/utils/gitignore.rs`.
- **Dual-mode decision logic**: any flag present or non-TTY → flag/silent mode; no flags + TTY → TUI mode.
- **New mock file**: `src/mocks/local.config.starter.toml` (minimal, no providers block).

## Capabilities

### New Capabilities

- `init-wizard`: Interactive cliclack prompt sequence during `dotagents init` — feature multiselect, template select, overwrite confirm, registry-backed target selection, per-file feedback.
- `init-templates`: Two scaffolding templates selectable during init — "Starter" (core files) and "With Custom Provider" (adds mycode example provider + templates directory).
- `deploy-gitignore-prompt`: Upgraded gitignore confirmation prompt using cliclack select (Yes / No radio) instead of raw keypress.
- `deploy-offline-prompt`: Interactive offline-mode selection at the start of deploy in TUI mode, surfacing the `--offline` flag from PR #38 to interactive users.

### Modified Capabilities

- `deploy-gitignore-update`: The prompt mechanism changes from raw crossterm to cliclack; flag-based bypass (`--gitignore` / `--no-gitignore`) is unchanged.

## Impact

- **`Cargo.toml`**: adds `cliclack` dependency; verify crossterm version alignment.
- **`src/cli/options.rs`**: `InitOptions` gains `template: Option<InitTemplate>` enum field and `--template` clap arg. `DeployOptions.offline` (from PR #38) is the flag bypass for the new offline prompt.
- **`src/cli/init.rs`**: branches on flag vs TTY mode; delegates TUI prompts to `src/cli/ui/init.rs`; after file writes, calls `ui::init::prompt_targets` to fetch registry and update `config.toml`.
- **`src/cli/deploy.rs`**: in TUI mode, prompts for offline mode before the registry-fetch block added by PR #38; sets `opts.offline` from prompt result if not already set by flag.
- **`src/utils/gitignore.rs`**: `prompt_gitignore_update()` removed; replaced by call to `src/cli/ui/deploy.rs`.
- **`src/cli/ui/`**: new module (`mod.rs`, `init.rs`, `deploy.rs`).
- **`src/schema/registry.rs`** and **`src/templates/registry_resolver.rs`** (from PR #38): reused in `ui/init.rs` to fetch and enumerate provider names for the target selection step.
- **`src/constants/mocks.rs`**: adds `LOCAL_CONFIG_STARTER` constant.
- **`src/mocks/local.config.starter.toml`**: new file.
