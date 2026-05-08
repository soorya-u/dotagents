## ADDED Requirements

### Requirement: --json flag outputs list as structured JSON
When `--json` is passed to a read-only list command (`commands ls`, `skills ls`), the command SHALL serialize each item's frontmatter fields and output the collection as a JSON array on stdout. Body content SHALL NOT be included unless `--full` is also passed. All log/warning output SHALL go to stderr.

#### Scenario: commands ls --json outputs frontmatter data
- **WHEN** `dotagents commands ls --json` is run in a workspace with two commands
- **THEN** stdout contains a JSON array with two objects, each containing the command's frontmatter fields (name, description, category, tags) without body content

#### Scenario: skills ls --json outputs frontmatter data
- **WHEN** `dotagents skills ls --json` is run in a workspace with one skill
- **THEN** stdout contains a JSON array with one object containing the skill's frontmatter fields without body content

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** `dotagents commands ls --json` is run with no commands present
- **THEN** stdout contains `[]` and the command exits 0

### Requirement: --full flag includes body content in output
When `--full` is passed to `commands ls` or `skills ls`, the command SHALL include the full markdown body content of each item in the output, in addition to the frontmatter fields already shown. Without `--full`, only the name and frontmatter fields SHALL be shown (current behavior).

#### Scenario: --full shows command body content in CLI mode
- **WHEN** `dotagents commands ls --full` is run in non-TTY mode
- **THEN** each command's full markdown body content is displayed after its name and frontmatter fields

#### Scenario: --full shows skill body content in CLI mode
- **WHEN** `dotagents skills ls --full` is run in non-TTY mode
- **THEN** each skill's full markdown body content is displayed after its name and frontmatter fields

#### Scenario: Without --full, body content is omitted
- **WHEN** `dotagents commands ls` is run without `--full`
- **THEN** only name and frontmatter fields (description, category, tags) are displayed; body content is not shown

### Requirement: --json and --full are independent and can be combined
The `--json` and `--full` flags SHALL be independent and combinable. When both are passed, the JSON output SHALL include frontmatter fields plus a `content` key containing the raw markdown body string. When `--json` is not passed but `--full` is, body content SHALL be included in the human-readable text output.

#### Scenario: --json --full outputs JSON with body content
- **WHEN** `dotagents commands ls --json --full` is run
- **THEN** stdout contains a JSON array where each object includes frontmatter fields and a `content` key with the raw markdown body string

#### Scenario: --json alone without --full omits body content
- **WHEN** `dotagents commands ls --json` is run without `--full`
- **THEN** stdout contains a JSON array where each object includes frontmatter fields only; the `content` key is absent

#### Scenario: --full without --json shows body in text output
- **WHEN** `dotagents commands ls --full` is run without `--json`
- **THEN** the text output includes the full body content of each command after the frontmatter

### Requirement: JSON output is valid and parseable
JSON output from `--json` SHALL be valid JSON that can be piped to tools like `jq`. Each object SHALL include at least a stable identifier field (e.g., `name` or `slug`). The `content` field (raw body string) SHALL be included only when `--full` is also passed. For commands that have no body (e.g., `providers ls`), `content` is never present regardless of `--full`. The output SHALL NOT include any non-JSON text (e.g., status messages, warnings) on stdout.

#### Scenario: JSON output is pipeable to jq
- **WHEN** `dotagents commands ls --json | jq '.[0].name'` is run
- **THEN** `jq` successfully parses the output and extracts the first command's name

### Requirement: Flag convention applies to future read-only list commands
Any future read-only list command (e.g., `providers ls`) SHALL also support `--json` and `--full` flags with the same semantics: `--json` for machine-readable JSON output via `to_value()`, and `--full` for verbose human-readable output including full content.

#### Scenario: Future read-only command supports --json and --full
- **WHEN** a new read-only list command is added (e.g., `providers ls`)
- **THEN** it accepts `--json` and `--full` flags consistent with the pattern established here
