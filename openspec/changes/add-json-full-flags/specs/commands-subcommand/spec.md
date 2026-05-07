## MODIFIED Requirements

### Requirement: commands ls lists command source files
`dotagents commands ls` SHALL read commands from `.dotagents/commands/*.md`, parse frontmatter for `name`, `description`, `category`, and `tags`, and display them using cliclack output. Descriptions SHALL be truncated to fit terminal width by default.

When `--json` is passed, the command SHALL output a JSON array of command objects using each command's `to_value()` representation (includes name, frontmatter fields, and body content). All log/warning output SHALL go to stderr.

When `--full` is passed, the command SHALL include the full markdown body content of each command after the frontmatter fields. Without `--full`, only name and frontmatter fields are shown. When both `--json` and `--full` are passed, the `--json` flag takes precedence: JSON output is produced using `to_value()` which already includes body content natively, and `--full` is effectively a no-op in JSON mode.

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
- **THEN** stdout contains a JSON array of command objects with name, frontmatter fields, and body content; stderr contains any log messages

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** user runs `dotagents commands ls --json` with no commands present
- **THEN** stdout contains `[]` and the command exits 0

#### Scenario: Without --full, body content is omitted
- **WHEN** user runs `dotagents commands ls` without `--full`
- **THEN** only the command name and frontmatter metadata (description, category, tags) are shown; body content is not displayed
