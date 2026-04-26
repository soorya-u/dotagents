## ADDED Requirements

### Requirement: Offline mode prompt shown at start of deploy in TUI mode
When `dotagents deploy` runs in TUI mode (TTY, no `--offline` flag), it SHALL display a cliclack `select` prompt asking whether to run in offline mode before the registry-fetch step executes. The prompt SHALL default to `No` (online mode).

#### Scenario: User selects No — deploy runs online
- **WHEN** the offline prompt is shown and the user selects `No` or accepts the default
- **THEN** deploy proceeds with registry fetch enabled (identical to running without `--offline`)

#### Scenario: User selects Yes — deploy runs offline
- **WHEN** the offline prompt is shown and the user selects `Yes`
- **THEN** deploy skips the registry fetch, identical to passing `--offline`

#### Scenario: Default selection is No (online)
- **WHEN** the offline prompt is displayed
- **THEN** `No` is highlighted as the default option

### Requirement: --offline flag bypasses the prompt
When `--offline` is passed to `dotagents deploy`, the offline mode prompt SHALL NOT be shown and deploy SHALL run in offline mode directly.

#### Scenario: --offline flag skips prompt
- **WHEN** `dotagents deploy --offline` is run
- **THEN** no offline prompt is shown and the registry fetch is skipped

### Requirement: Non-TTY defaults to online without prompting
When deploy runs in a non-interactive environment (CI, piped), no offline prompt is shown and online mode is used unless `--offline` is explicitly passed.

#### Scenario: Non-TTY online deploy
- **WHEN** deploy runs with no TTY and no `--offline` flag
- **THEN** no prompt is shown and deploy proceeds online

#### Scenario: Non-TTY with --offline flag
- **WHEN** deploy runs with no TTY and `--offline` is passed
- **THEN** no prompt is shown and deploy proceeds offline

### Requirement: Offline prompt lives in src/cli/ui/deploy.rs
The offline mode prompt function SHALL reside in `src/cli/ui/deploy.rs` alongside the gitignore confirmation prompt.

#### Scenario: Module boundary respected
- **WHEN** the deploy command needs to prompt for offline mode
- **THEN** it calls into `crate::cli::ui::deploy` for the prompt before the registry-fetch block in `src/cli/deploy.rs`
