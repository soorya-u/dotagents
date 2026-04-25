## ADDED Requirements

### Requirement: Flat targets list
The configuration SHALL declare deploy targets as a flat array of provider name strings under a top-level `targets` key. No grouping by provider type (ide/cli/custom) SHALL be supported.

#### Scenario: Valid flat targets
- **WHEN** `config.toml` contains `targets = ["claude", "cursor", "gemini"]`
- **THEN** `dotagents deploy` renders output for all three providers

#### Scenario: Empty targets list
- **WHEN** `targets = []` or `targets` is omitted
- **THEN** `dotagents deploy` produces no output and exits successfully

#### Scenario: Local config overrides targets
- **WHEN** `config.toml` has `targets = ["claude", "cursor"]` and `local.config.toml` has `targets = ["gemini"]`
- **THEN** `dotagents deploy` renders output only for `gemini` (local fully overrides global)

### Requirement: Flat provider config keys
Provider feature settings SHALL be declared as `[providers.<name>.<feature>]` with no intermediate group namespace. The keys `ide`, `cli`, and `custom` SHALL NOT be valid as provider grouping levels.

#### Scenario: Provider feature settings under flat key
- **WHEN** `config.toml` contains `[providers.claude.commands]` with valid `template` and `target`
- **THEN** `dotagents deploy` renders the commands feature for the `claude` provider using those settings

#### Scenario: Old grouped key is rejected
- **WHEN** `config.toml` contains `[providers.cli.claude.commands]`
- **THEN** the config is not parsed as a valid provider named `claude`; the key `cli` is treated as a provider name instead

### Requirement: Provider names are unique across config
A provider name SHALL appear at most once in `targets` and at most once as a key under `[providers]`. Duplicate entries in the `targets` array SHALL be silently deduplicated.

#### Scenario: Duplicate target deduplicated
- **WHEN** `targets = ["claude", "claude"]`
- **THEN** `dotagents deploy` renders output for `claude` exactly once

### Requirement: Local config deep-merges providers
Per-provider feature settings from `local.config.toml` SHALL be deep-merged over global settings at the individual field level (`template`, `target`, `disabled`, `variables`, `hash`), consistent with the existing `FeatureSettings::merge` behavior.

#### Scenario: Local adds a provider not in global
- **WHEN** `config.toml` declares no `[providers.myagent]` section and `local.config.toml` adds `[providers.myagent.commands]` with valid settings
- **THEN** `dotagents deploy` renders commands for `myagent` if `myagent` appears in the merged `targets`

#### Scenario: Local overrides a single field
- **WHEN** global sets `[providers.claude.commands] template = "url-a"` and local sets `[providers.claude.commands] target = "path-b"` only
- **THEN** the merged config uses `template = "url-a"` from global and `target = "path-b"` from local

### Requirement: Registry generation uses flat template layout
The CI registry generation script SHALL scan `public/v1/templates/*/` (flat) to discover providers. No `cli/` or `ide/` subdirectory structure SHALL be assumed.

#### Scenario: Registry includes all flat providers
- **WHEN** `scripts/ci/generate_registry.sh` is run
- **THEN** `public/v1/templates/registry.json` lists all providers found under `public/v1/templates/<provider>/provider.toml`

#### Scenario: Missing provider.toml is skipped
- **WHEN** a directory exists under `public/v1/templates/` with no `provider.toml`
- **THEN** that directory is excluded from the generated registry
