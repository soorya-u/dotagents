## MODIFIED Requirements

### Requirement: Default mode prompts the user interactively
When neither `--gitignore` nor `--no-gitignore` is passed, `dotagents deploy` SHALL prompt the user after deploy completes with the count of new paths using a cliclack `select` prompt with `Yes` and `No` options. Default selection is `No`.

#### Scenario: User selects Yes — update runs
- **WHEN** the cliclack select prompt is shown and the user selects `Yes`
- **THEN** the gitignore update step runs

#### Scenario: User selects No or accepts default — update skipped
- **WHEN** the cliclack select prompt is shown and the user selects `No` or presses Enter on the default
- **THEN** `.gitignore` is not modified

#### Scenario: No new paths to add — prompt is skipped
- **WHEN** all collected target paths are already present in the fenced section
- **THEN** no prompt is shown
