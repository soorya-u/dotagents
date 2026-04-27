## ADDED Requirements

### Requirement: Add command creates a new command source file
`dotagents add command <name>` SHALL create `.dotagents/commands/<name>.md` with YAML frontmatter (`name`, `description`, `category`, `tags`) and a fixed starter body template with `<name>` interpolated.

#### Scenario: File created with frontmatter from flags
- **WHEN** user runs `dotagents add command hello --description "Greets the user" --category Utility --tags greet,hello`
- **THEN** `.dotagents/commands/hello.md` is created with the provided frontmatter values and the command starter body

#### Scenario: Missing flags prompt in TTY mode
- **WHEN** user runs `dotagents add command hello` with no flags in a TTY
- **THEN** cliclack prompts for each missing field (description, category, tags) before creating the file

#### Scenario: Missing flags use empty defaults in non-TTY
- **WHEN** user runs `dotagents add command hello` with no flags and stdin is not a TTY
- **THEN** missing fields default to empty strings and the file is created without prompting

#### Scenario: File already exists errors without --force
- **WHEN** `.dotagents/commands/<name>.md` already exists and `--force` is not passed
- **THEN** the command exits 1 with an error indicating the file exists

#### Scenario: File already exists is overwritten with --force
- **WHEN** `.dotagents/commands/<name>.md` already exists and `--force` is passed
- **THEN** the file is overwritten with new frontmatter and the starter body

### Requirement: Add skill creates a new skill source directory and file
`dotagents add skill <name>` SHALL create `.dotagents/skills/<name>/SKILL.md` with YAML frontmatter (`name`, `description`, `license`, `compatibility`, `metadata.version`) and a fixed starter body template with `<name>` interpolated.

#### Scenario: Skill directory and file created with flags
- **WHEN** user runs `dotagents add skill my-skill --description "Does something" --license MIT`
- **THEN** `.dotagents/skills/my-skill/SKILL.md` is created with provided frontmatter and the skill starter body

#### Scenario: Missing flags prompt in TTY mode
- **WHEN** user runs `dotagents add skill my-skill` with no flags in a TTY
- **THEN** cliclack prompts for description, license, and compatibility before creating the file

#### Scenario: Skill already exists errors without --force
- **WHEN** `.dotagents/skills/<name>/SKILL.md` already exists and `--force` is not passed
- **THEN** the command exits 1 with an error indicating the skill already exists

### Requirement: Add supports --deploy flag and TTY deploy confirm
After creating the file, `add` SHALL optionally trigger a deploy.

#### Scenario: --deploy flag triggers deploy immediately
- **WHEN** `--deploy` is passed
- **THEN** `dotagents deploy` runs automatically after file creation, without prompting

#### Scenario: TTY confirm shown when --deploy is absent
- **WHEN** no `--deploy` flag is passed and stdin is a TTY
- **THEN** a cliclack confirm "Deploy now?" (default: No) is shown after file creation

#### Scenario: No deploy in non-TTY without --deploy
- **WHEN** no `--deploy` flag is passed and stdin is not a TTY
- **THEN** deploy is skipped silently after file creation

### Requirement: Starter body templates are fixed and name-interpolated
The body written after the frontmatter SHALL use a fixed template string with the provided `<name>` substituted literally — no Handlebars rendering, no user input.

#### Scenario: Command starter body contains name
- **WHEN** `dotagents add command my-cmd` is run
- **THEN** the body of `.dotagents/commands/my-cmd.md` begins with `# my-cmd`

#### Scenario: Skill starter body contains name
- **WHEN** `dotagents add skill my-skill` is run
- **THEN** the body of `.dotagents/skills/my-skill/SKILL.md` begins with `# my-skill`
