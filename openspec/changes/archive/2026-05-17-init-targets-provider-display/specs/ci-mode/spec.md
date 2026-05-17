## MODIFIED Requirements

### Requirement: CI mode flag

The CLI SHALL accept a global `--ci` flag on the `Options` struct. When present, the process SHALL treat `is_tty()` as `false` for its entire lifetime, regardless of PTY state.

#### Scenario: --ci suppresses interactive prompts on deploy

- **WHEN** user runs `dotagents --ci deploy`
- **THEN** no interactive prompts are shown and deploy runs with non-TTY defaults (online mode, gitignore skipped)

#### Scenario: --ci suppresses the init wizard

- **WHEN** user runs `dotagents --ci init`
- **THEN** the init wizard is not shown; no features are scaffolded, Starter template is used, and no targets are selected

#### Scenario: --ci suppresses undeploy confirmation

- **WHEN** user runs `dotagents --ci undeploy`
- **THEN** no confirmation prompt is shown and undeploy proceeds automatically

#### Scenario: --ci is accepted before any subcommand

- **WHEN** user runs `dotagents --ci <any-subcommand>`
- **THEN** the flag is accepted without error and CI mode is active for that subcommand
