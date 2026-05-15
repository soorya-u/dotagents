## MODIFIED Requirements

### Requirement: Missing template or target is resolved from the official registry
When a `FeatureSettings` block has `template = None` and/or `target = None`, `dotagents deploy` SHALL attempt to fill the missing fields by fetching `https://dotagents.soorya-u.dev/v1/templates/registry.json` and parsing the provider's `provider.toml`. Resolution is per-field: if only one field is absent, only that field is filled in. The resolved values are used in-memory only; `config.toml` is never written. The registry schema now includes optional `name` and `url` fields per provider entry; these SHALL be ignored by deploy resolution.

#### Scenario: Both template and target missing — resolved from registry
- **WHEN** a provider appears in `targets` with no `[providers.<name>.<feature>]` block configured
- **THEN** deploy fetches `registry.json`, downloads the provider's `provider.toml`, extracts the feature's `template` URL and `target` path, and proceeds with rendering using those values. The `name` and `url` fields in the registry entry are ignored by deploy.

#### Scenario: Only target missing — template taken from config, target from registry
- **WHEN** `[providers.claude.commands]` has `template = "my-custom.hbs"` but no `target`
- **THEN** deploy uses `my-custom.hbs` as the template and fills in `target` from the registry's `provider.toml` for the `claude` provider's `commands` feature

#### Scenario: Both fields present — registry is not consulted
- **WHEN** a provider's `FeatureSettings` has both `template` and `target` set
- **THEN** `registry.json` is not fetched for that provider/feature; existing behaviour is unchanged

## ADDED Requirements

### Requirement: Registry entries MAY include name and url display fields
Each provider entry in `registry.json` MAY include a `name` field (human-readable display label) and a `url` field (documentation hyperlink). Both fields SHALL be strings when present. Absence of these fields SHALL NOT cause parsing failures; the fields are purely additive for display purposes.

#### Scenario: Registry entry with name and url parses successfully
- **WHEN** `registry.json` contains `"gemini": { "path": "...", "checksums": {...}, "name": "Gemini CLI", "url": "https://google-gemini.github.io/cli" }`
- **THEN** the `Registry` struct parses `name` as `Some("Gemini CLI")` and `url` as `Some("https://google-gemini.github.io/cli")`

#### Scenario: Registry entry without name and url parses successfully
- **WHEN** `registry.json` contains `"claude": { "path": "...", "checksums": {...} }` without `name` or `url` fields
- **THEN** the `Registry` struct parses `name` as `None` and `url` as `None`

#### Scenario: Deploy resolution ignores name and url fields
- **WHEN** deploy resolves a provider's template/target from the registry and the entry has `name` and `url` fields
- **THEN** deploy proceeds normally; the `name` and `url` fields have no effect on template resolution or rendering
