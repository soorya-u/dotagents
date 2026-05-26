## ADDED Requirements

### Requirement: E2e test for user-edited file preservation on redeploy
Verify that deploy does not overwrite files the user has manually edited since the last deploy.

#### Scenario: Redeploy preserves user-edited deployed file (TC-DEPLOY-07)
- **WHEN** a deployed file is modified by the user after deploy, then `deploy --offline --no-gitignore` is run again
- **THEN** exit code is 0, the modified file retains the user's edits (content unchanged from edit), and other unmodified files are still current

### Requirement: E2e test for --force overriding user-edit protection
Verify that `--force` overwrites user-edited deployed files.

#### Scenario: --force overwrites user-edited deployed file (TC-DEPLOY-08)
- **WHEN** a deployed file is modified by the user after deploy, then `deploy --force --offline --no-gitignore` is run
- **THEN** exit code is 0, the modified file is overwritten with the template output (user edits lost), file content matches a fresh deploy
