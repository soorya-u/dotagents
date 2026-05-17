# commands-error-paths-e2e Specification

## Purpose
TBD - created by archiving change add-commands-skills-e2e-coverage. Update Purpose after archive.
## Requirements
### Requirement: E2e test for CI mode empty defaults on commands new
Verify that `commands new` in CI mode with no metadata flags produces a file with empty defaults.

#### Scenario: CI mode with no flags produces empty defaults (TC-CMD-NEW-03)
- **WHEN** `commands new NAME --ci` is run without `--description`, `--category`, or `--tags`
- **THEN** exit code is 0, the command file exists, frontmatter contains `description: ''`, and `category`/`tags` keys are absent from frontmatter

### Requirement: E2e test for --cwd pointing to non-workspace
Verify that `commands new --cwd` to a directory without `.dotagents/` exits with error.

#### Scenario: --cwd to non-workspace exits non-zero (TC-CMD-NEW-10)
- **WHEN** `commands new NAME --cwd /path/without/dotagents --ci` is run
- **THEN** exit code is 1, stderr contains "No .dotagents directory found" or similar workspace error

