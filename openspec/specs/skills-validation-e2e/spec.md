# skills-validation-e2e Specification

## Purpose
TBD - created by archiving change add-commands-skills-e2e-coverage. Update Purpose after archive.
## Requirements
### Requirement: E2e test for CI mode empty defaults on skills new
Verify that `skills new` in CI mode with no metadata flags produces a file with empty defaults.

#### Scenario: CI mode with no flags produces empty defaults (TC-SKILL-NEW-03)
- **WHEN** `skills new NAME --ci` is run without `--description`, `--license`, or `--compatibility`
- **THEN** exit code is 0, the skill file exists, frontmatter contains `description: ''`, and `license`/`compatibility` keys are absent from frontmatter

### Requirement: E2e test for duplicate skill without --force
Verify that creating a duplicate skill without `--force` exits with an error.

#### Scenario: Duplicate skill exits non-zero (TC-SKILL-NEW-04)
- **WHEN** a skill named NAME already exists and `skills new NAME --ci` is run without `--force`
- **THEN** exit code is 1, stderr contains "already exists" and "Use --force to overwrite"

### Requirement: E2e test for --json --skill combined filter
Verify that `skills ls --json --skill NAME` returns a filtered JSON array.

#### Scenario: --json --skill returns filtered JSON (TC-SKILL-LS-06)
- **WHEN** multiple skills exist and `skills ls --json --skill NAME` is run
- **THEN** exit code is 0, stdout is valid JSON, the array contains exactly one element matching NAME

### Requirement: E2e test for --runner not on PATH
Verify that `skills add` with a runner binary not on PATH exits with a helpful error.

#### Scenario: Runner not on PATH exits non-zero (TC-SKILL-ADD-04)
- **WHEN** `skills add NAME --runner yarn` is run and `yarn` is not available on PATH
- **THEN** exit code is 1, stderr contains "not found on PATH" or similar runner error

### Requirement: E2e test for invalid --runner value
Verify that `skills add` with an invalid runner value exits with a Clap error.

#### Scenario: Invalid runner value exits with Clap error (TC-SKILL-ADD-05)
- **WHEN** `skills add NAME --runner maven` is run
- **THEN** exit code is 2, stderr contains "invalid value" and lists valid values (npm, pnpm, yarn, bun)

