### Requirement: Log output routes based on TTY mode
The system SHALL detect whether stdout is an interactive terminal once at startup and cache the result. All subsequent log output SHALL be routed to cliclack in TTY mode and to simplelog in non-TTY mode without re-checking TTY state.

#### Scenario: TTY mode routes to cliclack
- **WHEN** the process runs with stdout connected to a terminal
- **THEN** `warn!`, `error!`, `success!`, `step!` calls render using cliclack styled output

#### Scenario: Non-TTY mode routes to simplelog
- **WHEN** the process runs with stdout piped or redirected
- **THEN** all log macros produce plain-text output via simplelog

#### Scenario: TTY mode detection is cached
- **WHEN** `set_log_config()` is called
- **THEN** `is_tty()` is called exactly once and the result is stored in `OnceLock<LogConfig>`

### Requirement: Verbosity levels gate output in both modes
In non-TTY mode the system SHALL apply `LevelFilter` from the `-v` / `-vv` / `-vvv` flags to simplelog. In TTY mode the same filter SHALL gate cliclack output for `info!`, `debug!`, and `trace!`.

#### Scenario: Default verbosity suppresses info in TTY
- **WHEN** no `-v` flag is passed and mode is TTY
- **THEN** `info!` calls produce no output

#### Scenario: `-v` enables info in TTY
- **WHEN** `-v` is passed and mode is TTY
- **THEN** `info!` calls render via `cliclack::log::info()`

#### Scenario: `-vv` enables debug as remark in TTY
- **WHEN** `-vv` is passed and mode is TTY
- **THEN** `debug!` calls render via `cliclack::log::remark()`

#### Scenario: `-vvv` enables trace as remark in TTY
- **WHEN** `-vvv` is passed and mode is TTY
- **THEN** `trace!` calls render via `cliclack::log::remark()`

### Requirement: `--quiet` applies to non-TTY mode only
The system SHALL suppress all output below `error` level when `--quiet` is passed in non-TTY mode. In TTY mode `--quiet` SHALL have no effect — `warn!` and `error!` are always rendered.

#### Scenario: Quiet non-TTY suppresses warnings
- **WHEN** `--quiet` is passed and mode is non-TTY
- **THEN** `warn!` calls produce no output

#### Scenario: Quiet TTY still shows warnings
- **WHEN** `--quiet` is passed and mode is TTY
- **THEN** `warn!` calls still render via `cliclack::log::warning()`

### Requirement: `success!` and `step!` are first-class log macros
The system SHALL provide `success!` and `step!` macros as semantic log levels. In TTY mode they SHALL always render regardless of verbosity. In non-TTY mode they SHALL degrade to `info` and `debug` levels respectively, controlled by `-v` / `-vv`.

#### Scenario: `success!` renders styled in TTY
- **WHEN** `success!("Created {}", path)` is called and mode is TTY
- **THEN** output renders via `cliclack::log::success()`

#### Scenario: `success!` degrades to info in non-TTY
- **WHEN** `success!("Created {}", path)` is called and mode is non-TTY
- **THEN** output is logged at `info` level via simplelog (visible with `-v`)

#### Scenario: `step!` renders styled in TTY
- **WHEN** `step!("Fetching providers")` is called and mode is TTY
- **THEN** output renders via `cliclack::log::step()`

#### Scenario: `step!` degrades to debug in non-TTY
- **WHEN** `step!("Fetching providers")` is called and mode is non-TTY
- **THEN** output is logged at `debug` level via simplelog (visible with `-vv`)

### Requirement: Prelude provides unified imports
The system SHALL provide `src/prelude.rs` re-exporting all seven logging macros and `anyhow::{Context, Result, anyhow, bail}`. Files that add `use crate::prelude::*` SHALL NOT need any separate `use log::*` or individual anyhow imports.

#### Scenario: Prelude replaces log imports
- **WHEN** a file uses `use crate::prelude::*`
- **THEN** `warn!`, `info!`, `debug!`, `error!`, `trace!`, `success!`, `step!` are all in scope

#### Scenario: Prelude replaces anyhow imports
- **WHEN** a file uses `use crate::prelude::*`
- **THEN** `Context`, `Result`, `anyhow`, and `bail` are all in scope

### Requirement: Fatal errors close cleanly in TTY mode
The system SHALL call `cliclack::outro_cancel()` after displaying a fatal error when in TTY mode, leaving the terminal in a clean cliclack-closed state before `process::exit(1)`.

#### Scenario: Fatal error in TTY shows outro
- **WHEN** `main.rs` receives an `Err` result and mode is TTY
- **THEN** `display_error()` renders via `cliclack::log::error()` followed by `outro_cancel()`

#### Scenario: Fatal error in non-TTY uses simplelog
- **WHEN** `main.rs` receives an `Err` result and mode is non-TTY
- **THEN** `display_error()` renders via simplelog `error` level only
