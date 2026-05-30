## MODIFIED Requirements

### Requirement: Deploy pipeline processes ignore feature
The system SHALL process the `ignore` feature in the deploy pipeline alongside `commands`, `instructions`, `mcp`, and `skills`. When `"ignore"` is in the `features` list, the pipeline SHALL load patterns from config, build an `IgnoreFeature`, and render it once per active provider.

List-typed fields in the deploy config (`features`, `targets`) SHALL use whole-list replacement during config layering — the local config value completely replaces the global value with no union or element-wise merge.

#### Scenario: Deploy writes ignore file for each provider
- **WHEN** `features = ["commands", "ignore"]` and `targets = ["opencode", "junie"]` with patterns `["node_modules/"]`
- **THEN** the system SHALL write `.ignore` for opencode and `.aiignore` for junie, each containing `node_modules/`

#### Scenario: Deploy skips providers with disabled ignore
- **WHEN** `[providers.opencode.ignore]` has `disabled = true`
- **THEN** the system SHALL NOT write an ignore file for opencode

#### Scenario: Deploy creates parent directories for ignore files
- **WHEN** an ignore file target path includes nested directories
- **THEN** the system SHALL create all parent directories before writing the file

#### Scenario: List fields use whole-list replacement
- **WHEN** `config.toml` sets `features = ["commands", "mcp"]` and `local.config.toml` sets `features = ["instructions"]`
- **THEN** the deploy pipeline SHALL use `features = ["instructions"]` (local replaces global entirely)
