## ADDED Requirements

### Requirement: Provider deduplication for same-target writes
When multiple providers target the same resolved file path during deploy, the system SHALL write the file exactly once by selecting a single "winner" provider and skipping the rest.

#### Scenario: Single provider targets a path — no dedup
- **WHEN** only one provider targets a given file path
- **THEN** the provider renders and writes the file normally

#### Scenario: Multiple providers target the same path — dedup selects winner
- **WHEN** two or more providers resolve to the same target path for the same feature
- **THEN** exactly one provider renders and writes the file
- **THEN** the remaining providers are skipped with reason "same target as <winner>"

#### Scenario: Winner selection is alphabetical by provider name
- **WHEN** multiple providers target the same path and no priority is configured
- **THEN** the provider with the alphabetically first name is selected as the winner

#### Scenario: Dedup is per-target-path, not per-feature
- **WHEN** provider A and B target `AGENTS.md` and provider C targets `CLAUDE.md` for the same feature
- **THEN** A or B wins for `AGENTS.md` and C writes to `CLAUDE.md` without dedup

#### Scenario: Dedup applies only within a single deploy_feature call
- **WHEN** the instructions feature and MCP feature both target `AGENTS.md`
- **THEN** each feature deploys independently — dedup does not cross feature boundaries

#### Scenario: Disabled providers do not participate in dedup
- **WHEN** a provider has `disabled = true` in its feature settings
- **THEN** that provider is excluded from dedup consideration (already filtered by `get_provider_feature_settings`)

### Requirement: Dedup logging behavior
The system SHALL log dedup decisions at appropriate log levels based on context.

#### Scenario: Normal deploy — dedup logged at debug level
- **WHEN** a provider is skipped due to dedup during a normal deploy
- **THEN** a debug-level log message is emitted: `"provider <skipped> targets same file as <winner> — deduplicating"`

#### Scenario: Dry-run — dedup shown in output summary
- **WHEN** `--dry-run` is set and dedup occurs
- **THEN** the dry-run summary shows which provider would write and which were skipped

### Requirement: Dedup stats tracking
The system SHALL track dedup decisions in `DeployStats` for summary output.

#### Scenario: Skipped count includes dedup-skipped providers
- **WHEN** providers are skipped due to dedup
- **THEN** `DeployStats::skipped` is incremented for each dedup-skipped provider
