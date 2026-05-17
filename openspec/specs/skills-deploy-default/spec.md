## Purpose

Specifies the default deploy behavior for `skills new` and `skills rm` in CI/non-TTY mode, where deploy runs automatically after the mutation without needing the `--deploy` flag. Also defines the `--no-deploy` flag to suppress this automatic deploy in both CI and TTY modes.

## Requirements

### Requirement: skills new and rm deploy automatically in CI unless --no-deploy
After `skills new` or `skills rm` completes successfully, the CLI SHALL trigger a deploy pass. In non-TTY/CI mode the deploy SHALL run automatically. In TTY mode the user SHALL be prompted whether to deploy. If `--no-deploy` is passed, the deploy step SHALL be skipped entirely in both modes.

#### Scenario: skills new auto-deploys in CI mode
- **WHEN** `dotagents skills new <name> --ci` is run (or DOTAGENTS_CI=true)
- **WHEN** `--no-deploy` is NOT passed
- **THEN** a deploy pass runs after the skill directory is created
- **THEN** deployed output files appear in the configured provider target paths

#### Scenario: skills new skips deploy with --no-deploy
- **WHEN** `dotagents skills new <name> --no-deploy` is run
- **THEN** no deploy pass runs
- **THEN** no deploy output is written

#### Scenario: skills rm auto-deploys in CI mode
- **WHEN** `dotagents skills rm <name> --ci` is run (or DOTAGENTS_CI=true)
- **WHEN** `--no-deploy` is NOT passed
- **THEN** a deploy pass runs after the skill directory is deleted
- **THEN** the previously deployed output for the removed skill no longer exists

#### Scenario: skills rm skips deploy with --no-deploy
- **WHEN** `dotagents skills rm <name> --no-deploy` is run
- **THEN** no deploy pass runs
