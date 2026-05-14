## ADDED Requirements

### Requirement: CI mode flag

The CLI SHALL accept a global `--ci` flag on the `Options` struct. When present, the process SHALL treat `is_tty()` as `false` for its entire lifetime, regardless of PTY state.

#### Scenario: --ci suppresses interactive prompts on deploy

- **WHEN** user runs `dotagents --ci deploy`
- **THEN** no interactive prompts are shown and deploy runs with non-TTY defaults (online mode, gitignore skipped)

#### Scenario: --ci suppresses the init wizard

- **WHEN** user runs `dotagents --ci init`
- **THEN** the init wizard is not shown; all features are enabled, Starter template is used, and no targets are selected

#### Scenario: --ci suppresses undeploy confirmation

- **WHEN** user runs `dotagents --ci undeploy`
- **THEN** no confirmation prompt is shown and undeploy proceeds automatically

#### Scenario: --ci is accepted before any subcommand

- **WHEN** user runs `dotagents --ci <any-subcommand>`
- **THEN** the flag is accepted without error and CI mode is active for that subcommand

### Requirement: CI mode environment variable

The CLI SHALL check the `DOTAGENTS_CI` environment variable at startup. When the value is `true`, `1`, or `yes` (case-insensitive), the process SHALL activate CI mode identically to the `--ci` flag.

#### Scenario: DOTAGENTS_CI=true activates CI mode

- **WHEN** the environment has `DOTAGENTS_CI=true` and user runs `dotagents deploy`
- **THEN** no interactive prompts are shown and deploy runs with non-TTY defaults

#### Scenario: DOTAGENTS_CI=false leaves TTY detection unchanged

- **WHEN** the environment has `DOTAGENTS_CI=false` and user runs `dotagents deploy` in a TTY
- **THEN** interactive prompts appear normally

#### Scenario: --ci flag overrides absent DOTAGENTS_CI

- **WHEN** `DOTAGENTS_CI` is not set and user passes `--ci`
- **THEN** CI mode is active

### Requirement: CI mode does not affect logging

The `--ci` flag SHALL NOT change log verbosity or output format. It is orthogonal to `--quiet` and `--verbose`.

#### Scenario: --ci and --quiet can be combined

- **WHEN** user runs `dotagents --ci --quiet deploy`
- **THEN** CI mode suppresses prompts and --quiet suppresses non-error output independently

#### Scenario: --ci alone does not suppress info logs

- **WHEN** user runs `dotagents --ci deploy` with default verbosity
- **THEN** info-level log output is unchanged compared to a non-CI non-TTY run

### Requirement: Direct is_terminal() call sites use is_tty()

All interactive-mode gates in `init.rs` and `config.rs` that call `std::io::stdin().is_terminal()` directly SHALL be replaced with `is_tty()` so CI mode affects them.

#### Scenario: init wizard gate respects CI mode

- **WHEN** CI mode is active and `is_tui_mode()` is evaluated
- **THEN** it returns `false` even if stdin is a terminal

#### Scenario: config edit gate respects CI mode

- **WHEN** CI mode is active and `dotagents config --edit global` is run
- **THEN** interactive edit mode is not entered
