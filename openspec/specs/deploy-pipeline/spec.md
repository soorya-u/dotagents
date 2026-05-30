## Purpose

Defines the deploy pipeline's processing of all feature types including the ignore feature.

## Requirements

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

### Requirement: Deploy pipeline handles empty patterns
The system SHALL gracefully handle the case where `.agentignore` is empty or missing.

#### Scenario: Empty .agentignore produces no file
- **WHEN** `.dotagents/.agentignore` exists but is empty
- **THEN** the deploy pipeline SHALL NOT write any ignore files

#### Scenario: Missing .agentignore produces no file
- **WHEN** `.dotagents/.agentignore` does not exist
- **THEN** the deploy pipeline SHALL NOT write any ignore files

### Requirement: Ignore files tracked in gitignore fence
The system SHALL include deployed ignore file paths in the `.gitignore` fence section managed by dotagents.

#### Scenario: Ignore files added to gitignore fence
- **WHEN** deploy writes `.ignore` and `.aiignore` files
- **THEN** `rebuild_fence_from_cache()` SHALL include these paths in the `#region dotagents` section of `.gitignore`

#### Scenario: Undeploy removes ignore files from gitignore fence
- **WHEN** `dotagents undeploy` is run
- **THEN** `clear_gitignore_fence()` SHALL remove ignore file paths from the `.gitignore` fence section

### Requirement: Deploy pipeline uses single-phase rendering for ignore
The system SHALL render ignore templates in a single phase (no two-phase content rendering) since ignore files have no frontmatter or complex structure.

#### Scenario: Single-phase rendering for ignore
- **WHEN** the deploy pipeline processes the ignore feature
- **THEN** it SHALL skip the content pre-rendering phase and render the template directly against `var.*` and `ignore.patterns`

#### Scenario: Variables available in ignore templates
- **WHEN** an ignore template references `{{ var.workspace_exclude }}`
- **THEN** the rendered output SHALL contain the value of that variable
