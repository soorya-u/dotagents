## Purpose

E2e tests for graceful handling of missing config files in the config command.

## Requirements

### Requirement: E2e test for missing local.config.toml graceful handling
Verify that `config local` exits cleanly when `local.config.toml` does not exist.

#### Scenario: Missing local.config.toml shows message, exits 0 (TC-CFG-08)
- **WHEN** `.dotagents/local.config.toml` is deleted and `config local --ci` is run
- **THEN** exit code is 0, stdout contains "No local config found"

#### Scenario: Missing local.config.toml with --json returns empty object (TC-CFG-08)
- **WHEN** `.dotagents/local.config.toml` is deleted and `config local --json` is run
- **THEN** exit code is 0, stdout is `{}` (valid empty JSON object)

### Requirement: E2e test for missing config.toml error
Verify that `config global` exits with error when `config.toml` does not exist.

#### Scenario: Missing config.toml exits non-zero (TC-CFG-09)
- **WHEN** `.dotagents/config.toml` is deleted and `config global --ci` is run
- **THEN** exit code is 1, stderr contains "not found" referencing `config.toml`

#### Scenario: Missing config.toml with --json also exits non-zero (TC-CFG-09)
- **WHEN** `.dotagents/config.toml` is deleted and `config global --json` is run
- **THEN** exit code is 1, stderr contains "not found" referencing `config.toml`
