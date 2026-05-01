## MODIFIED Requirements

### Requirement: Init wizard runs when no flags are given in a TTY
When `dotagents init` is invoked with no `--features` flag and no `--template` flag, and stdin is an interactive terminal, the CLI SHALL display an interactive cliclack prompt sequence before writing any files.

#### Scenario: Full wizard flow with no flags in TTY
- **WHEN** `dotagents init` is run with no flags in an interactive terminal
- **THEN** the wizard shows: intro header, feature multiselect, template select, and per-file log steps, then an outro message

#### Scenario: --features flag presence skips wizard
- **WHEN** `dotagents init --features commands` is run (any `--features` value or `--template` flag present)
- **THEN** no interactive prompts are shown and init proceeds immediately using the provided feature list

#### Scenario: Non-TTY skips wizard silently
- **WHEN** `dotagents init` is run with stdin not attached to a terminal (e.g. piped or CI)
- **THEN** no prompts are shown; init proceeds with all features enabled and Starter template

## REMOVED Requirements

### Requirement: Flag presence skips wizard (--no-* form)
**Reason**: The `--no-mcp`, `--no-command`, `--no-instruction`, and `--no-skill` boolean flags are removed and replaced by the `--features` whitelist flag. The "any flag skips wizard" rule is updated in the MODIFIED requirement above.
**Migration**: Replace `--no-mcp` with `--features commands,instructions,skills`. Replace `--no-command` with `--features instructions,mcp,skills`. Replace `--no-instruction` with `--features commands,mcp,skills`. Replace `--no-skill` with `--features commands,instructions,mcp`. To disable all features use `--features none`.
