## 1. Add mock() to each feature type

- [x] 1.1 In `src/schema/features/command.rs`, add a module-level `const` with the hello.md content and a `pub(crate) fn mock() -> &'static str` returning it
- [x] 1.2 In `src/schema/features/skill.rs`, add a module-level `const` with the hello-skill SKILL.md content and a `pub(crate) fn mock() -> &'static str` returning it
- [x] 1.3 In `src/schema/features/mcp.rs`, add a module-level `const` with the mcp.jsonc content and a `pub(crate) fn mock() -> &'static str` returning it
- [x] 1.4 In `src/schema/features/instruction.rs`, add a module-level `const` with the INSTRUCTIONS.md content and a `pub(crate) fn mock() -> &'static str` returning it

## 2. Rewrite src/constants/mocks.rs

- [x] 2.1 Replace all `include_str!` macros with inline `pub(crate) const` string literals for static content: `ENV_EXAMPLE`, `GITIGNORE`, `TEMPLATE_MYCODE_COMMAND`, `TEMPLATE_MYCODE_SKILL`, `TEMPLATE_MYCODE_INSTRUCTIONS`, `TEMPLATE_MYCODE_MCP`
- [x] 2.2 Add `pub(crate) fn default_config(features: &[&str], targets: &[&str]) -> String` that returns a formatted TOML string with `schema`, `features`, `targets`, and `variables` keys
- [x] 2.3 Remove `CONFIG`, `LOCAL_CONFIG_WITH_PROVIDER`, `COMMAND_HELLO`, `SKILL_HELLO`, `MCP`, `INSTRUCTIONS` constants from `mocks.rs` (these move to feature `mock()` methods or `default_config`)

## 3. Update src/cli/init.rs

- [x] 3.1 Replace `InitFile::new(GLOBAL_CONFIG_FILE, mocks::CONFIG)` with a direct `write_file` call using `mocks::default_config(features_slice, targets_slice)` outside the `init_files` vec
- [x] 3.2 Replace `InitFile::new(LOCAL_CONFIG_FILE, …)` similarly — both config files written outside the vec
- [x] 3.3 Replace `mocks::COMMAND_HELLO` with `CommandFeature::mock()`
- [x] 3.4 Replace `mocks::SKILL_HELLO` with `SkillFeature::mock()`
- [x] 3.5 Remove now-unused imports from `init.rs` (`mocks::CONFIG`, `mocks::LOCAL_CONFIG_WITH_PROVIDER`, `mocks::COMMAND_HELLO`, `mocks::SKILL_HELLO`, `mocks::MCP`, `mocks::INSTRUCTIONS`)

## 4. Clean up dead constants

- [x] 4.1 In `src/constants/file.rs`, remove any constants that are no longer referenced after the refactor (e.g. `MOCK_COMMAND_FILE`, `MOCK_SKILL_DIR`, `MOCK_COMMAND_TEMPLATE_FILE`, `MOCK_INSTRUCTION_TEMPLATE_FILE`, `MOCK_MCP_TEMPLATE_FILE`, `MOCK_SKILL_TEMPLATE_FILE`)
- [x] 4.2 In `src/constants/dir.rs`, remove `MOCK_CUSTOM_AGENT_DIR` and `TEMPLATE_DIR` if unused
- [x] 4.3 Delete `src/mocks/` directory entirely

## 5. Tests

- [x] 5.1 Add unit test `default_config_produces_valid_toml` — assert output parses as TOML with expected `features` and `targets` keys
- [x] 5.2 Add unit test `default_config_empty_slices` — assert `features = []` and `targets = []`
- [x] 5.3 Add unit tests `command_mock_has_frontmatter` and `skill_mock_has_frontmatter` — assert `mock()` output is non-empty and contains `name:` in frontmatter
- [x] 5.4 Update any existing unit tests in `init.rs` that reference `mocks::CONFIG` directly — replace with `default_config` calls
- [x] 5.5 Run `mise check && mise tests` and fix any failures
