## ADDED Requirements

### Requirement: Gitignore confirmation uses a cliclack select prompt
When neither `--gitignore` nor `--no-gitignore` is passed and new paths need to be added, `dotagents deploy` SHALL display a cliclack `select` prompt with two options — `Yes` and `No` — instead of a plain text `[y/N]` line.

#### Scenario: Select prompt shown with Yes/No options
- **WHEN** deploy has new paths to add and runs in a TTY with no gitignore flags
- **THEN** a cliclack select prompt is displayed with `Yes` and `No` as the only options

#### Scenario: User selects Yes — update runs
- **WHEN** the user navigates to `Yes` and presses Enter
- **THEN** the gitignore update step runs and the new paths are written

#### Scenario: User selects No — update skipped
- **WHEN** the user navigates to `No` and presses Enter
- **THEN** `.gitignore` is not modified

#### Scenario: Default selection is No
- **WHEN** the select prompt is displayed
- **THEN** `No` is highlighted as the default option

### Requirement: Raw crossterm prompt is replaced entirely
The previous implementation that used raw crossterm mode and a single `y`/`Y` keypress SHALL be removed. All gitignore confirmation UI SHALL go through cliclack.

#### Scenario: No raw mode code path for gitignore
- **WHEN** deploy shows the gitignore confirmation in a TTY
- **THEN** the output is a cliclack select widget, not a `[y/N]` text line

### Requirement: TUI prompt lives in src/cli/ui/deploy.rs
The gitignore confirmation prompt function SHALL reside in `src/cli/ui/deploy.rs`, not in `src/utils/gitignore.rs`. Utility functions for reading and writing `.gitignore` content SHALL remain in `src/utils/gitignore.rs`.

#### Scenario: Module boundary respected
- **WHEN** the deploy command needs to prompt for gitignore
- **THEN** it calls into `crate::cli::ui::deploy` for the prompt and `crate::utils::gitignore` for file operations
