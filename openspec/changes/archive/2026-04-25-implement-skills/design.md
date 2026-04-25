## Context

Dotagents manages AI agent configurations through a feature-based deploy pipeline. Each feature has a Rust struct implementing `FeatureTrait`, constants, mock files for `init`, and provider templates in `public/v1/`.

The [Agent Skills specification](https://agentskills.io/specification) defines a `SKILL.md`-based format for reusable agent capabilities. A skill is a directory containing `SKILL.md` plus optional `scripts/`, `references/`, and `assets/` subdirectories. The spec is supported natively by Claude Code (`.claude/skills/`), Codex (`.codex/skills/`), and other agents.

**Frontmatter schema (from spec):**

| Field           | Required | Constraints                                                                                          |
| --------------- | -------- | ---------------------------------------------------------------------------------------------------- |
| `name`          | Yes      | Max 64 chars; `[a-z0-9-]`; no leading/trailing/consecutive hyphens; must match parent directory name |
| `description`   | Yes      | Max 1024 chars; non-empty                                                                            |
| `license`       | No       | License name or reference                                                                            |
| `compatibility` | No       | Max 500 chars; environment requirements                                                              |
| `metadata`      | No       | Arbitrary key-value map                                                                              |
| `allowed-tools` | No       | Space-delimited list of pre-approved tools (experimental)                                            |

**Progressive disclosure model:** agents load only `name`+`description` at startup (~50-100 tokens/skill); full `SKILL.md` is loaded on activation; bundled resources loaded on demand.

**Source storage in dotagents:** skill directories in `.dotagents/skills/`, each named after the skill and containing a `SKILL.md` file (e.g. `.dotagents/skills/hello-skill/SKILL.md`). This mirrors the Agent Skills spec's directory structure end-to-end. The deploy step writes each skill to `<provider>/skills/<skill-name>/SKILL.md`, creating the per-skill subdirectory required by the spec.

## Goals / Non-Goals

**Goals:**

- `SkillFeature` struct with full Agent Skills spec frontmatter
- Load skills from `.dotagents/skills/` flat directory
- Deploy to `<provider>/skills/<skill-name>/SKILL.md` (subdirectory per skill)
- `{{ skill.name }}` variable for target path interpolation
- `--no-skill` init flag; sample skill on init
- Provider templates for Claude Code and Codex

**Non-Goals:**

- Installing skills from external repos (skills.sh / `npx skills add` handles that)
- Bundling `scripts/`, `references/`, `assets/` in the source (users can manage these manually outside dotagents)
- Skills catalog generation (XML `<available_skills>` format) — agents handle this themselves
- Validating skill name against parent directory at deploy time (lenient, warn only)

## Decisions

### 1. Directory source files, directory targets

Source: `.dotagents/skills/pdf-processing/SKILL.md` (directory per skill, matching the spec).
Target: `.claude/skills/pdf-processing/SKILL.md` (same structure, per spec).

The `render_feature_with_settings` pipeline already creates parent directories via `write_file`. Target paths like `{{ dir.workspace }}/.claude/skills/{{ skill.name }}/SKILL.md` will create the skill subdirectory automatically.

`from_application()` scans subdirectories of `.dotagents/skills/`, reads `SKILL.md` from each, and warns (but continues) if a directory has no `SKILL.md` or if the directory name does not match the `name` frontmatter field.

The `FeatureTrait` methods (`from_string`, `to_string`, `to_value`) operate purely on `SKILL.md` content — the directory discovery logic lives only in `from_application()`, keeping the trait clean.

**Rejected alternative**: Flat `.md` files (`.dotagents/skills/pdf-processing.md`). Initially considered for MVP simplicity, but rejected because it diverges from the spec's directory model and provides no practical advantage — dotagents users would need to maintain a different mental model for source vs. deployed skill structure.

### 2. Full frontmatter schema in `SkillMetadata`

Include all spec fields: `name`, `description`, `license`, `compatibility`, `metadata`, `allowed-tools`. Optional fields use `Option<>` with `#[serde(skip_serializing_if = "Option::is_none")]` to avoid writing empty keys.

The `allowed-tools` field uses a hyphen which is not a valid Rust identifier — use `#[serde(rename = "allowed-tools")]` on the struct field `allowed_tools`.

**Alternative**: Store only `name`+`description`. Rejected — loses spec-defined fields that agents actively use (e.g., `compatibility`, `allowed-tools`).

### 3. `get_name_variable` on `FeatureTrait` (default impl)

Add `get_name_variable(filename: &str) -> Result<Value>` to `FeatureTrait` with a default implementation calling `get_command_name_variable` (preserves existing behavior for `CommandFeature`). `SkillFeature` overrides to call `get_skill_name_variable`, injecting `{{ skill.name }}`.

Update `renderer.rs` to call `feature.get_name_variable(filename)` instead of the hardcoded `get_command_name_variable`.

**Alternative**: Add a separate code path in `renderer.rs` that checks the feature type. Rejected — tight coupling, not extensible.

### 4. Lenient name validation

Warn (via `log::warn!`) if the skill directory name doesn't match the `name` frontmatter in `SKILL.md` (e.g., directory is `pdf/` but name is `pdf-processing`). Also warn if a subdirectory has no `SKILL.md`. Don't error in either case — lenient loading keeps the tool usable even with partially correct setups.

### 5. Providers: Claude Code and Codex only

Both have documented skills paths in the skills.sh `agents.ts`:

- Claude Code: `.claude/skills/<name>/SKILL.md`
- Codex: `.codex/skills/<name>/SKILL.md`

Other providers (cursor, windsurf, etc.) don't have established conventions — skip for now.

## Risks / Trade-offs

- **`allowed-tools` field name**: Hyphenated key requires `#[serde(rename)]`; easy to miss. → Explicit rename in struct definition.
- **Directory creation at deploy**: `write_file` must create nested directories (e.g., `.claude/skills/pdf-processing/SKILL.md`). → Verify `write_file` calls `create_dir_all` on the parent; add if not already present.
- **Directory-based source**: `from_application()` must handle gracefully: subdirs with no `SKILL.md`, non-directory entries (warn + skip both), and name mismatches. → Explicit warn + continue in all cases.

## Migration Plan

Additive feature — no migration needed. Users opt in by adding `"skills"` to their `config.toml` `features` array and creating skill files in `.dotagents/skills/`.

## Open Questions

- Should `write_file` be confirmed to use `create_dir_all`? Verify in `src/utils/fs.rs` before implementing task 8.
