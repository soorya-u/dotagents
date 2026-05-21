## Why

`SkillMetadata` is missing several fields that providers actively use. The [Agent Skills specification](https://agentskills.io/specification) defines the core schema, and Claude Code extends it with invocation-control fields. These all share the same solution space — extending `SkillMetadata` with optional fields and fixing template rendering for existing ones. Resolves [gh#138](https://github.com/soorya-u/dotagents/issues/138).

## What Changes

- Add `disable-model-invocation` (`Option<bool>`) and `user-invocable` (`Option<bool>`) to `SkillMetadata` — Claude Code extensions for controlling skill activation
- Add `paths` (`Option<Vec<String>>`) to `SkillMetadata` — Claude Code field for file-gated auto-activation
- Render `metadata` field in all 20 provider `skill.hbs` templates (struct field exists but no template outputs it)
- Render the new fields in the provider templates that support them

## Capabilities

### New Capabilities

- None

### Modified Capabilities

- `skill-feature`: `SkillMetadata` gains three new optional fields (`disable-model-invocation`, `user-invocable`, `paths`). All 20 provider `skill.hbs` templates gain `metadata` rendering. Four templates gain `disable-model-invocation`, three gain `user-invocable`, one gains `paths`.

## Impact

- Modified: `src/core/features/skill.rs` — add 3 fields to `SkillMetadata`, update `to_value`, scaffold, tests
- Modified: 20 `public/v1/templates/*/skill.hbs` files — add `metadata` rendering to all, new fields to specific providers
- No breaking changes — all new fields are `Option<>` and omitted when `None`
