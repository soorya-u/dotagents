## MODIFIED Requirements

### Requirement: Deploy pipeline processes ignore feature
The system SHALL process the `ignore` feature in the deploy pipeline alongside `commands`, `instructions`, `mcp`, and `skills`. When `"ignore"` is in the `features` list, the pipeline SHALL load patterns from config, build an `IgnoreFeature`, and render it once per active provider.

List-typed fields in the deploy config (`features`, `targets`) SHALL use whole-list replacement during config layering — the local config value completely replaces the global value with no union or element-wise merge.

When the deploy pipeline writes output to a target file that is a structured config format (JSON, JSONC, TOML, YAML) and the file already exists, the system SHALL perform a read-modify-write merge instead of a pure overwrite. The rendered output is merged at the top level on top of the existing file, with rendered values winning on key conflicts. Nested objects are replaced rather than recursively merged. See the `deploy-merge-write` capability for full merge semantics.

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

#### Scenario: Deploy merges MCP into existing shared config file
- **WHEN** deploying MCP to `.gemini/settings.json` and the file already exists with user settings
- **THEN** the system SHALL merge the rendered MCP output into the existing file, preserving user settings outside the `mcpServers` key
