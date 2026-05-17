# Init Wizard

## Purpose

Specifies the interactive TUI wizard that runs when `dotagents init` is invoked in an interactive terminal, including the prompt sequence, conditional prompt skipping, and cancellation behaviour.

## Requirements

### Requirement: Init wizard runs when no flags are given in a TTY
When `dotagents init` is invoked in an interactive terminal (TUI enabled), the CLI SHALL display the interactive wizard. The wizard SHALL conditionally skip individual prompts when their corresponding flag is provided: `--features` skips the feature multiselect, `--template` skips the template select, `--targets` skips the provider target multiselect. When no flags are given, all prompts are shown.

#### Scenario: Full wizard flow with no flags in TTY
- **WHEN** `dotagents init` is run with no flags in an interactive terminal
- **THEN** the wizard shows: intro header, feature multiselect, template select, target multiselect, and per-file log steps, then an outro message

#### Scenario: Partial wizard when --features is provided
- **WHEN** `dotagents init --features commands` is run in an interactive terminal
- **THEN** the feature multiselect is skipped; the template select and target multiselect are shown

#### Scenario: Partial wizard when --targets is provided
- **WHEN** `dotagents init --targets claude` is run in an interactive terminal
- **THEN** the target multiselect is skipped; the feature multiselect and template select are shown

#### Scenario: Partial wizard when --template is provided
- **WHEN** `dotagents init --template starter` is run in an interactive terminal
- **THEN** the template select is skipped; the feature multiselect and target multiselect are shown

#### Scenario: All flags provided skips all prompts but still runs in TUI mode
- **WHEN** `dotagents init --features commands --template starter --targets claude` is run in an interactive terminal
- **THEN** no wizard prompts are shown; the TUI outro message is still displayed

#### Scenario: Non-TTY skips wizard silently
- **WHEN** `dotagents init` is run with stdin not attached to a terminal (e.g. piped or CI)
- **THEN** no prompts are shown; init proceeds with empty features, default template, and empty targets

#### Scenario: Wizard cancelled — no directory created
- **WHEN** `dotagents init` is run in an interactive terminal and the user cancels the wizard
- **THEN** init exits 0 and no directory or file has been written to disk

### Requirement: Feature multiselect defaults to all features enabled
The feature selection prompt SHALL present all four features (commands, instructions, mcp, skills) as a multiselect with all options pre-checked.

#### Scenario: User accepts defaults
- **WHEN** the user presses Enter without toggling any feature
- **THEN** all four features are enabled and all corresponding mock files are written

#### Scenario: User deselects one feature
- **WHEN** the user deselects `mcp` and confirms
- **THEN** `mcp.jsonc` is not written; all other feature files are written

### Requirement: Selected features are persisted to config files
After the wizard completes, the features the user selected SHALL be written to both `config.toml` and `local.config.toml`. If no features are selected, `features = []` SHALL be written. The `variables` key in both files remains hardcoded and is not affected by feature selection.

#### Scenario: User selects a subset of features
- **WHEN** the user deselects `mcp` and `skills` in the feature multiselect
- **THEN** `config.toml` contains `features = ["commands", "instructions"]` and `local.config.toml` contains the same

#### Scenario: User selects no features
- **WHEN** no features are selected (via deselecting all or via `--ci` mode)
- **THEN** both `config.toml` and `local.config.toml` contain `features = []`

#### Scenario: User accepts all feature defaults
- **WHEN** the user presses Enter without deselecting any feature
- **THEN** both config files contain `features = ["commands", "instructions", "mcp", "skills"]`

### Requirement: Overwrite confirmation replaces --force in interactive mode
When `.dotagents/` already exists and no `--force` flag is given, the wizard SHALL ask for confirmation before overwriting. If the user declines, init SHALL exit 0 without writing any files.

#### Scenario: Existing directory — user confirms overwrite
- **WHEN** `.dotagents/` exists, no `--force` flag is passed, and the user selects Yes
- **THEN** the directory is removed and re-created with new files

#### Scenario: Existing directory — user declines
- **WHEN** `.dotagents/` exists, no `--force` flag is passed, and the user selects No
- **THEN** init exits 0 and `.dotagents/` is not modified

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

### Requirement: Registry target selection runs after file writes
After all scaffold files are written, the wizard SHALL fetch `registry.json` using `Registry::fetch(REGISTRY_URL)`, display a cliclack multiselect of all provider names from `registry.providers`, and write the user's selections as `targets = [...]` in the generated `config.toml` (global config only — `local.config.toml` is not modified). If the registry fetch fails, a warning is shown and the step is skipped; `targets` remains as the default empty value in `config.toml`.

#### Scenario: Registry fetched — user selects providers
- **WHEN** the registry is reachable and the user selects `claude`, `cursor`, and `codex`
- **THEN** `.dotagents/config.toml` contains `targets = ["claude", "cursor", "codex"]`

#### Scenario: User selects no providers
- **WHEN** the registry is reachable and the user confirms with no providers selected
- **THEN** `.dotagents/config.toml` contains `targets = []`

#### Scenario: Registry fetch fails — step skipped gracefully
- **WHEN** the registry is unreachable during init
- **THEN** a cliclack warning is printed, the target selection prompt is not shown, and `config.toml` is written with `targets = []`

### Requirement: Intro and outro frame the wizard session
The wizard SHALL start with a cliclack `intro` banner and end with a cliclack `outro` message that hints at running `dotagents deploy`. The intro text SHALL NOT mirror the command the user typed; it SHALL use a short descriptive phrase (e.g. the app name `dotagents`).

#### Scenario: Intro shown at start
- **WHEN** the wizard begins
- **THEN** an intro line is printed before any prompts that does not repeat `dotagents init` verbatim

#### Scenario: Outro shown on success
- **WHEN** all files have been written and target selection is complete (or skipped)
- **THEN** an outro line is printed suggesting the user run `dotagents deploy`
