## Why

Users currently have no way to browse the provider registry outside the init wizard. After init, the 24 registered providers are invisible — users can't discover new providers, look up documentation, or verify which providers are available without re-running init or digging into the registry JSON. A `providers` command gives users visibility into the ecosystem.

## What Changes

- Add a `dotagents providers ls` subcommand that lists all providers from the official registry
- Extend `registry.json` entries with `name` (human-readable display label) and `url` (documentation link) fields — **additive, not breaking**
- CLI mode: list provider slug + name per line; `--url` appends URLs; `--json` outputs structured JSON
- TUI mode: interactive fuzzy-search browser showing provider details with clickable URLs
- Offline mode: lists providers from the cached registry if available, errors if cold and `--offline` is set
- Update the CI registry generation script to include `name` and `url` fields for each provider

## Capabilities

### New Capabilities
- `providers-list`: listing and browsing registered AI agent providers from the official registry, with machine-readable JSON and interactive TUI modes

### Modified Capabilities
- `provider-registry-resolution`: registry entry schema extended with `name` and `url` fields — resolution logic unchanged, new fields are purely additive for display purposes

## Impact

- `src/schema/registry.rs` — `ProviderEntry` struct gains `name` and `url` fields
- `src/cli/providers.rs` — new module implementing the `providers ls` subcommand
- `src/cli/options.rs` — new `Action::Providers` variant with `ProvidersAction` enum
- `public/v1/templates/registry.json` — each provider entry extended with `name` and `url`
- `scripts/ci/generate_registry.sh` — updated to include `name`/`url` from `provider.toml`
- `tests/e2e/` — new e2e tests for CLI and TUI paths
