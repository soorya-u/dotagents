## Why

Dotagents manages MCP servers, commands, and instructions for AI agents, but lacks support for Skills — a lightweight, open format defined by the [Agent Skills specification](https://agentskills.io/specification) for extending AI agents with reusable procedural knowledge. Skills are already a first-class concept in Claude Code, Codex, and other agents, and users need a way to manage them from a single source of truth just like they do for commands and instructions.

## What Changes

- New `skills` feature type alongside `mcp`, `commands`, and `instructions`
- Skills are sourced from subdirectories in `.dotagents/skills/`, each named after the skill and containing a `SKILL.md` file (e.g. `.dotagents/skills/my-skill/SKILL.md`)
- Each skill file uses YAML frontmatter matching the Agent Skills spec (`name`, `description`, optional `license`, `compatibility`, `metadata`, `allowed-tools`) plus free-form Markdown body
- The `deploy` command renders and writes each skill to the provider-specific skill path (e.g., `.claude/skills/<skill-name>/SKILL.md`), creating the required per-skill subdirectory
- `init` scaffolds a sample skill file; a `--no-skill` flag opts out
- Config schema recognizes `"skills"` as a valid feature

## Capabilities

### New Capabilities

- `skill-feature`: Defines the `SkillFeature` data model — reads skill files with YAML frontmatter matching the Agent Skills specification, serializes them, supports Handlebars templating, and exposes `{{ skill.name }}` for target path interpolation. Mirrors the `CommandFeature` pattern (per-file, named from frontmatter), but with richer metadata and a target path that includes a subdirectory per skill.

### Modified Capabilities

- None

## Impact

- New file: `src/schema/features/skill.rs`
- Updated: `src/schema/features/mod.rs` — export `SkillFeature`
- Updated: `src/constants/features.rs` — add `SKILLS_FEATURE`
- Updated: `src/constants/dir.rs` — add `SKILLS_DIR`
- Updated: `src/constants/file.rs` — add `SKILL_FILE`, `MOCK_SKILL_FILE`, `MOCK_SKILL_TEMPLATE_FILE`
- Updated: `src/constants/mocks.rs` — embed mock skill file content
- Updated: `src/utils/path.rs` — add `get_skills_dir()`
- Updated: `src/templates/variables.rs` — add `get_skill_name_variable()`
- Updated: `src/schema/features/traits.rs` — add `get_name_variable()` default method
- Updated: `src/templates/renderer.rs` — call `feature.get_name_variable()` instead of hardcoded `get_command_name_variable`
- Updated: `src/cli/init.rs` — scaffold skills directory and mock skill
- Updated: `src/cli/options.rs` — add `--no-skill` flag
- Updated: `src/cli/deploy.rs` — handle `skills` feature
- Updated: `public/v1/` provider templates — add skill template/target for Claude Code and Codex
