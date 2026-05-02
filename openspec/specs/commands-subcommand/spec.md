## Purpose

Specifies the `dotagents commands` subcommand group, which provides domain-scoped CRUD operations for local command source files under `.dotagents/commands/`. Replaces the removed top-level `add command`, `rm command`, and `ls` commands.

## Requirements

### Requirement: commands new creates a new command source file
`dotagents commands new <name>` SHALL create `.dotagents/commands/<name>.md` with YAML frontmatter (`name`, `description`, `category`, `tags`) and a fixed starter body template with `<name>` interpolated. Behaviour is identical to the removed `dotagents add command`.

#### Scenario: File created with frontmatter from flags
- **WHEN** user runs `dotagents commands new hello --description "Greets the user" --category Utility --tags greet,hello`
- **THEN** `.dotagents/commands/hello.md` is created with the provided frontmatter values and the command starter body

#### Scenario: Missing flags prompt in TTY mode
- **WHEN** user runs `dotagents commands new hello` with no flags in a TTY
- **THEN** cliclack prompts for each missing field (description, category, tags) before creating the file

#### Scenario: Missing flags use empty defaults in non-TTY
- **WHEN** user runs `dotagents commands new hello` with no flags and stdin is not a TTY
- **THEN** missing fields default to empty strings and the file is created without prompting

#### Scenario: File already exists errors without --force
- **WHEN** `.dotagents/commands/<name>.md` already exists and `--force` is not passed
- **THEN** the command exits 1 with an error indicating the file exists

#### Scenario: File already exists is overwritten with --force
- **WHEN** `.dotagents/commands/<name>.md` already exists and `--force` is passed
- **THEN** the file is overwritten with new frontmatter and the starter body

### Requirement: commands rm deletes a command source file
`dotagents commands rm <name>` SHALL delete `.dotagents/commands/<name>.md`. If the file does not exist, the command SHALL exit 1 with a clear error. Behaviour is identical to the removed `dotagents rm command`.

#### Scenario: Existing command is removed
- **WHEN** user runs `dotagents commands rm hello` and `.dotagents/commands/hello.md` exists
- **THEN** the file is deleted and a success message is shown

#### Scenario: Non-existent command errors
- **WHEN** user runs `dotagents commands rm hello` and no such file exists
- **THEN** the command exits 1 with an error indicating the command was not found

#### Scenario: Confirm shown in TTY without --force
- **WHEN** user runs `dotagents commands rm hello` in a TTY without `--force`
- **THEN** a cliclack confirm prompt is displayed before deletion

#### Scenario: Confirm declined aborts deletion
- **WHEN** user declines the confirm prompt
- **THEN** no file is deleted and the command exits 0

#### Scenario: --force skips confirm
- **WHEN** `--force` is passed
- **THEN** deletion proceeds immediately without any confirmation prompt

#### Scenario: Non-TTY skips confirm
- **WHEN** stdin is not a TTY
- **THEN** deletion proceeds without prompting regardless of `--force`

### Requirement: commands ls lists command source files
`dotagents commands ls` SHALL read commands from `.dotagents/commands/*.md`, parse frontmatter for `name` and `description`, and display them using cliclack output. Descriptions SHALL be truncated to fit terminal width by default.

#### Scenario: Commands listed
- **WHEN** user runs `dotagents commands ls`
- **THEN** all commands found in `.dotagents/commands/` are listed with names and truncated descriptions

#### Scenario: Empty workspace
- **WHEN** `.dotagents/commands/` is empty or absent
- **THEN** a message indicating no commands were found is displayed and the command exits 0

#### Scenario: Missing workspace exits with error
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`

#### Scenario: --full shows complete descriptions
- **WHEN** user runs `dotagents commands ls --full`
- **THEN** each command's full description is shown, word-wrapped at terminal width

### Requirement: commands subcommand supports --deploy after new and rm
After creating or deleting a command, `commands new` and `commands rm` SHALL optionally trigger a deploy.

#### Scenario: --deploy flag triggers deploy immediately
- **WHEN** `--deploy` is passed to `commands new` or `commands rm`
- **THEN** `dotagents deploy` runs automatically after the operation, without prompting

#### Scenario: TTY confirm shown when --deploy is absent
- **WHEN** no `--deploy` flag is passed and stdin is a TTY
- **THEN** a cliclack confirm "Deploy now?" (default: No) is shown after the operation

#### Scenario: No deploy in non-TTY without --deploy
- **WHEN** no `--deploy` flag is passed and stdin is not a TTY
- **THEN** deploy is skipped silently
