## MODIFIED Requirements

### Requirement: Init wizard runs when no flags are given in a TTY
When `dotagents init` is invoked with no `--features` flag and no `--template` flag, and stdin is an interactive terminal, the CLI SHALL display an interactive cliclack prompt sequence before writing any files. No filesystem writes of any kind SHALL occur before the wizard completes and the user confirms intent to proceed.

#### Scenario: Full wizard flow with no flags in TTY
- **WHEN** `dotagents init` is run with no flags in an interactive terminal
- **THEN** the wizard shows: intro header, feature multiselect, template select, and per-file log steps, then an outro message

#### Scenario: --features flag presence skips wizard
- **WHEN** `dotagents init --features commands` is run (any `--features` value or `--template` flag present)
- **THEN** no interactive prompts are shown and init proceeds immediately using the provided feature list

#### Scenario: Non-TTY skips wizard silently
- **WHEN** `dotagents init` is run with stdin not attached to a terminal (e.g. piped or CI)
- **THEN** no prompts are shown; init proceeds with all features enabled and Starter template

#### Scenario: Wizard cancelled — no directory created
- **WHEN** `dotagents init` is run in an interactive terminal and the user cancels the wizard
- **THEN** init exits 0 and no directory or file has been written to disk
