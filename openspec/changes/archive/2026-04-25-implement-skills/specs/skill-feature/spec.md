## ADDED Requirements

### Requirement: Skill frontmatter schema
The system SHALL represent a skill's metadata using the Agent Skills specification frontmatter: `name` (string, required), `description` (string, required), `license` (string, optional), `compatibility` (string, optional), `metadata` (arbitrary key-value map, optional), and `allowed-tools` (space-delimited string, optional). Optional fields SHALL be omitted from output when not set.

#### Scenario: Parse SKILL.md with all frontmatter fields
- **WHEN** a skill file has all frontmatter fields (name, description, license, compatibility, metadata, allowed-tools)
- **THEN** the system SHALL parse all fields into the corresponding `SkillMetadata` struct fields without error

#### Scenario: Parse SKILL.md with only required fields
- **WHEN** a skill file has only `name` and `description` in frontmatter
- **THEN** the system SHALL parse successfully with all optional fields absent (None)

#### Scenario: Serialize optional fields only when present
- **WHEN** a `SkillFeature` with no optional fields is serialized to SKILL.md
- **THEN** the output frontmatter SHALL contain only `name` and `description`, with no null or empty optional keys

#### Scenario: Roundtrip parse-serialize preserves all fields
- **WHEN** a SKILL.md with all fields is parsed and then serialized
- **THEN** the output SHALL be semantically equivalent to the input, preserving all frontmatter and body content

### Requirement: Skill name validation
The system SHALL warn (via log) when a skill's `name` frontmatter value does not match the source filename stem, but SHALL still load the skill.

#### Scenario: Name matches filename stem
- **WHEN** the skill file is `pdf-processing.md` and frontmatter `name` is `pdf-processing`
- **THEN** the system SHALL load the skill without any warning

#### Scenario: Name does not match filename stem
- **WHEN** the skill file is `pdf.md` but frontmatter `name` is `pdf-processing`
- **THEN** the system SHALL log a warning and still load the skill successfully

### Requirement: Skills directory loading
The system SHALL load all `.md` files from the `.dotagents/skills/` directory when deploying the `skills` feature, treating each as a `SkillFeature`.

#### Scenario: Multiple skills in directory
- **WHEN** the `skills/` directory contains multiple `.md` files
- **THEN** the system SHALL load each file as a separate `SkillFeature` instance

#### Scenario: Empty skills directory
- **WHEN** the `skills/` directory is empty
- **THEN** the system SHALL return an empty list without error

### Requirement: Skills feature in deploy pipeline
The system SHALL deploy the `skills` feature when `"skills"` is listed in the `features` array of `config.toml`, rendering each skill for each active provider using the provider's skill template and target path.

#### Scenario: Deploy creates per-skill subdirectory
- **WHEN** a skill named `pdf-processing` is deployed to the claude provider
- **THEN** the system SHALL write the rendered skill to `.claude/skills/pdf-processing/SKILL.md`, creating the `pdf-processing/` subdirectory if it does not exist

#### Scenario: Skip deploy when feature not listed
- **WHEN** `"skills"` is not in the `features` array
- **THEN** the system SHALL not attempt to load or deploy any skill files

### Requirement: Skill name variable for target interpolation
The system SHALL expose `{{ skill.name }}` as a Handlebars variable during target path rendering for skill features, containing the skill's `name` value from frontmatter.

#### Scenario: Target path uses skill name
- **WHEN** a provider's skill target is `{{ dir.workspace }}/.claude/skills/{{ skill.name }}/SKILL.md`
- **THEN** the rendered path for a skill named `pdf-processing` SHALL be `<workspace>/.claude/skills/pdf-processing/SKILL.md`

#### Scenario: Skill template variables include skill fields
- **WHEN** a skill template references `{{ skill.name }}`, `{{ skill.description }}`, or `{{ skill.content }}`
- **THEN** the rendered output SHALL substitute those values from the loaded skill

### Requirement: Init scaffolds skills directory
The system SHALL create a `skills/` directory with a sample skill file during `dotagents init`, unless `--no-skill` is passed.

#### Scenario: Init without flags creates sample skill
- **WHEN** `dotagents init` is run without `--no-skill`
- **THEN** a sample skill file SHALL be created at `skills/hello-skill.md` inside the `.dotagents/` directory

#### Scenario: Init with --no-skill skips skills
- **WHEN** `dotagents init --no-skill` is run
- **THEN** no skill files SHALL be created

### Requirement: Public provider templates for skills
The system SHALL provide skill template and target path configurations in `public/v1/templates/` for Claude Code and Codex providers, with targets following the Agent Skills specification directory structure.

#### Scenario: Claude Code skill target follows spec
- **WHEN** the claude provider deploys a skill named `my-skill`
- **THEN** the target path SHALL resolve to `<workspace>/.claude/skills/my-skill/SKILL.md`

#### Scenario: Codex skill target follows spec
- **WHEN** the codex provider deploys a skill named `my-skill`
- **THEN** the target path SHALL resolve to `<workspace>/.codex/skills/my-skill/SKILL.md`
