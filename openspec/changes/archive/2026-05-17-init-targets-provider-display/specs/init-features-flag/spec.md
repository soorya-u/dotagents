## REMOVED Requirements

### Requirement: --features none disables all feature scaffolding
**Reason**: The `Feature::None` sentinel was a workaround before `--ci` existed. With `--ci`, users can run `dotagents --ci init` without `--features` to get no features. The sentinel adds unnecessary complexity.
**Migration**: Use `dotagents --ci init` (no `--features` flag) instead of `dotagents init --features none`.

### Requirement: --features with no values is an error
**Reason**: The validation logic in `validate_features()` was primarily for checking `None` sentinel mixing. With `Feature::None` removed, clap's built-in `num_args = 1..` validation handles the empty-value case.
**Migration**: Clap will reject `--features` with no values at the argument-parsing level.

## MODIFIED Requirements

### Requirement: --features flag selects which features init scaffolds
`dotagents init` SHALL accept an optional `--features` flag that takes a comma-separated list of feature names and/or may be repeated. Valid feature names are `commands`, `instructions`, `mcp`, and `skills`. When `--features` is absent and TUI is available, the wizard SHALL prompt for features. When `--features` is absent and TUI is not available (CI/non-TTY), no features SHALL be scaffolded.

#### Scenario: Single feature via comma-separated value
- **WHEN** user runs `dotagents init --features commands,mcp`
- **THEN** only `commands` and `mcp` feature files are scaffolded; `instructions` and `skills` files are omitted

#### Scenario: Multiple features via repeated flag
- **WHEN** user runs `dotagents init --features commands --features mcp`
- **THEN** only `commands` and `mcp` feature files are scaffolded; same result as comma-separated form

#### Scenario: No features scaffolded when flag is absent in non-TUI mode
- **WHEN** user runs `dotagents --ci init` with no `--features` flag
- **THEN** no feature files (commands, instructions, mcp, skills) are scaffolded; only base config files are written

#### Scenario: --features skips only the feature prompt in TUI
- **WHEN** user runs `dotagents init --features commands` in an interactive terminal without `--template` or `--targets`
- **THEN** the template and target wizard prompts are shown, but the feature selection prompt is skipped
- **THEN** only `commands` feature files are scaffolded

### Requirement: --features presence skips the TUI feature selection screen
When `--features` is explicitly provided, the init TUI wizard SHALL skip only the feature selection prompt. Other wizard prompts (template, targets) SHALL still be shown if their corresponding flags are absent.

#### Scenario: --features skips feature prompt but shows others
- **WHEN** user runs `dotagents init --features commands` in an interactive terminal
- **THEN** the feature multiselect is NOT shown; the template select and target selection prompts ARE shown

#### Scenario: Absent --features allows feature prompt to run
- **WHEN** user runs `dotagents init` with no flags in an interactive terminal
- **THEN** the TUI wizard runs including the feature multiselect screen
