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

### Requirement: Outro frames the wizard session on completion
The wizard SHALL end with a cliclack `outro` message that hints at running `dotagents deploy`. No `intro` banner is shown — the first interactive element the user sees SHALL be the feature multiselect prompt.

#### Scenario: No intro shown at start
- **WHEN** the wizard begins
- **THEN** the first visible output is the feature-selection prompt (no preceding intro banner)

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
