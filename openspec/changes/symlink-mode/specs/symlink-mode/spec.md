## ADDED Requirements

### Requirement: Per-feature deploy mode
The system SHALL support a `mode` field in `[feature.<name>]` config tables, accepting values `"link"` or `"template"`. When no `[feature.<name>]` table exists, the system SHALL default to `"link"` mode.

#### Scenario: Mode defaults to link when no config
- **WHEN** `config.toml` has `features = ["skill"]` with no `[feature.skill]` table
- **THEN** deploy SHALL use link mode for the skill feature

#### Scenario: Explicit mode in config
- **WHEN** `config.toml` has `[feature.skill] mode = "template"`
- **THEN** deploy SHALL use template mode for the skill feature

#### Scenario: Invalid mode value
- **WHEN** `[feature.skill] mode = "invalid"`
- **THEN** the system SHALL error with a message listing valid modes (`link`, `template`)

### Requirement: Per-item mode overrides
The system SHALL support a `mode_override` map in `[feature.<name>]` for multi-file features (commands, skills), where each entry maps an item name to a mode value that overrides the feature-level default for that specific item.

#### Scenario: Per-command override
- **WHEN** `[feature.command] mode = "link"` and `mode_override = { hello = "template" }`
- **THEN** the `hello` command SHALL deploy in template mode while all other commands SHALL deploy in link mode

#### Scenario: Override for non-existent item
- **WHEN** `mode_override` lists an item name that does not match any loaded feature item
- **THEN** the system SHALL silently ignore the unmatched override (no error)

#### Scenario: Override mode value validation
- **WHEN** `mode_override = { hello = "invalid" }`
- **THEN** the system SHALL error on config load

### Requirement: Feature type classification
The system SHALL classify features as "symlinkable" (Type 1) or "non-symlinkable" (Type 2) via a `FeatureTrait::is_symlinkable()` method. Skills, agent-ignore, and agent config files SHALL be Type 1. Commands, instructions, and MCP SHALL be Type 2.

#### Scenario: Skill returns symlinkable
- **WHEN** `SkillFeature::is_symlinkable()` is called
- **THEN** it SHALL return `true`

#### Scenario: Command returns non-symlinkable
- **WHEN** `CommandFeature::is_symlinkable()` is called
- **THEN** it SHALL return `false`

### Requirement: Symlink deploy for Type 1 link mode
When a Type 1 feature is deployed with mode `"link"`, the system SHALL create a filesystem symlink from the source file in `.dotagents/` to the resolved target path, without rendering any templates or injecting any variables into the content.

#### Scenario: Skill symlinked to provider directory
- **WHEN** a skill named `my-skill` is deployed to `claude` with mode=link
- **THEN** a symlink SHALL be created at `<workspace>/.claude/skills/my-skill/SKILL.md` pointing to `<workspace>/.dotagents/skills/my-skill/SKILL.md`

#### Scenario: Agent-ignore symlinked
- **WHEN** the agent-ignore feature is deployed with mode=link
- **THEN** a symlink SHALL be created at the provider's `.agentignore` path pointing to `.dotagents/.agentignore`

#### Scenario: Target path still rendered from config
- **WHEN** Type 1 link mode is used and the `target` config string contains `{{ dir.workspace }}`
- **THEN** the target path template SHALL be rendered to resolve the absolute path, but the SYMLINK shall point to the source WITHOUT any content-level rendering

### Requirement: Template deploy for Type 1 template mode
When a Type 1 feature is deployed with mode `"template"`, the system SHALL write the source content to the target path after injecting user variables and env variables into the content (Phase 3). No `.hbs` template file SHALL be used (Phase 2 skipped).

#### Scenario: Skill deployed with template mode
- **WHEN** a skill is deployed with mode=template and source contains `{{ var.api_key }}`
- **THEN** the written output SHALL have `{{ var.api_key }}` replaced with the variable value, with no `.hbs` template transformation

#### Scenario: Template mode writes a regular file
- **WHEN** Type 1 template mode is used
- **THEN** the output SHALL be a regular file (not a symlink)

### Requirement: Deploy for Type 2 link mode
When a Type 2 feature is deployed with mode `"link"`, the system SHALL render through the `.hbs` template (Phase 2) but SHALL skip variable and env injection into the content (Phase 3). Output SHALL be a written file, not a symlink.

#### Scenario: Command deployed with link mode
- **WHEN** a command is deployed with mode=link
- **THEN** the rendered output SHALL pass through the provider's `.hbs` template, but `{{ var.* }}` and `{{ env.* }}` expressions in the source command body SHALL NOT be substituted

#### Scenario: Type 2 link mode writes a regular file
- **WHEN** Type 2 link mode is used
- **THEN** the output SHALL be a regular file (not a symlink)

### Requirement: Deploy for Type 2 template mode (existing behavior)
When a Type 2 feature is deployed with mode `"template"`, the system SHALL perform the full 3-phase pipeline: resolve target path, inject variables into content, render through `.hbs` template, and write the resulting file. This SHALL be the existing behavior preserved.

