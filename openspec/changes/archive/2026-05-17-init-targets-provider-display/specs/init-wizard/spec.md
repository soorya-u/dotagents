## MODIFIED Requirements

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
