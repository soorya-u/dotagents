## 1. Extend SkillMetadata struct

- [x] 1.1 Add `disable_model_invocation: Option<bool>` with `#[serde(rename = "disable-model-invocation", skip_serializing_if = "Option::is_none")]` to `SkillMetadata`
- [x] 1.2 Add `user_invocable: Option<bool>` with `#[serde(rename = "user-invocable", skip_serializing_if = "Option::is_none")]` to `SkillMetadata`
- [x] 1.3 Add `paths: Option<Vec<String>>` with `#[serde(skip_serializing_if = "Option::is_none")]` to `SkillMetadata`
- [x] 1.4 Update `scaffold()` to accept `disable_model_invocation` and `user_invocable` optional bool parameters
- [x] 1.5 Update `to_value` test to include new fields when present
- [x] 1.6 Add unit tests: parse new fields, serialize omits absent, roundtrip preserves, `to_value` includes new fields
- [x] 1.7 Test with tui-devtools: create a skill with new fields, verify `skills ls --content` shows them

## 2. Render metadata in all 20 provider templates

- [x] 2.1 Add `metadata` rendering block to all 20 `skill.hbs` templates (after `compatibility`, before provider-specific fields)
- [x] 2.2 Templates: amp, autohand, auggie, claude, cline, codex, copilot, cursor, deepagents, factory-droid, gemini, goose, junie, kilocode, kimi, mistral-vibe, opencode, pi, qoder-cli, qwen
- [x] 2.3 Test with tui-devtools: deploy a skill with `metadata: {author: test, version: "1.0"}` to multiple providers, verify output

## 3. Render disable-model-invocation in 4 provider templates

- [x] 3.1 Add `disable-model-invocation` rendering to claude/skill.hbs
- [x] 3.2 Add `disable-model-invocation` rendering to cursor/skill.hbs
- [x] 3.3 Add `disable-model-invocation` rendering to factory-droid/skill.hbs
- [x] 3.4 Add `disable-model-invocation` rendering to pi/skill.hbs
- [x] 3.5 Test with tui-devtools: deploy skill with `disable-model-invocation: true` to each provider

## 4. Render user-invocable in 3 provider templates

- [x] 4.1 Add `user-invocable` rendering to claude/skill.hbs
- [x] 4.2 Add `user-invocable` rendering to factory-droid/skill.hbs
- [x] 4.3 Add `user-invocable` rendering to mistral-vibe/skill.hbs
- [x] 4.4 Test with tui-devtools: deploy skill with `user-invocable: false` to each provider

## 5. Render paths in claude template

- [x] 5.1 Add `paths` rendering to claude/skill.hbs
- [x] 5.2 Test with tui-devtools: deploy skill with `paths: ["src/**/*.rs", "tests/**"]` to claude provider

## 6. Update spec and verify

- [x] 6.1 Update `openspec/specs/skill-feature/spec.md` with new requirements for the 3 fields and metadata rendering
- [x] 6.2 Run `mise check` — cargo fmt + cargo clippy must exit 0
- [x] 6.3 Run `mise tests` — all unit + integration + e2e tests must pass
- [x] 6.4 Add unit tests for new struct fields in `src/core/features/skill.rs`
- [x] 6.5 Add e2e tests in `tests/e2e/skills.test.ts` for new fields in deployed output
