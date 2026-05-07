## MODIFIED Requirements

### Requirement: skills ls lists local skill source directories
`dotagents skills ls` SHALL read skills from `.dotagents/skills/*/SKILL.md`, parse frontmatter for `name`, `description`, `license`, and `compatibility`, and display them using cliclack output. Descriptions SHALL be truncated to fit terminal width by default.

When `--json` is passed, the command SHALL output a JSON array of skill objects containing frontmatter fields (name, description, license, compatibility). Body content SHALL NOT be included in JSON output unless `--full` is also passed. All log/warning output SHALL go to stderr.

When `--full` is passed, the command SHALL include the full markdown body content of each skill after the frontmatter fields. Without `--full`, only name and frontmatter fields are shown. When both `--json` and `--full` are passed, each JSON object SHALL include a `content` key with the raw markdown body string in addition to the frontmatter fields.

#### Scenario: Skills listed
- **WHEN** user runs `dotagents skills ls`
- **THEN** all skills found in `.dotagents/skills/` are listed with names and truncated descriptions

#### Scenario: Empty workspace
- **WHEN** `.dotagents/skills/` is empty or absent
- **THEN** a message indicating no skills were found is displayed and the command exits 0

#### Scenario: Missing workspace exits with error
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`

#### Scenario: --full shows complete descriptions and body content
- **WHEN** user runs `dotagents skills ls --full`
- **THEN** each skill's full description is shown, word-wrapped at terminal width, followed by the full markdown body content

#### Scenario: --json outputs skills as JSON array
- **WHEN** user runs `dotagents skills ls --json`
- **THEN** stdout contains a JSON array of skill objects with frontmatter fields (name, description, license, compatibility); body content is absent; stderr contains any log messages

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** user runs `dotagents skills ls --json` with no skills present
- **THEN** stdout contains `[]` and the command exits 0

#### Scenario: Without --full, body content is omitted
- **WHEN** user runs `dotagents skills ls` without `--full`
- **THEN** only the skill name and frontmatter metadata (description, license, compatibility) are shown; body content is not displayed
