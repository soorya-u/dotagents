## ADDED Requirements

### Requirement: Config preserves unknown top-level keys on round-trip
The system SHALL preserve any TOML keys in `config.toml` and `local.config.toml` that are not recognized by the config schema. When a config file is parsed and then serialized, all unknown top-level keys SHALL appear in the output with their original values intact.

#### Scenario: Unknown string key preserved
- **WHEN** `config.toml` contains `my-custom-key = "hello"` alongside standard keys
- **THEN** parsing and re-serializing the file SHALL include `my-custom-key = "hello"` in the output

#### Scenario: Unknown table preserved
- **WHEN** `config.toml` contains `[metadata]\nauthor = "alice"` alongside standard keys
- **THEN** parsing and re-serializing the file SHALL include the `[metadata]` table with `author = "alice"`

#### Scenario: Unknown array preserved
- **WHEN** `config.toml` contains `tags = ["rust", "cli"]` alongside standard keys
- **THEN** parsing and re-serializing the file SHALL include `tags = ["rust", "cli"]`

#### Scenario: Config without unknown keys behaves identically
- **WHEN** `config.toml` contains only recognized keys
- **THEN** parsing and re-serializing SHALL produce the same output as before this change

### Requirement: Unknown keys merged during config layering
The system SHALL merge unknown top-level keys from `config.toml` (global) and `local.config.toml` (local) using shallow union. When both files define the same unknown key, the local value SHALL override the global value.

#### Scenario: Unknown key only in global
- **WHEN** `config.toml` has `custom = "global"` and `local.config.toml` does not define `custom`
- **THEN** the merged config SHALL contain `custom = "global"`

#### Scenario: Unknown key only in local
- **WHEN** `config.toml` does not define `custom` and `local.config.toml` has `custom = "local"`
- **THEN** the merged config SHALL contain `custom = "local"`

#### Scenario: Unknown key in both global and local
- **WHEN** `config.toml` has `custom = "global"` and `local.config.toml` has `custom = "local"`
- **THEN** the merged config SHALL contain `custom = "local"`

### Requirement: List fields use whole-list replacement
The system SHALL use whole-list replacement for all list-typed config fields (`features`, `targets`) during config layering. When the local config defines a list field, it SHALL completely replace the global value. No union, append, or element-wise merge SHALL occur.

#### Scenario: Local features replaces global features entirely
- **WHEN** `config.toml` has `features = ["commands", "mcp"]` and `local.config.toml` has `features = ["instructions"]`
- **THEN** the merged config SHALL have `features = ["instructions"]` (not `["commands", "mcp", "instructions"]`)

#### Scenario: Local targets replaces global targets entirely
- **WHEN** `config.toml` has `targets = ["claude", "codex"]` and `local.config.toml` has `targets = ["cursor"]`
- **THEN** the merged config SHALL have `targets = ["cursor"]`

#### Scenario: Local omits list field, global value used
- **WHEN** `config.toml` has `features = ["commands"]` and `local.config.toml` does not define `features`
- **THEN** the merged config SHALL have `features = ["commands"]`
