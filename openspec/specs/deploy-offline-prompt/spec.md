## ADDED Requirements

### Requirement: Deploy defaults to online mode
When `dotagents deploy` runs without the `--offline` flag, it SHALL attempt to fetch the provider registry and resolve templates from the network. No interactive prompt is shown to ask the user about offline mode.

#### Scenario: Default deploy runs online
- **WHEN** `dotagents deploy` is run without `--offline`
- **THEN** deploy proceeds with registry fetch enabled and attempts network resolution

#### Scenario: Network failure falls back to cache
- **WHEN** deploy runs online but the registry fetch fails
- **THEN** a warning is logged and deploy falls back to the local template-source cache

#### Scenario: Network failure and cache cold — provider skipped
- **WHEN** deploy runs online, the registry fetch fails, and no cached `provider.toml` exists for a provider
- **THEN** a warning is logged identifying the provider as skipped; deploy continues for other providers

### Requirement: --offline flag enables offline mode
When `--offline` is passed to `dotagents deploy`, deploy SHALL skip the registry fetch and resolve templates from the local template-source cache only.

#### Scenario: --offline flag skips registry fetch
- **WHEN** `dotagents deploy --offline` is run
- **THEN** no network request is made and resolution uses the cached `provider.toml`

### Requirement: Non-TTY defaults to online without prompting
When deploy runs in a non-interactive environment (CI, piped), online mode is used unless `--offline` is explicitly passed.

#### Scenario: Non-TTY online deploy
- **WHEN** deploy runs with no TTY and no `--offline` flag
- **THEN** deploy proceeds online

#### Scenario: Non-TTY with --offline flag
- **WHEN** deploy runs with no TTY and `--offline` is passed
- **THEN** deploy proceeds offline
