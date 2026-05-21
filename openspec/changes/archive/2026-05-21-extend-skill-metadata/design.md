## Context

The `SkillMetadata` struct currently has 6 fields matching the Agent Skills spec: `name`, `description`, `license`, `compatibility`, `metadata`, `allowed-tools`. Two gaps exist:

1. **`metadata` not rendered** — the struct has `Option<HashMap<String, String>>` but no `skill.hbs` template outputs it via `{{#each skill.metadata}}`
2. **Claude Code extensions missing** — `disable-model-invocation`, `user-invocable`, and `paths` are documented in [Claude Code's skill frontmatter reference](https://code.claude.com/docs/en/skills#frontmatter-reference) but not in our struct

The `allowed_tools` type was questioned in the issue but the Agent Skills spec confirms it should remain a space-separated `Option<String>`. No type change needed.

Amp's `globs` field was also mentioned but it is an AGENTS.md instruction-file feature, not a skill frontmatter field — out of scope.

## Goals / Non-Goals

**Goals:**

- Add `disable_model_invocation: Option<bool>` with `#[serde(rename = "disable-model-invocation")]`
- Add `user_invocable: Option<bool>` with `#[serde(rename = "user-invocable")]`
- Add `paths: Option<Vec<String>>`
- Render `metadata` in all 20 provider templates
- Render new fields in the specific provider templates that use them
- Update scaffold function to accept new optional fields
- Update tests

**Non-Goals:**

- Changing `allowed_tools` type (spec says space-separated string)
- Adding `globs` (not a skill frontmatter field)
- Adding other Claude Code extensions not in scope (`when_to_use`, `argument-hint`, `arguments`, `model`, `effort`, `context`, `agent`, `hooks`, `shell`) — these can be future PRs

## Decisions

### 1. New fields on `SkillMetadata`

Add three fields, all `Option<>` with `skip_serializing_if = "Option::is_none"`:

```rust
#[serde(rename = "disable-model-invocation", skip_serializing_if = "Option::is_none")]
pub disable_model_invocation: Option<bool>,

#[serde(rename = "user-invocable", skip_serializing_if = "Option::is_none")]
pub user_invocable: Option<bool>,

#[serde(skip_serializing_if = "Option::is_none")]
pub paths: Option<Vec<String>>,
```

`paths` uses `Vec<String>` because the spec says it accepts "a comma-separated string or a YAML list" — storing as `Vec<String>` lets us output as a YAML list in templates.

### 2. Template rendering strategy

**`metadata` — all 20 providers.** Pattern:
```handlebars
{{#if skill.metadata}}metadata:
{{#each skill.metadata}}  {{@key}}: {{this}}
{{/each}}
{{/if}}
```

**`disable-model-invocation` — 4 providers** (Claude, Cursor, Factory Droid, Pi):
```handlebars
{{#if skill.disable-model-invocation}}disable-model-invocation: {{skill.disable-model-invocation}}
{{/if}}
```

**`user-invocable` — 3 providers** (Claude, Factory Droid, Mistral Vibe):
```handlebars
{{#if skill.user-invocable}}user-invocable: {{skill.user-invocable}}
{{/if}}
```

**`paths` — 1 provider** (Claude):
```handlebars
{{#if skill.paths}}paths:
{{#each skill.paths}}  - {{this}}
{{/each}}
{{/if}}
```

### 3. Template field ordering

New fields are inserted in a consistent order after existing fields: `name`, `description`, `license`, `compatibility`, `metadata`, `disable-model-invocation`, `user-invocable`, `allowed-tools`, `paths`. This matches the logical grouping from the spec and Claude docs.

### 4. `to_value` update

The `to_value` impl serializes `SkillMetadata` via `serde_json::to_value`, so new fields automatically appear in the Handlebars context. No code change needed beyond adding the struct fields.

### 5. Scaffold update

`SkillFeature::scaffold()` gains two new parameters (`disable_model_invocation: bool`, `user_invocable: bool`) defaulting to `None` in the struct. The CLI scaffold call passes `None` for both — interactive TUI can set them later.

## Risks / Trade-offs

- **Template churn across 20 files** — mechanical change but high surface area. Each template must be verified individually.
- **`paths` as `Vec<String>` vs `String`** — spec says "comma-separated string or YAML list". We store as `Vec<String>` and render as YAML list. If a provider needs comma-separated, the template can use `{{join skill.paths ","}}` but Handlebars doesn't have a built-in join. If needed later, we can add a custom helper.
- **Breaking existing deployed skills** — additive only, existing skills without new fields deploy unchanged (fields are `None`, templates skip them).

## Migration Plan

Additive feature — no migration needed. Existing skill files without the new fields parse fine (fields default to `None`). Existing deployed outputs are unaffected.

## Open Questions

- None
