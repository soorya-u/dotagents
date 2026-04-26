## Why

Configuring a provider today requires copy-pasting template URLs and target paths from the official docs into every project's `config.toml`. Providers like `claude` and `cursor` already have canonical templates hosted at `dotagents.soorya-u.dev` — the CLI should be able to discover and use them automatically, keeping user config minimal and templates current without redundant network downloads.

## What Changes

- **New**: When `template` and/or `target` are absent from a `FeatureSettings` block, deploy auto-resolves the missing fields from the official provider registry (`/v1/templates/registry.json`) before rendering.
- **New**: Registry entries gain a `checksums` map (SHA-256 per file) enabling cache invalidation without re-downloading unchanged template files.
- **New**: Downloaded provider templates (`.hbs` files and `provider.toml`) are cached at `dirs::config_dir()/dotagents/cache/templates/<provider>/` — shared across all projects, persisted between deploys.
- **New**: `dotagents deploy --offline` skips the registry fetch and resolves only from the local template cache; errors if a required file is not cached.
- **Changed**: Network failure during registry lookup (online mode, no `--offline`) is a soft failure — the CLI logs a warning, falls back to cache, and skips the provider/feature with a second warning if the cache is also cold.
- **New**: `public/v1/schemas/registry.schema.json` added, formalising the registry document shape including the optional `checksums` field.
- **Changed**: `scripts/ci/generate_registry.sh` computes and embeds per-file SHA-256 checksums when building `registry.json`.

## Capabilities

### New Capabilities

- `provider-registry-resolution`: Auto-resolve missing `template` and/or `target` fields by fetching the provider's entry from the official registry and parsing its `provider.toml`. Priority: config file → local template cache → remote fetch. Covers `--offline` mode, network-failure degradation, and the warn-and-skip path when a provider doesn't support a given feature.
- `template-source-cache`: User-level disk cache at `dirs::config_dir()/dotagents/cache/templates/` for downloaded `provider.toml` and `.hbs` files. Cache is validated against SHA-256 checksums embedded in `registry.json`; stale or missing files are replaced on next online deploy.

### Modified Capabilities

- `remote-template-fetch`: Network failure behaviour changes in the registry-lookup path (warn + fallback vs. hard error). The existing hard-error behaviour for an explicit `template` URL is unchanged; only the new registry-fetch path uses soft failure.

## Impact

- **`src/cli/options.rs`** — add `--offline` flag to `DeployOptions`
- **`src/cli/deploy.rs`** — new `resolve_provider_defaults()` step between config load and `deploy_feature()` calls
- **`src/schema/registry.rs`** — new module: `Registry`, `ProviderRegistryEntry` structs with serde deserialisation
- **`src/utils/path.rs`** — new `get_global_template_cache_dir()` using `dirs::config_dir()`
- **`src/constants/dir.rs`** — new sub-path constant for the template cache subdirectory
- **`Cargo.toml`** — add `dirs` crate dependency
- **`scripts/ci/generate_registry.sh`** — add SHA-256 checksum loop per provider
- **`public/v1/schemas/registry.schema.json`** — created (documents `path` + optional `checksums`)
- **`src/templates/remote.rs`** — no changes; remains the low-level HTTP fetch utility
- **Existing `deploy-output-cache`** — unchanged; rendered-output hash cache at workspace level is a separate concern
