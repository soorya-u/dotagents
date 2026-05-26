## Purpose

Defines provider-specific ignore templates, their configuration in provider.toml, and registry checksums.

## Requirements

### Requirement: Provider ignore templates
The system SHALL provide `ignore.hbs` Handlebars templates for all supported providers. Templates SHALL render patterns as newline-separated entries.

#### Scenario: Template renders patterns as lines
- **WHEN** an `ignore.hbs` template is rendered with patterns `["node_modules/", "*.log"]`
- **THEN** the output SHALL contain each pattern on its own line

#### Scenario: Template supports variable interpolation
- **WHEN** a template references `{{ var.custom_pattern }}`
- **THEN** the rendered output SHALL substitute the variable value

### Requirement: Provider.toml includes ignore feature settings
Each provider SHALL have an `[providers.<slug>.ignore]` section in its `provider.toml` with `template` and `target` fields.

#### Scenario: Opencode provider has ignore config
- **WHEN** the opencode provider.toml is parsed
- **THEN** it SHALL include `[providers.opencode.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.ignore`

#### Scenario: Auggie provider has ignore config
- **WHEN** the auggie provider.toml is parsed
- **THEN** it SHALL include `[providers.auggie.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.augmentignore`

#### Scenario: Autohand provider has ignore config
- **WHEN** the autohand provider.toml is parsed
- **THEN** it SHALL include `[providers.autohand.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.autohandignore`

#### Scenario: Junie provider has ignore config
- **WHEN** the junie provider.toml is parsed
- **THEN** it SHALL include `[providers.junie.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.aiignore`

#### Scenario: Pi provider has ignore config
- **WHEN** the pi provider.toml is parsed
- **THEN** it SHALL include `[providers.pi.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.piignore`

#### Scenario: Goose provider has ignore config
- **WHEN** the goose provider.toml is parsed
- **THEN** it SHALL include `[providers.goose.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.gooseignore`

#### Scenario: Cline provider has ignore config
- **WHEN** the cline provider.toml is parsed
- **THEN** it SHALL include `[providers.cline.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.clineignore`

#### Scenario: Gemini provider has ignore config
- **WHEN** the gemini provider.toml is parsed
- **THEN** it SHALL include `[providers.gemini.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.geminiignore`

#### Scenario: Qwen provider has ignore config
- **WHEN** the qwen provider.toml is parsed
- **THEN** it SHALL include `[providers.qwen.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.qwenignore`

#### Scenario: Kilocode provider has ignore config
- **WHEN** the kilocode provider.toml is parsed
- **THEN** it SHALL include `[providers.kilocode.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.kilocodeignore`

#### Scenario: Cursor provider has ignore config
- **WHEN** the cursor provider.toml is parsed
- **THEN** it SHALL include `[providers.cursor.ignore]` with `template` pointing to `ignore.hbs` and `target` resolving to `{{ dir.workspace }}/.cursorignore`

> **Note**: Only providers with documented ignore-file support receive ignore templates.
> Supported providers: opencode, auggie, autohand, cline, cursor, gemini, goose, junie, kilocode, pi, qwen.
> The following providers do NOT have valid ignore file formats and are excluded:
> claude, copilot, codex, factory-droid, deepagents, kimi, mistral-vibe, qoder-cli, amp.

### Requirement: Registry includes ignore template checksums
The system SHALL update `registry.json` with SHA-256 checksums for all new `ignore.hbs` template files.

#### Scenario: Registry entry includes ignore.hbs checksum
- **WHEN** the registry is generated after adding ignore templates
- **THEN** each provider's `checksums` map SHALL include an `"ignore.hbs"` key with its SHA-256 hash

### Requirement: Ignore templates follow provider-specific formats
Each provider's ignore template SHALL render patterns in the format expected by that provider.

#### Scenario: All providers use newline-separated glob patterns
- **WHEN** any provider's ignore template is rendered
- **THEN** the output SHALL be newline-separated glob patterns (standard gitignore-like format)
