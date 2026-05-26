## Purpose

Defines the ignore feature type, its implementation via FeatureTrait, pattern loading, and per-provider configuration.

## Requirements

### Requirement: Ignore feature type
The system SHALL support an `ignore` feature type alongside `commands`, `instructions`, `mcp`, and `skills`. The feature SHALL be identified by the string `"ignore"` in config files and the deploy pipeline.

#### Scenario: Feature enum includes Ignore variant
- **WHEN** the `Feature` enum is constructed
- **THEN** it SHALL include a `Feature::Ignore` variant that maps to `"ignore"` in config files

#### Scenario: Feature parsing recognizes "ignore"
- **WHEN** `Feature::from_str("ignore")` is called
- **THEN** it SHALL return `Some(Feature::Ignore)`

#### Scenario: Feature::all() includes Ignore
- **WHEN** `Feature::all()` is called
- **THEN** the returned array SHALL include `Feature::Ignore`

### Requirement: IgnoreFeature implements FeatureTrait
The system SHALL provide an `IgnoreFeature` struct that implements `FeatureTrait` for handling ignore pattern lists.

#### Scenario: IgnoreFeature holds a list of patterns
- **WHEN** an `IgnoreFeature` is created with patterns `["node_modules/", "*.log", ".env"]`
- **THEN** `to_value()` SHALL return a JSON object with an `ignore.patterns` array containing those strings

#### Scenario: IgnoreFeature serializes to newline-separated patterns
- **WHEN** `to_string()` is called on an `IgnoreFeature` with patterns `["node_modules/", "*.log"]`
- **THEN** the output SHALL be `"node_modules/\n*.log\n"` (one pattern per line, trailing newline)

#### Scenario: IgnoreFeature deserializes from newline-separated patterns
- **WHEN** `from_string("node_modules/\n*.log\n")` is called
- **THEN** the resulting `IgnoreFeature` SHALL have patterns `["node_modules/", "*.log"]`

#### Scenario: IgnoreFeature returns None for get_file_name
- **WHEN** `get_file_name()` is called on an `IgnoreFeature`
- **THEN** it SHALL return `None` (singleton feature, one file per provider)

#### Scenario: IgnoreFeature roundtrip preserves patterns
- **WHEN** an `IgnoreFeature` is serialized with `to_string()` then parsed with `from_string()`
- **THEN** the resulting patterns SHALL be identical to the original

### Requirement: Ignore patterns from `.agentignore` file
The system SHALL load ignore patterns from `.dotagents/.agentignore` — a newline-separated file (one pattern per line), similar to `INSTRUCTIONS.md` and `mcp.jsonc`.

#### Scenario: Load patterns from .agentignore
- **WHEN** `.dotagents/.agentignore` contains:
  ```
  node_modules/
  *.log
  .env
  ```
- **THEN** the deploy pipeline SHALL load these patterns into an `IgnoreFeature`

#### Scenario: Missing .agentignore is valid
- **WHEN** `.dotagents/.agentignore` does not exist
- **THEN** the deploy pipeline SHALL load an `IgnoreFeature` with zero patterns (no error)

#### Scenario: Empty .agentignore is valid
- **WHEN** `.dotagents/.agentignore` exists but is empty
- **THEN** the deploy pipeline SHALL load an `IgnoreFeature` with zero patterns

### Requirement: Per-provider ignore disable
The system SHALL support `[providers.<name>.ignore]` in `config.toml` with `disabled = true/false` to skip ignore file generation for specific providers.

#### Scenario: Provider disables ignore feature
- **WHEN** `[providers.claude.ignore]` has `disabled = true`
- **THEN** the deploy pipeline SHALL NOT write any ignore file for the claude provider

#### Scenario: Provider omits ignore section
- **WHEN** a provider has no `[providers.<name>.ignore]` section
- **THEN** the deploy pipeline SHALL write the ignore file using patterns from `.agentignore`

### Requirement: Ignore feature in Features config struct
The system SHALL add an `ignore: Option<FeatureSettings>` field to the `Features` struct in `src/core/config/common.rs`.

#### Scenario: Features struct includes ignore field
- **WHEN** a `Features` struct is serialized to TOML
- **THEN** it SHALL include an `[features.ignore]` section if `ignore` is `Some`

#### Scenario: Features::get_config returns ignore settings
- **WHEN** `get_config(&Feature::Ignore)` is called on a `Features` instance
- **THEN** it SHALL return the `ignore` field value

#### Scenario: Features::merge includes ignore
- **WHEN** two `Features` instances are merged
- **THEN** the `ignore` field SHALL be merged using the same logic as other features

### Requirement: Ignore feature gated by features list
The system SHALL only deploy the ignore feature when `"ignore"` is listed in the `features` array of `config.toml`.

#### Scenario: Deploy ignore when feature is enabled
- **WHEN** `features = ["commands", "ignore"]` in `config.toml`
- **THEN** the deploy pipeline SHALL process the ignore feature

#### Scenario: Skip deploy when feature is not listed
- **WHEN** `features = ["commands", "instructions"]` in `config.toml`
- **THEN** the deploy pipeline SHALL NOT attempt to load or deploy ignore patterns

### Requirement: Ignore feature can be disabled per-provider
The system SHALL respect `disabled = true` in `[providers.<name>.ignore]` to skip ignore file generation for specific providers.

#### Scenario: Provider disables ignore feature
- **WHEN** `[providers.claude.ignore]` has `disabled = true`
- **THEN** the deploy pipeline SHALL NOT write an ignore file for the claude provider

#### Scenario: Provider enables ignore by default
- **WHEN** a provider has no `disabled` setting for ignore
- **THEN** the deploy pipeline SHALL write the ignore file if patterns exist
