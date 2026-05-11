## MODIFIED Requirements

### Requirement: commands ls lists command source files
`dotagents commands ls` SHALL read commands from `.dotagents/commands/*.md`, parse frontmatter for `name`, `description`, `category`, and `tags`, and display them using cliclack output. Descriptions SHALL be truncated to fit terminal width by default. The output SHALL NOT print a `cliclack::intro` header or a `Commands (N)` section header; the count appears only in the `outro`. In TTY mode, each name SHALL be rendered in cyan+bold with a ` — ` separator before the description; column width SHALL match the actual longest name with no artificial minimum.

When `--json` is passed, the command SHALL output a JSON array of command objects containing frontmatter fields (name, description, category, tags). Body content SHALL NOT be included in JSON output unless `--content` is also passed. All log/warning output SHALL go to stderr.

When `--content` is passed, the command SHALL include the full markdown body content of each command. In TTY mode, each command is rendered as a `cliclack::note` block with `name — description` as the note header (name styled green+bold) and the body inside the note box; no separate `info!` line is printed for that item. In non-TTY mode, body lines are printed indented below the name-description row. Without `--content`, only name and description are shown. When both `--json` and `--content` are passed, each JSON object SHALL include a `content` key with the raw markdown body string.

When `--command <name>` is passed, the listing SHALL be filtered to only the command whose name exactly matches `<name>`. If no command matches, an empty list is shown.

#### Scenario: Commands listed
- **WHEN** user runs `dotagents commands ls`
- **THEN** all commands found in `.dotagents/commands/` are listed with names and truncated descriptions

#### Scenario: Empty workspace
- **WHEN** `.dotagents/commands/` is empty or absent
- **THEN** a message indicating no commands were found is displayed and the command exits 0

#### Scenario: Missing workspace exits with error
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`

#### Scenario: --content shows complete descriptions and body content in TTY mode
- **WHEN** user runs `dotagents commands ls --content` in a TTY
- **THEN** each command is rendered as a `cliclack::note` block with `name — description` as the note header and the full markdown body inside the note box

#### Scenario: --json outputs commands as JSON array
- **WHEN** user runs `dotagents commands ls --json`
- **THEN** stdout contains a JSON array of command objects with frontmatter fields (name, description, category, tags); body content is absent; stderr contains any log messages

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** user runs `dotagents commands ls --json` with no commands present
- **THEN** stdout contains `[]` and the command exits 0

#### Scenario: Without --content, body content is omitted
- **WHEN** user runs `dotagents commands ls` without `--content`
- **THEN** only the command name and frontmatter metadata (description, category, tags) are shown; body content is not displayed

#### Scenario: --command filters listing to a single command
- **WHEN** user runs `dotagents commands ls --command hello`
- **THEN** only the command named `hello` is shown; all other commands are excluded

#### Scenario: Text output shows styled name with separator
- **WHEN** user runs `dotagents commands ls` in a TTY
- **THEN** each row renders as `{cyan+bold name} — {truncated description}` with no intro header and no `Commands (N)` section header; the count appears only in the outro
