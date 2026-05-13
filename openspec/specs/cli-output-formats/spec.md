## Purpose

Specifies the `--json` and `--content` output format flags available on read-only list commands (`commands ls`, `skills ls`, and future list commands), along with the standard text rendering behaviour for those commands.

## Requirements

### Requirement: --json flag outputs list as structured JSON
When `--json` is passed to a read-only list command (`commands ls`, `skills ls`), the command SHALL serialize each item's frontmatter fields and output the collection as a JSON array on stdout. Body content SHALL NOT be included unless `--content` is also passed. All log/warning output SHALL go to stderr.

#### Scenario: commands ls --json outputs frontmatter data
- **WHEN** `dotagents commands ls --json` is run in a workspace with two commands
- **THEN** stdout contains a JSON array with two objects, each containing the command's frontmatter fields (name, description, category, tags) without body content

#### Scenario: skills ls --json outputs frontmatter data
- **WHEN** `dotagents skills ls --json` is run in a workspace with one skill
- **THEN** stdout contains a JSON array with one object containing the skill's frontmatter fields without body content

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** `dotagents commands ls --json` is run with no commands present
- **THEN** stdout contains `[]` and the command exits 0

### Requirement: --content flag includes body content in output
When `--content` is passed to `commands ls` or `skills ls`, the command SHALL include the full markdown body content of each item in the output, in addition to the frontmatter fields already shown. Without `--content`, only the name and frontmatter fields SHALL be shown.

In TTY mode, `--content` renders each item that has body content using a `cliclack::note` block: the note header shows the item name styled **green+bold** (via `console::style`) followed by ` — ` and the description; the note body contains the raw markdown content. No separate `info!` row is printed for items rendered as a note. Items with no body content fall back to the standard `info!` row with the name in **cyan+bold**.

In non-TTY mode, `--content` appends body lines indented below the name-description row; the name is not styled (no ANSI output in non-TTY).

#### Scenario: --content shows command body content in TTY mode
- **WHEN** `dotagents commands ls --content` is run in a TTY
- **THEN** each command is rendered as a `cliclack::note` block with `name — description` as the note header and the full markdown body inside the note box

#### Scenario: --content shows skill body content in non-TTY mode
- **WHEN** `dotagents skills ls --content` is run in non-TTY mode
- **THEN** each skill's name and description are printed followed by the full markdown body content indented below

#### Scenario: Without --content, body content is omitted
- **WHEN** `dotagents commands ls` is run without `--content`
- **THEN** only name and description (truncated to fit terminal width) are displayed; body content is not shown

### Requirement: --json and --content are independent and can be combined
The `--json` and `--content` flags SHALL be independent and combinable. When both are passed, the JSON output SHALL include frontmatter fields plus a `content` key containing the raw markdown body string. When `--json` is not passed but `--content` is, body content SHALL be included in the human-readable text output.

#### Scenario: --json --content outputs JSON with body content
- **WHEN** `dotagents commands ls --json --content` is run
- **THEN** stdout contains a JSON array where each object includes frontmatter fields and a `content` key with the raw markdown body string

#### Scenario: --json alone without --content omits body content
- **WHEN** `dotagents commands ls --json` is run without `--content`
- **THEN** stdout contains a JSON array where each object includes frontmatter fields only; the `content` key is absent

#### Scenario: --content without --json shows body in text output
- **WHEN** `dotagents commands ls --content` is run without `--json`
- **THEN** the text output includes the full body content of each command

### Requirement: Text listing uses styled name and separator
In TTY mode, the item name in standard `info!` rows (i.e. not rendered as a `cliclack::note`) SHALL be rendered in **cyan+bold** using `console::style`. Note headers (TTY + `--content` + non-empty body) use **green+bold** instead, as described in the `--content` requirement above. The name and description SHALL be separated by ` — `. Column width SHALL match the actual longest name in the result set with no artificial minimum padding.

#### Scenario: Name is styled and separated from description
- **WHEN** `dotagents commands ls` is run in a TTY
- **THEN** each row renders as `{cyan+bold name} — {description}` with no leading indent and no section header

#### Scenario: Column width fits the actual longest name
- **WHEN** the longest command name is shorter than 10 characters
- **THEN** there is no extra padding between the name and ` — `

### Requirement: No intro header and no section count header
List commands SHALL NOT print a `cliclack::intro` header or a `Commands (N)` / `Skills (N)` section header. The count SHALL appear only in the `outro` footer line.

#### Scenario: No intro or section header in output
- **WHEN** `dotagents skills ls` is run
- **THEN** the output does not contain an intro line or a `Skills (N)` header; the count appears only in the outro

### Requirement: JSON output is valid and parseable
JSON output from `--json` SHALL be valid JSON that can be piped to tools like `jq`. Each object SHALL include at least a stable identifier field (e.g., `name`). The `content` field (raw body string) SHALL be included only when `--content` is also passed. The output SHALL NOT include any non-JSON text on stdout.

#### Scenario: JSON output is pipeable to jq
- **WHEN** `dotagents commands ls --json | jq '.[0].name'` is run
- **THEN** `jq` successfully parses the output and extracts the first command's name

### Requirement: Flag convention applies to future read-only list commands
Any future read-only list command (e.g., `providers ls`) SHALL also support `--json` and `--content` flags with the same semantics.

#### Scenario: Future read-only command supports --json and --content
- **WHEN** a new read-only list command is added (e.g., `providers ls`)
- **THEN** it accepts `--json` and `--content` flags consistent with the pattern established here
