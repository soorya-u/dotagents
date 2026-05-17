## ADDED Requirements

### Requirement: E2e test for --deploy flag on commands new
Verify that `commands new --deploy` creates the command file and triggers a deploy pass.

#### Scenario: --deploy triggers deploy after creation (TC-CMD-NEW-06)
- **WHEN** `commands new NAME --deploy --ci` is run with a local provider configured
- **THEN** exit code is 0, the command file exists in `.dotagents/commands/`, and deployed output files exist in the provider target directory

### Requirement: E2e test for --deploy flag on commands rm
Verify that `commands rm --deploy` removes the command and re-runs deploy.

#### Scenario: --deploy re-runs deploy after removal (TC-CMD-RM-06)
- **WHEN** multiple commands exist, `commands rm NAME --force --deploy --ci` is run with a local provider configured
- **THEN** exit code is 0, the named command is deleted, remaining commands are still deployed, and the removed command's deployed file is cleaned up
