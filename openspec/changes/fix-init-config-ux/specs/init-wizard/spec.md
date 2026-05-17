## MODIFIED Requirements

### Requirement: --template does not bypass the interactive feature-selection wizard
When `dotagents init` is run in a TTY with `--template` but without `--features`, the CLI SHALL enter interactive TUI mode and display the feature-selection multiselect prompt. The `--template` flag SHALL pre-fill the template choice but SHALL NOT suppress the wizard.

#### Scenario: --template with TTY still shows feature prompt
- **WHEN** `dotagents init --template mycode` is run in a TTY
- **WHEN** `--features` is NOT passed
- **THEN** the interactive feature-selection multiselect is shown
- **THEN** the user can select features before init completes

#### Scenario: --template with --features skips prompts entirely
- **WHEN** `dotagents init --template mycode --features commands`
- **THEN** no prompts are shown (both template and features are already specified)

### Requirement: instructions feature is labelled "INSTRUCTIONS.md" in the wizard
The `instructions` item in the init wizard's feature-selection multiselect SHALL display the label `"INSTRUCTIONS.md"`.

#### Scenario: Init wizard shows correct label for instructions
- **WHEN** `dotagents init` is run in a TTY
- **THEN** the feature multiselect shows an item labelled `"INSTRUCTIONS.md"` for the instructions feature
- **THEN** the item is NOT labelled `"AGENTS.md"`
