## Why

Mock content for `dotagents init` is scattered across `src/mocks/` as individual files included at compile time via `include_str!`. This creates a disconnect between the code that uses mock content and the content itself, makes dynamic config generation impossible (the static `config.toml` mock is what forces the features-persistence bug), and duplicates responsibility with the inline starter templates already in `src/constants/templates.rs`. Consolidating everything into inline constants and per-feature `mock()` methods makes the init scaffolding self-contained and opens the door to the `default_config()` builder needed by `fix-init-ux`.

## What Changes

- **Delete `src/mocks/`** entirely — `config.toml`, `local.config.toml`, `INSTRUCTIONS.md`, `mcp.jsonc`, `.env.example`, `.gitignore.example`, `commands/hello.md`, `skills/hello-skill/SKILL.md`, `templates/mycode/*.hbs`.
- **`src/constants/mocks.rs`** becomes inline string constants for static content (`.env`, `.gitignore`, `INSTRUCTIONS.md`, `mcp.jsonc`, mycode template files) and a builder function `fn default_config(features: &[&str], targets: &[&str]) -> String` that generates the config TOML string programmatically.
- **`Feature::mock()` on each feature type** — `CommandFeature`, `SkillFeature`, `McpFeature`, and `InstructionFeature` in `src/schema/features/` each gain a `pub(crate) fn mock() -> &'static str` returning their example file content inline.
- **`src/cli/init.rs` call sites updated** — `mocks::COMMAND_HELLO` → `CommandFeature::mock()`, `mocks::SKILL_HELLO` → `SkillFeature::mock()`, config content replaced with `mocks::default_config(...)`.
- **`src/constants/templates.rs` unchanged** — `COMMAND_STARTER` and `SKILL_STARTER` remain as-is.

## Capabilities

### New Capabilities

*(none — pure refactor; no user-visible behaviour changes)*

### Modified Capabilities

*(none — no spec-level requirement changes)*

## Impact

- `src/mocks/` — deleted.
- `src/constants/mocks.rs` — rewritten: `include_str!` macros removed, inline `&'static str` constants added, `default_config()` function added.
- `src/schema/features/command.rs`, `skill.rs`, `mcp.rs`, `instruction.rs` — each gains a `mock()` associated function.
- `src/cli/init.rs` — call sites updated; `InitFile` content field for config entries replaced with `default_config()` calls; `COMMAND_HELLO` / `SKILL_HELLO` replaced with feature `mock()` calls.
- `src/constants/file.rs` / `src/constants/dir.rs` — constants referencing mock file paths may be removed if unused after refactor.
- No new external dependencies.
