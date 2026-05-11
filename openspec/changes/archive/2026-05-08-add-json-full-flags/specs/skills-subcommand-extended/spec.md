## MODIFIED Requirements

### Requirement: skills ls lists local skill source directories
`dotagents skills ls` SHALL read skills from `.dotagents/skills/*/SKILL.md`, parse frontmatter for `name`, `description`, `license`, and `compatibility`, and display them using cliclack output. Descriptions SHALL be truncated to fit terminal width by default. The output SHALL NOT print a `cliclack::intro` header or a `Skills (N)` section header; the count appears only in the `outro`. In TTY mode, each name SHALL be rendered in cyan+bold with a ` — ` separator before the description; column width SHALL match the actual longest name with no artificial minimum.

When `--json` is passed, the command SHALL output a JSON array of skill objects containing frontmatter fields (name, description, license, compatibility). Body content SHALL NOT be included in JSON output unless `--content` is also passed. All log/warning output SHALL go to stderr.

When `--content` is passed, the command SHALL include the full markdown body content of each skill. In TTY mode, each skill is rendered as a `cliclack::note` block with `name — description` as the note header (name styled green+bold) and the body inside the note box; no separate `info!` line is printed for that item. In non-TTY mode, body lines are printed indented below the name-description row. Without `--content`, only name and description are shown. When both `--json` and `--content` are passed, each JSON object SHALL include a `content` key with the raw markdown body string.

When `--skill <name>` is passed, the listing SHALL be filtered to only the skill whose name exactly matches `<name>`. If no skill matches, an empty list is shown.

#### Scenario: Skills listed
- **WHEN** user runs `dotagents skills ls`
- **THEN** all skills found in `.dotagents/skills/` are listed with names and truncated descriptions

#### Scenario: Empty workspace
- **WHEN** `.dotagents/skills/` is empty or absent
- **THEN** a message indicating no skills were found is displayed and the command exits 0

#### Scenario: Missing workspace exits with error
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`

#### Scenario: --content shows complete descriptions and body content in TTY mode
- **WHEN** user runs `dotagents skills ls --content` in a TTY
- **THEN** each skill is rendered as a `cliclack::note` block with `name — description` as the note header and the full markdown body inside the note box

#### Scenario: --json outputs skills as JSON array
- **WHEN** user runs `dotagents skills ls --json`
- **THEN** stdout contains a JSON array of skill objects with frontmatter fields (name, description, license, compatibility); body content is absent; stderr contains any log messages

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** user runs `dotagents skills ls --json` with no skills present
- **THEN** stdout contains `[]` and the command exits 0

#### Scenario: Without --content, body content is omitted
- **WHEN** user runs `dotagents skills ls` without `--content`
- **THEN** only the skill name and frontmatter metadata (description, license, compatibility) are shown; body content is not displayed

#### Scenario: --skill filters listing to a single skill
- **WHEN** user runs `dotagents skills ls --skill my-skill`
- **THEN** only the skill named `my-skill` is shown; all other skills are excluded

#### Scenario: Text output shows styled name with separator
- **WHEN** user runs `dotagents skills ls` in a TTY
- **THEN** each row renders as `{cyan+bold name} — {truncated description}` with no intro header and no `Skills (N)` section header; the count appears only in the outro