#### Scenario: Command deployed with template mode
- **WHEN** a command is deployed with mode=template
- **THEN** the rendered output SHALL include both `.hbs` template transformation AND variable injection from `{{ var.* }}` and `{{ env.* }}`

### Requirement: Source path tracking
Feature items SHALL store the path to their source file for use in symlink creation. The `FeatureTrait` SHALL expose a `get_source_path() -> Option<PathBuf>` method.

#### Scenario: Skill stores source path
- **WHEN** a `SkillFeature` is loaded from `.dotagents/skills/my-skill/SKILL.md`
- **THEN** `get_source_path()` SHALL return `Some(<workspace>/.dotagents/skills/my-skill/SKILL.md)`

#### Scenario: Command stores source path
- **WHEN** a `CommandFeature` is loaded from `.dotagents/commands/hello.md`
- **THEN** `get_source_path()` SHALL return `Some(<workspace>/.dotagents/commands/hello.md)`

### Requirement: Template field optional for Type 1
The `template` field in `FeatureSettings` SHALL be optional for Type 1 features (skills, agent-ignore, agent configs) and SHALL be required for Type 2 features (commands, instructions, MCP) regardless of mode.

#### Scenario: Type 1 without template succeeds
- **WHEN** `[providers.claude.skills]` has `target` but no `template`
- **THEN** deploy SHALL succeed for the skill feature

#### Scenario: Type 2 without template errors
- **WHEN** `[providers.claude.commands]` has `target` but no `template`
- **THEN** deploy SHALL fail with an error message indicating template is required

### Requirement: Skills extra files always symlinked
When deploying a skill, all files in the skill directory except `SKILL.md` SHALL be symlinked to the provider's target directory, maintaining relative paths. This SHALL happen regardless of the mode setting for the skill.

#### Scenario: Extra Python script symlinked
- **WHEN** a skill directory contains `SKILL.md`, `script.py`, and `data/config.json`
- **AND** the skill is deployed to `claude` with target path `.claude/skills/my-skill/SKILL.md`
- **THEN** symlinks SHALL be created at `.claude/skills/my-skill/script.py` and `.claude/skills/my-skill/data/config.json` pointing to their `.dotagents/skills/my-skill/` counterparts

#### Scenario: No extra files
- **WHEN** a skill directory contains only `SKILL.md`
- **THEN** no additional symlinks SHALL be created

#### Scenario: Extra files with template mode
- **WHEN** a skill is deployed with mode=template and has extra files
- **THEN** `SKILL.md` SHALL be written as a rendered file while extra files SHALL still be symlinked

### Requirement: No cache for Type 1 link mode
The system SHALL NOT create cache entries for items deployed via Type 1 link mode. Cache SHALL still be used for template mode and for Type 2 features.

#### Scenario: Symlinked skill not in cache
- **WHEN** a skill is deployed with mode=link
- **THEN** no entry SHALL be written to `cache.toml` for that skill

#### Scenario: Template mode skill in cache
- **WHEN** a skill is deployed with mode=template
- **THEN** a cache entry with hash and target SHALL be written

### Requirement: Dedup supports symlinks
The system SHALL apply the existing path-based deduplication logic to symlink deploys. If two providers resolve to the same target path, the alphabetically-first provider SHALL win and create the symlink; others SHALL be skipped with a dedup warning.

#### Scenario: Symlink dedup
- **WHEN** providers `claude` and `codex` both target the same symlink path for a skill
- **THEN** exactly one symlink SHALL be created and the other SHALL be skipped

### Requirement: .gitignore fence includes symlinked paths
When updating the `.gitignore` fence after deploy, the system SHALL include target paths from symlinked items alongside written items.

#### Scenario: Gitignore includes symlink paths
- **WHEN** deploy creates symlinks for skills and agent-ignore
- **THEN** the `.gitignore` fence SHALL contain the target paths of those symlinks

### Requirement: Init scaffold unchanged
The `dotagents init` scaffolded config SHALL NOT include explicit `[feature.<name>]` tables. The hardcoded `"link"` default SHALL apply implicitly.

#### Scenario: Init config has no feature tables
- **WHEN** `dotagents init` runs
- **THEN** the generated `config.toml` SHALL contain `features = [...]` but no `[feature.<name>]` tables

### Requirement: Registry provider.toml drops template for Type 1
Provider template files (`provider.toml`) for Type 1 features SHALL include `target` but SHALL NOT include `template`.

#### Scenario: Claude skill provider config
- **WHEN** reading `public/v1/templates/claude/provider.toml`
- **THEN** the `[providers.claude.skills]` section SHALL have `target` but no `template` field

#### Scenario: Claude command provider config unchanged
- **WHEN** reading `public/v1/templates/claude/provider.toml`
- **THEN** the `[providers.claude.commands]` section SHALL still have both `template` and `target`

### Requirement: Dry-run reports symlink operations
When `--dry-run` is passed, the system SHALL report what would be symlinked without creating actual symlinks.

#### Scenario: Dry-run for link mode
- **WHEN** deploy runs with `--dry-run` and mode is link for a skill
- **THEN** the output SHALL indicate the symlink would be created without actually creating it
