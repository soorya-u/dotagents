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
`dotagents commands ls` SHALL read commands from `.dotagents/commands/*.md`, parse frontmatter for `name`, `description`, `category`, and `tags`, and display them using cliclack output. Descriptions SHALL be truncated to fit terminal width by default.

When `--json` is passed, the command SHALL output a JSON array of command objects containing frontmatter fields (name, description, category, tags). Body content SHALL NOT be included in JSON output unless `--full` is also passed. All log/warning output SHALL go to stderr.

When `--full` is passed, the command SHALL include the full markdown body content of each command after the frontmatter fields. Without `--full`, only name and frontmatter fields are shown. When both `--json` and `--full` are passed, each JSON object SHALL include a `content` key with the raw markdown body string in addition to the frontmatter fields.

#### Scenario: Commands listed
- **WHEN** user runs `dotagents commands ls`
- **THEN** all commands found in `.dotagents/commands/` are listed with names and truncated descriptions

#### Scenario: Empty workspace
- **WHEN** `.dotagents/commands/` is empty or absent
- **THEN** a message indicating no commands were found is displayed and the command exits 0

#### Scenario: Missing workspace exits with error
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`

#### Scenario: --full shows complete descriptions and body content
- **WHEN** user runs `dotagents commands ls --full`
- **THEN** each command's full description is shown, word-wrapped at terminal width, followed by the full markdown body content

#### Scenario: --json outputs commands as JSON array
- **WHEN** user runs `dotagents commands ls --json`
- **THEN** stdout contains a JSON array of command objects with frontmatter fields (name, description, category, tags); body content is absent; stderr contains any log messages

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** user runs `dotagents commands ls --json` with no commands present
- **THEN** stdout contains `[]` and the command exits 0

#### Scenario: Without --full, body content is omitted
- **WHEN** user runs `dotagents commands ls` without `--full`
- **THEN** only the command name and frontmatter metadata (description, category, tags) are shown; body content is not displayed

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
