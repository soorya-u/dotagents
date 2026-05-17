## ADDED Requirements

### Requirement: E2e test for --deploy flag on skills new
Verify that `skills new --deploy` creates the skill and triggers a deploy pass.

#### Scenario: --deploy triggers deploy after creation (TC-SKILL-NEW-06)
- **WHEN** `skills new NAME --deploy --ci` is run with a local provider configured
- **THEN** exit code is 0, the skill directory exists in `.dotagents/skills/`, and deployed output files exist in the provider target directory

### Requirement: E2e test for --deploy flag on skills rm
Verify that `skills rm --deploy` removes the skill and re-runs deploy.

#### Scenario: --deploy re-runs deploy after removal (TC-SKILL-RM-06)
- **WHEN** multiple skills exist, `skills rm NAME --force --deploy --ci` is run with a local provider configured
- **THEN** exit code is 0, the named skill directory is deleted, remaining skills are still deployed, and the removed skill's deployed file is cleaned up
