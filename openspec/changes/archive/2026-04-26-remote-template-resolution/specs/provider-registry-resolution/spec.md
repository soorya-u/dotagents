## ADDED Requirements

### Requirement: Missing template or target is resolved from the official registry
When a `FeatureSettings` block has `template = None` and/or `target = None`, `dotagents deploy` SHALL attempt to fill the missing fields by fetching `https://dotagents.soorya-u.dev/v1/templates/registry.json` and parsing the provider's `provider.toml`. Resolution is per-field: if only one field is absent, only that field is filled in. The resolved values are used in-memory only; `config.toml` is never written.

#### Scenario: Both template and target missing — resolved from registry
- **WHEN** a provider appears in `targets` with no `[providers.<name>.<feature>]` block configured
- **THEN** deploy fetches `registry.json`, downloads the provider's `provider.toml`, extracts the feature's `template` URL and `target` path, and proceeds with rendering using those values

#### Scenario: Only target missing — template taken from config, target from registry
- **WHEN** `[providers.claude.commands]` has `template = "my-custom.hbs"` but no `target`
- **THEN** deploy uses `my-custom.hbs` as the template and fills in `target` from the registry's `provider.toml` for the `claude` provider's `commands` feature

#### Scenario: Both fields present — registry is not consulted
- **WHEN** a provider's `FeatureSettings` has both `template` and `target` set
- **THEN** `registry.json` is not fetched for that provider/feature; existing behaviour is unchanged

### Requirement: Registry is fetched once per deploy invocation
`dotagents deploy` SHALL fetch `registry.json` at most once per invocation, regardless of how many providers require auto-resolution.

#### Scenario: Multiple providers with missing config share a single registry fetch
- **WHEN** three providers each have missing `template`/`target`
- **THEN** `registry.json` is fetched exactly once and all three providers are resolved from the in-memory result

### Requirement: Provider absent from registry is skipped with a warning
If the registry does not list a provider that requires auto-resolution, `dotagents deploy` SHALL emit a warning and skip all features for that provider. Other providers are unaffected.

#### Scenario: Unknown provider name — warn and skip
- **WHEN** `targets` includes `"my-custom-ide"` and that name does not appear in `registry.json`
- **THEN** a warning is logged and `my-custom-ide` is skipped; deploy continues for all other providers

### Requirement: Provider found in registry but lacking a requested feature is skipped with a warning
If the provider's `provider.toml` does not declare a block for the requested feature, `dotagents deploy` SHALL emit a warning identifying the provider and feature, and skip that provider/feature combination.

#### Scenario: Provider has no skill template — warn and skip
- **WHEN** the `skills` feature is enabled globally and `goose` is in `targets` but `provider.toml` for `goose` has no `[providers.goose.skills]` block
- **THEN** a warning is logged ("Provider 'goose' does not support the 'skills' feature — skipping") and `goose` is skipped for `skills`; other features for `goose` proceed normally

### Requirement: Registry fetch failure in online mode falls back to the template-source cache
If `registry.json` cannot be fetched due to a network error and `--offline` has not been specified, `dotagents deploy` SHALL log a warning and attempt to resolve missing fields from the local template-source cache. If the cache is also cold for a provider, a second warning SHALL be emitted and that provider/feature SHALL be skipped. The deploy SHALL NOT hard-error.

#### Scenario: Registry unreachable, cache warm — deploy succeeds with warning
- **WHEN** `registry.json` cannot be fetched and the provider's `provider.toml` is present in the template-source cache
- **THEN** a warning is logged about the registry fetch failure, the cached `provider.toml` is used for resolution, and deploy proceeds

#### Scenario: Registry unreachable, cache cold — provider skipped with warning
- **WHEN** `registry.json` cannot be fetched and no cached `provider.toml` exists for the provider
- **THEN** a warning is logged for the registry failure and a second warning is logged identifying the provider/feature as skipped; deploy continues for other providers

### Requirement: `--offline` flag skips registry fetch and resolves from cache only
When `dotagents deploy --offline` is specified, `dotagents deploy` SHALL NOT make any network request for registry or template resolution. Missing fields SHALL be resolved from the template-source cache only. If the cache is cold for a required provider/feature, deploy SHALL error with a clear message.

#### Scenario: Offline mode, cache warm — resolves without network
- **WHEN** `--offline` is passed and the provider's `provider.toml` is in the template-source cache
- **THEN** no network request is made; resolution uses the cached `provider.toml`; deploy proceeds

#### Scenario: Offline mode, cache cold — hard error
- **WHEN** `--offline` is passed and no cached `provider.toml` exists for a provider that requires auto-resolution
- **THEN** deploy stops with an error identifying the provider and instructing the user to run without `--offline` first to populate the cache
