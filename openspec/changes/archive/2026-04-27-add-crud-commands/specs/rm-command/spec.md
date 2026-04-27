## ADDED Requirements

### Requirement: Remove command deletes a command source file
`dotagents rm command <name>` SHALL delete `.dotagents/commands/<name>.md`. If the file does not exist, the command SHALL exit 1 with a clear error.

#### Scenario: Existing command is removed
- **WHEN** user runs `dotagents rm command hello` and `.dotagents/commands/hello.md` exists
- **THEN** the file is deleted and a success message is shown

#### Scenario: Non-existent command errors
- **WHEN** user runs `dotagents rm command hello` and no such file exists
- **THEN** the command exits 1 with an error indicating the command was not found

### Requirement: Remove skill deletes the skill directory
`dotagents rm skill <name>` SHALL delete `.dotagents/skills/<name>/` and all its contents. If the directory does not exist, the command SHALL exit 1 with a clear error.

#### Scenario: Existing skill directory is removed
- **WHEN** user runs `dotagents rm skill my-skill` and `.dotagents/skills/my-skill/` exists
- **THEN** the directory and all contents are deleted and a success message is shown

#### Scenario: Non-existent skill errors
- **WHEN** user runs `dotagents rm skill my-skill` and no such directory exists
- **THEN** the command exits 1 with an error indicating the skill was not found

### Requirement: TTY removal prompts for confirmation
In a TTY without `--force`, `rm` SHALL show a cliclack confirm prompt before deletion to prevent accidental loss.

#### Scenario: Confirm shown in TTY
- **WHEN** user runs `dotagents rm command hello` in a TTY without `--force`
- **THEN** a cliclack confirm "Remove hello? This cannot be undone." is displayed before deletion

#### Scenario: Confirm declined aborts deletion
- **WHEN** user declines the confirm prompt
- **THEN** no file is deleted and the command exits 0

#### Scenario: --force skips confirm
- **WHEN** `--force` is passed
- **THEN** deletion proceeds immediately without any confirmation prompt

#### Scenario: Non-TTY skips confirm
- **WHEN** stdin is not a TTY (scripting context)
- **THEN** deletion proceeds without prompting regardless of `--force`

### Requirement: Remove supports --deploy flag and TTY deploy confirm
After deletion, `rm` SHALL optionally trigger a deploy using the same pattern as `add`.

#### Scenario: --deploy flag triggers deploy after removal
- **WHEN** `--deploy` is passed
- **THEN** `dotagents deploy` runs automatically after deletion

#### Scenario: TTY deploy confirm shown when --deploy is absent
- **WHEN** no `--deploy` flag and stdin is a TTY
- **THEN** a cliclack confirm "Deploy now?" (default: No) is shown after deletion

#### Scenario: No deploy in non-TTY without --deploy
- **WHEN** no `--deploy` flag and stdin is not a TTY
- **THEN** deploy is skipped silently
