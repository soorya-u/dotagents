## ADDED Requirements

### Requirement: Selected features are persisted to config files
After the wizard completes, the features the user selected SHALL be written to both `config.toml` and `local.config.toml`. If `--features none` was used, `features = []` SHALL be written. The `variables` key in both files remains hardcoded and is not affected by feature selection.

#### Scenario: User selects a subset of features
- **WHEN** the user deselects `mcp` and `skills` in the feature multiselect
- **THEN** `config.toml` contains `features = ["commands", "instructions"]` and `local.config.toml` contains the same

#### Scenario: User selects no features (--features none headless)
- **WHEN** `dotagents init --features none` is run
- **THEN** both `config.toml` and `local.config.toml` contain `features = []`

#### Scenario: User accepts all feature defaults
- **WHEN** the user presses Enter without deselecting any feature
- **THEN** both config files contain `features = ["commands", "instructions", "mcp", "skills"]`

## MODIFIED Requirements

### Requirement: Intro and outro frame the wizard session
The wizard SHALL start with a cliclack `intro` banner and end with a cliclack `outro` message that hints at running `dotagents deploy`. The intro text SHALL NOT mirror the command the user typed; it SHALL use a short descriptive phrase (e.g. the app name `dotagents`).

#### Scenario: Intro shown at start
- **WHEN** the wizard begins
- **THEN** an intro line is printed before any prompts that does not repeat `dotagents init` verbatim

#### Scenario: Outro shown on success
- **WHEN** all files have been written and target selection is complete (or skipped)
- **THEN** an outro line is printed suggesting the user run `dotagents deploy`

### Requirement: Per-file log feedback during write
After all prompts are answered, the CLI SHALL print a `log::step` confirmation line for each file successfully written. Files that are skipped due to feature deselection or template variant SHALL NOT emit any visible output at the default verbosity level; they MAY appear at debug verbosity (`-v`).

#### Scenario: File write feedback
- **WHEN** init writes `commands/hello.md`
- **THEN** a step line is printed to stdout confirming the write

#### Scenario: Skipped file produces no output at default verbosity
- **WHEN** the user deselects `mcp` and init skips writing `mcp.jsonc`
- **THEN** no `Skipping mcp.jsonc` line appears in the terminal output at default verbosity

#### Scenario: Skipped file visible at debug verbosity
- **WHEN** the user passes `-v` and init skips writing `mcp.jsonc`
- **THEN** a debug-level message indicating the skip is printed
