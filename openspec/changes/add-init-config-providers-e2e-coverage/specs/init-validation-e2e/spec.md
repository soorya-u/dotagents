## ADDED Requirements

### Requirement: E2e test for --features none combined with other features
Verify that `--features none,commands` is rejected as an exclusive combination.

#### Scenario: Exclusive features combination exits with error (TC-INIT-07)
- **WHEN** `init --features none,commands --ci` is run
- **THEN** exit code is 1, stderr contains an error about `none` being exclusive or not combinable with other features, no `.dotagents/` directory is created

### Requirement: E2e test for invalid --features value
Verify that `init --features bogus` is rejected by Clap validation.

#### Scenario: Invalid features value exits with Clap error (TC-INIT-ERR-01)
- **WHEN** `init --features bogus --ci` is run
- **THEN** exit code is 2, stderr contains "invalid value" and lists valid feature names including `none`

### Requirement: E2e test for invalid --template value
Verify that `init --template bogus` is rejected by Clap validation.

#### Scenario: Invalid template value exits with Clap error (TC-INIT-ERR-02)
- **WHEN** `init --template bogus --ci` is run
- **THEN** exit code is 2, stderr contains "invalid value" and lists valid values (`starter`, `with-custom-provider`)
