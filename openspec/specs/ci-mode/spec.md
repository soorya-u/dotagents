# CI Mode

## Purpose

Defines how the CLI behaves in non-interactive (CI) environments, including how CI mode is activated and which behaviors it suppresses.

## Requirements

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

### Requirement: CI deploy fails on user-edited files
When `dotagents deploy` runs in CI mode (non-TTY) and detects user-edited target files without `--force`, it SHALL exit with status 1 and display a concise error message with the count of edited files. Individual file paths SHALL NOT be listed in the error output.

#### Scenario: CI deploy exits 1 when edited files detected
- **WHEN** `dotagents deploy` runs in non-TTY mode, at least one target file was manually edited, and `--force` is not passed
- **THEN** the process SHALL exit with status 1

#### Scenario: CI deploy error message shows count not paths
- **WHEN** `dotagents deploy` exits with status 1 due to user-edited files in non-TTY mode
- **THEN** the error output SHALL state the number of edited files and suggest `--force` to override, without listing individual file paths

#### Scenario: CI deploy with --force exits 0 despite edited files
- **WHEN** `dotagents deploy --force` runs in non-TTY mode and target files were manually edited
- **THEN** the process SHALL exit with status 0 after overwriting all files

