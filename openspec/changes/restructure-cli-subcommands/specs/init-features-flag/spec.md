## ADDED Requirements

### Requirement: --features flag selects which features init scaffolds
`dotagents init` SHALL accept an optional `--features` flag that takes a comma-separated list of feature names and/or may be repeated. Valid feature names are `commands`, `instructions`, `mcp`, `skills`, and the sentinel `none`. When `--features` is absent, all features are enabled by default.

#### Scenario: Single feature via comma-separated value
- **WHEN** user runs `dotagents init --features commands,mcp`
- **THEN** only `commands` and `mcp` feature files are scaffolded; `instructions` and `skills` files are omitted

#### Scenario: Multiple features via repeated flag
- **WHEN** user runs `dotagents init --features commands --features mcp`
- **THEN** only `commands` and `mcp` feature files are scaffolded; same result as comma-separated form

#### Scenario: All features enabled when flag is absent
- **WHEN** user runs `dotagents init` with no `--features` flag
- **THEN** all four feature sets (commands, instructions, mcp, skills) are scaffolded

### Requirement: --features none disables all feature scaffolding
The sentinel value `none` SHALL be accepted as the sole value for `--features` and SHALL cause no feature files to be written. Only the base config files (`config.toml`, `local.config.toml`, `.env`, `.gitignore`) SHALL be written.

#### Scenario: none disables all features
- **WHEN** user runs `dotagents init --features none`
- **THEN** no command, instruction, mcp, or skill files are written
- **THEN** `config.toml`, `local.config.toml`, `.env`, and `.gitignore` are still written

#### Scenario: none combined with other values is an error
- **WHEN** user runs `dotagents init --features none,commands`
- **THEN** the command exits 1 with an error message stating that `none` cannot be combined with other feature names

### Requirement: --features with no values is an error
Passing `--features` without any value SHALL cause the command to exit 1 with a clear error.

#### Scenario: Empty --features errors
- **WHEN** user runs `dotagents init --features` with no value following the flag
- **THEN** the command exits 1 with an error indicating a value is required

### Requirement: --features presence skips the TUI feature selection screen
When `--features` is explicitly provided, the `init` TUI wizard SHALL be bypassed entirely (same as other headless flags).

#### Scenario: --features bypasses TUI
- **WHEN** user runs `dotagents init --features commands` in an interactive terminal
- **THEN** no TUI prompts are shown and init proceeds headlessly with only command files scaffolded

#### Scenario: Absent --features allows TUI to run
- **WHEN** user runs `dotagents init` with no flags in an interactive terminal
- **THEN** the TUI wizard runs including the feature multiselect screen
