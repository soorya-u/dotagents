## 1. Constants and Directory Setup

- [x] 1.1 Add `SKILLS_FEATURE` constant (`"skills"`) to `src/constants/features.rs`
- [x] 1.2 Add `SKILLS_DIR` constant (`"skills"`) to `src/constants/dir.rs`
- [x] 1.3 Add `MOCK_SKILL_FILE` (`"hello-skill.md"`) and `MOCK_SKILL_TEMPLATE_FILE` (`"skill.hbs"`) constants to `src/constants/file.rs`

## 2. Mock Files

- [x] 2.1 Create `src/mocks/skills/hello-skill.md` — a sample skill with all common frontmatter fields (name, description, license, compatibility, metadata) and a short Markdown body following the Agent Skills spec
- [x] 2.2 Create `src/mocks/templates/mycode/skill.hbs` — passthrough template (e.g., `{{{ skill.content }}}` or full file passthrough)
- [x] 2.3 Add `SKILL_HELLO` and `TEMPLATE_MYCODE_SKILL` entries to `src/constants/mocks.rs` using `include_str!`

## 3. Path Utilities

- [x] 3.1 Add `get_skills_dir()` to `src/utils/path.rs`, returning `<application_dir>/skills/` (parallel to `get_commands_dir`)
- [x] 3.2 Verify that `write_file` in `src/utils/fs.rs` calls `create_dir_all` on the parent path before writing — add it if missing (required for nested skill target paths like `.claude/skills/<name>/SKILL.md`)

## 4. SkillFeature Implementation

- [x] 4.1 Create `src/schema/features/skill.rs` with `SkillMetadata` struct containing: `name` (String), `description` (String), `license` (Option<String>), `compatibility` (Option<String>), `metadata` (Option<HashMap<String, String>>), `allowed_tools` (Option<String> with `#[serde(rename = "allowed-tools")]`)
- [x] 4.2 Add `SkillFeature` struct with `metadata: SkillMetadata` and `content: String`
- [x] 4.3 Implement `from_markdown(md: &str) -> Result<Self>` using `gray_matter` YAML parsing (same pattern as `CommandFeature::from_markdown`)
- [x] 4.4 Implement `to_markdown(&self) -> Result<String>` serializing frontmatter back to YAML + content (use `serde_yaml` with `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields)
- [x] 4.5 Implement `from_application() -> Result<Vec<Self>>` — scan `.md` files in `get_skills_dir()`, parse each, and warn (via `log::warn!`) when `metadata.name` doesn't match the file stem
- [x] 4.6 Implement `FeatureTrait` for `SkillFeature`: `to_string` → `to_markdown`, `from_string` → `from_markdown`, `to_value` exposing `skill.name`, `skill.description`, `skill.content`, `get_file_name` returning `Some(metadata.name.clone())`
- [x] 4.7 Export `SkillFeature` from `src/schema/features/mod.rs`

## 5. Skill Name Variable for Renderer

- [x] 5.1 Add `get_skill_name_variable(val: &str) -> Result<Value>` to `src/templates/variables.rs` returning `json!({ "skill": { "name": val } })`
- [x] 5.2 Add `get_name_variable(&self, filename: &str) -> Result<Value>` default method to `FeatureTrait` in `src/schema/features/traits.rs` — default impl calls `get_command_name_variable(filename)`
- [x] 5.3 Override `get_name_variable` in `SkillFeature` to call `get_skill_name_variable(filename)`
- [x] 5.4 Update `src/templates/renderer.rs` to call `feature.get_name_variable(&filename)` instead of the hardcoded `get_command_name_variable(&filename)` call

## 6. Init Scaffolding

- [x] 6.1 Add `no_skill: bool` field with `#[clap(long)]` to `InitOptions` in `src/cli/options.rs`
- [x] 6.2 Add skill `InitFile` entry to `src/cli/init.rs`: `Path::new(SKILLS_DIR).join(MOCK_SKILL_FILE)` with content `mocks::SKILL_HELLO`, skipped if `opts.no_skill`
- [x] 6.3 Add skill template `InitFile` entry: `Path::new(TEMPLATE_DIR).join(MOCK_CUSTOM_AGENT_DIR).join(MOCK_SKILL_TEMPLATE_FILE)` with content `mocks::TEMPLATE_MYCODE_SKILL`

## 7. Deploy Pipeline

- [x] 7.1 Import `SKILLS_FEATURE` and `SkillFeature` in `src/cli/deploy.rs`
- [x] 7.2 Add `deploy_feature::<SkillFeature>` call in `deploy()` using `SkillFeature::from_application`

## 8. Public Provider Templates

- [x] 8.1 Create `public/v1/templates/claude/skill.hbs` — template that renders the skill as a SKILL.md (passthrough of `{{{ skill.content }}}` or full file content)
- [x] 8.2 Add `[providers.ide.claude.skills]` entry to `public/v1/templates/claude/provider.toml`: template URL pointing to `skill.hbs`, target `{{ dir.workspace }}/.claude/skills/{{ skill.name }}/SKILL.md`
- [x] 8.3 Create `public/v1/templates/codex/skill.hbs` — same passthrough template
- [x] 8.4 Add skills entry to `public/v1/templates/codex/provider.toml` with target `{{ dir.workspace }}/.codex/skills/{{ skill.name }}/SKILL.md`

## 9. Tests

- [x] 9.1 Add unit tests to `src/schema/features/skill.rs`: parse all fields, parse required-only, serialize omits absent optionals, roundtrip, `to_value`, `get_file_name`, `from_string`/`to_string`, `allowed-tools` rename
- [x] 9.2 Add test for `get_skill_name_variable` in `src/templates/variables.rs`
- [x] 9.3 Run `cargo test` and ensure all tests pass
- [x] 9.4 Run `cargo build` and ensure compilation succeeds
