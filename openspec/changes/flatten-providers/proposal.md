## Why

The `ide`/`cli`/`custom` grouping in `Targets` and `Providers` has no behavioral effect at deploy time — the runtime immediately chains all three groups into a flat map and forgets which group a provider came from. The grouping adds config noise, creates an artificial taxonomy (many tools are both CLI and IDE), and causes a registry generation bug where the CI script looks for `cli/` and `ide/` subdirectories that don't exist in `public/v1/templates/`.

## What Changes

- **BREAKING**: `targets` in `config.toml` changes from a grouped table (`[targets] ide = [...] cli = [...] custom = [...]`) to a flat list (`targets = ["claude", "cursor", ...]`)
- **BREAKING**: Provider config keys change from `[providers.<group>.<name>.<feature>]` to `[providers.<name>.<feature>]`
- **BREAKING**: `Targets` struct loses `ide`, `cli`, `custom` fields — replaced by a single `providers: Option<HashSet<String>>`
- **BREAKING**: `Providers` struct loses `ide`, `cli`, `custom` fields — replaced by a flat `Option<HashMap<String, Features>>`
- The three-iterator chain in `AppConfig::get_provider_feature_settings` becomes a single flat iterator
- `custom_providers` validation in `global.rs` and `local.rs` is removed (no longer meaningful)
- `cache.rs` match arms for `"ide"` / `"cli"` / `"custom"` are removed
- `provider.toml` snippets in `public/v1/templates/*/provider.toml` updated to new key shape
- Mock files (`src/mocks/config.toml`, `src/mocks/local.config.toml`) rewritten for new shape
- Registry generation script (`scripts/ci/generate_registry.sh`) updated to scan the existing flat `public/v1/templates/<provider>/` layout instead of the non-existent `cli/`/`ide/` subdirs

No migration path — this is a hard break appropriate for pre-1.0.

## Capabilities

### New Capabilities

- `flat-provider-config`: Defines the new flat configuration schema for declaring deploy targets and per-provider feature settings. Replaces the three-group `Targets`/`Providers` model with a single flat list and map respectively.

### Modified Capabilities

*(none — no existing specs)*

## Impact

- `src/schema/config/common.rs` — `Targets` and `Providers` structs, all merge impls, tests
- `src/schema/config/app.rs` — `get_provider_feature_settings` iterator logic
- `src/schema/config/global.rs` — `custom_providers` validation removed
- `src/schema/config/local.rs` — `custom_providers` validation removed
- `src/schema/config/cache.rs` — group-name match arms removed
- `src/mocks/config.toml`, `src/mocks/local.config.toml` — rewritten
- `public/v1/templates/*/provider.toml` — all 14 provider snippets updated
- `scripts/ci/generate_registry.sh` — flat layout scanning
