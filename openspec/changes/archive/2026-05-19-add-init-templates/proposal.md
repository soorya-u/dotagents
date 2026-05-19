## Why

`dotagents init` currently offers only two templates: `Starter` (full scaffolding with `.env`, `local.config.toml`, variables) and `WithCustomProvider` (adds a `mycode` example provider). Users who want a minimal starting point — just `config.toml`, feature files, and `.gitignore` — have no option and must manually delete files after init. The `Starter` name also doesn't communicate what it provides (variables, env, rendering-ready). This change adds a `Blank` template, renames the existing `WithCustomProvider` to `Advanced`, and makes `Blank` the new default.

## What Changes

- **Add `Blank` template** — writes `config.toml` (reflects `--features`/`--targets`), feature mock files, `.gitignore`. Skips `.env`, `local.config.toml`, and provider templates.
- **Rename `WithCustomProvider` → `Advanced`** — same behavior (mycode templates + provider block in `local.config.toml`), updated CLI flag and TUI label.
- **Keep `Starter` as-is** — already provides variables, `.env`, and `local.config.toml` with rendering-ready config. No content changes.
- **Default template changes from `Starter` → `Blank`** — non-interactive `dotagents init` now produces minimal scaffolding.
- **TUI template selector updated** — three options with concise descriptions:
  - Blank → "Minimal scaffolding"
  - Starter → "Variables, env & rendering"
  - Advanced → "Custom provider & overrides"
- **`build_config_content` refactored** — returns `(global, local)` where `Blank` produces a local that signals "skip write" (or caller handles it).

## Capabilities

### New Capabilities

- **Blank init template** — minimal scaffolding for users who want to build config from scratch.

### Modified Capabilities

- **Starter template** — no content change, but no longer the default.
- **WithCustomProvider → Advanced** — renamed enum variant and CLI flag (`--template advanced`).
- **Default template** — changed from `Starter` to `Blank`.
- **TUI wizard** — template selector shows 3 options instead of 2.

## Impact

- `src/cli/options.rs` — `InitTemplate` enum: add `Blank`, rename `WithCustomProvider` → `Advanced`, update `ValueEnum` labels, change default.
- `src/cli/init.rs` — `build_config_content` handles `Blank` (no local config), file write logic skips `local.config.toml` and `.env` for `Blank`, default template resolves to `Blank`.
- `src/cli/ui/init.rs` — TUI template selector adds `Blank` option, updates descriptions, renames `WithCustomProvider` → `Advanced`.
- `openspec/specs/init-templates/spec.md` — updated from 2-template to 3-template spec.
- No new dependencies.

## Verification

- `mise check` — cargo fmt + clippy pass.
- `mise tests` — all unit, integration, and e2e tests pass.
- Manual tui-devtools testing: run `dotagents init` interactively, verify all 3 templates produce correct file sets.
