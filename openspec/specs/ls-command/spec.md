## ADDED Requirements

### Requirement: List skills and commands from source directory
`dotagents ls` SHALL read skills from `.dotagents/skills/*/SKILL.md` and commands from `.dotagents/commands/*.md`, parse frontmatter for `name` and `description`, and display them grouped in two sections (Skills, Commands) using cliclack output.

#### Scenario: Both sections shown by default
- **WHEN** user runs `dotagents ls` with no flags
- **THEN** both a Skills section and a Commands section are rendered, each listing all items found in `.dotagents/`

#### Scenario: Filter to commands only
- **WHEN** user runs `dotagents ls --commands`
- **THEN** only the Commands section is rendered; Skills section is omitted

#### Scenario: Filter to skills only
- **WHEN** user runs `dotagents ls --skills`
- **THEN** only the Skills section is rendered; Commands section is omitted

#### Scenario: Both filter flags show both sections
- **WHEN** user runs `dotagents ls --commands --skills`
- **THEN** both sections are rendered (same as no flags)

#### Scenario: Empty section is omitted
- **WHEN** a section has zero items (e.g. no commands exist in `.dotagents/commands/`)
- **THEN** that section is not rendered and no empty header is shown

### Requirement: Descriptions truncated to terminal width by default
Description text displayed next to each item name SHALL be truncated to fit within the current terminal width. Truncated descriptions SHALL end with `…`.

#### Scenario: Long description is truncated
- **WHEN** a description exceeds the available column width
- **THEN** it is cut at the available width and `…` is appended

#### Scenario: Short description is shown in full
- **WHEN** a description fits within the available column width
- **THEN** it is shown without modification

#### Scenario: Terminal width detection fails
- **WHEN** `crossterm::terminal::size()` returns an error
- **THEN** the display falls back to 80-column width

### Requirement: Verbose flag shows full descriptions
With `--verbose`, descriptions SHALL be shown in full, word-wrapped at terminal width, rather than truncated on a single line.

#### Scenario: Verbose shows complete text
- **WHEN** user runs `dotagents ls --verbose`
- **THEN** each item's full description is shown, wrapped across multiple lines if needed

### Requirement: No items found exits cleanly
If no skills or commands exist in `.dotagents/` (after applying filters), the command SHALL print a message indicating nothing was found and exit 0.

#### Scenario: Empty workspace
- **WHEN** `.dotagents/skills/` and `.dotagents/commands/` are both empty or absent
- **THEN** a message like "No skills or commands found." is displayed and the command exits 0

### Requirement: Workspace not found produces actionable error
If no `.dotagents/` directory is found by walking parent directories, `ls` SHALL exit 1 with an error message directing the user to run `dotagents init`.

#### Scenario: Missing workspace
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`
