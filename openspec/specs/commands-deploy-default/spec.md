## Purpose

Specifies the default deploy behavior for `commands new` and `commands rm` in CI/non-TTY mode, where deploy runs automatically after the mutation without needing the `--deploy` flag. Also defines the `--no-deploy` flag to suppress this automatic deploy in both CI and TTY modes.

## Requirements

### Requirement: commands new and rm deploy automatically in CI unless --no-deploy
After `commands new` or `commands rm` completes successfully, the CLI SHALL trigger a deploy pass. In non-TTY/CI mode the deploy SHALL run automatically. In TTY mode the user SHALL be prompted whether to deploy. If `--no-deploy` is passed, the deploy step SHALL be skipped entirely in both modes.

> **Note**: TTY prompt behavior is verified through unit tests (task 3.4) rather than e2e scenarios due to the complexity of simulating nested interactive prompts in a PTY session.

#### Scenario: commands new auto-deploys in CI mode
- **WHEN** `dotagents commands new <name> --ci` is run (or DOTAGENTS_CI=true)
- **WHEN** `--no-deploy` is NOT passed
- **THEN** a deploy pass runs after the file is created
- **THEN** deployed output files appear in the configured provider target paths

#### Scenario: commands new skips deploy with --no-deploy
- **WHEN** `dotagents commands new <name> --no-deploy` is run
- **THEN** no deploy pass runs
- **THEN** no deploy output is written

#### Scenario: commands rm auto-deploys in CI mode
- **WHEN** `dotagents commands rm <name> --ci` is run (or DOTAGENTS_CI=true)
- **WHEN** `--no-deploy` is NOT passed
- **THEN** a deploy pass runs after the file is deleted
- **THEN** the previously deployed output for the removed command no longer exists

#### Scenario: commands rm skips deploy with --no-deploy
- **WHEN** `dotagents commands rm <name> --no-deploy` is run
- **THEN** no deploy pass runs
