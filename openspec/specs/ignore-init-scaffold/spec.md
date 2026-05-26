## Purpose

Defines how the ignore feature is integrated into the init command, including CLI flags, TUI wizard, and scaffolding of the default `.agentignore` file.

## Requirements

### Requirement: Ignore feature in init Feature enum
The system SHALL include an `Ignore` variant in the `Feature` enum used by `dotagents init` for feature selection.

#### Scenario: Feature enum includes Ignore variant
- **WHEN** the `Feature` enum is used in `dotagents init --features`
- **THEN** it SHALL accept `ignore` as a valid feature value

#### Scenario: Feature::as_str returns "ignore"
- **WHEN** `Feature::Ignore.as_str()` is called
- **THEN** it SHALL return `"ignore"`

### Requirement: TUI init wizard includes ignore feature
The TUI init wizard SHALL present "Ignore Patterns" as a selectable feature in the multiselect prompt.

#### Scenario: Ignore appears in TUI feature selection
- **WHEN** the TUI init wizard displays the feature multiselect
- **THEN** it SHALL include an item labeled "Ignore Patterns" with description "Sync ignore patterns to AI tools"

#### Scenario: Ignore is pre-selected by default
- **WHEN** the TUI feature multiselect is displayed
- **THEN** "Ignore Patterns" SHALL be pre-selected alongside "Custom Commands", "INSTRUCTIONS.md", and "MCP Configuration"

### Requirement: Init scaffolds `.agentignore` file
The system SHALL create a default `.dotagents/.agentignore` file when the ignore feature is selected during `init`.

#### Scenario: Init creates .agentignore when feature is enabled
- **WHEN** `dotagents init --features commands,ignore` is run
- **THEN** a default ignore patterns file SHALL be created at `.dotagents/.agentignore`

#### Scenario: Init skips .agentignore when feature is not selected
- **WHEN** `dotagents init --features commands,instructions` is run
- **THEN** no `.agentignore` file SHALL be created

#### Scenario: Default .agentignore contains common patterns
- **WHEN** the default `.agentignore` file is created during init
- **THEN** it SHALL contain common patterns: `node_modules/`, `.git/`, `target/`, `.env`

### Requirement: Default ignore patterns mock content
The system SHALL embed default ignore patterns as a compile-time mock string, sourced from a mock file.

#### Scenario: Mock content is valid newline-separated patterns
- **WHEN** the default ignore mock content is read
- **THEN** it SHALL be a valid newline-separated list of glob patterns

### Requirement: InitOptions supports ignore feature flag
The `InitOptions` struct SHALL support the ignore feature in the `--features` flag.

#### Scenario: --features accepts ignore
- **WHEN** `dotagents init --features ignore` is run
- **THEN** the `InitOptions.features` field SHALL contain `Feature::Ignore`

#### Scenario: has_feature returns true for ignore
- **WHEN** `InitOptions.has_feature(Feature::Ignore)` is called with ignore in the features list
- **THEN** it SHALL return `true`
