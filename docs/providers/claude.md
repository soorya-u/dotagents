# Claude Code — Integration Notes

Source: https://code.claude.com/docs/en/

Things observed in Claude Code that could be integrated into dotagents:

---

## 1. Rich skill frontmatter fields

Claude Code skills support many frontmatter fields that dotagents' `SkillMetadata` doesn't model:

| Field | Purpose |
|---|---|
| `when_to_use` | Additional trigger context appended to `description` in the skill listing |
| `argument-hint` | Autocomplete hint showing expected arguments (e.g., `[issue-number]`) |
| `arguments` | Named positional args mapped to `$name` substitutions in skill content |
| `disable-model-invocation` | `true` prevents Claude from auto-loading the skill (manual-only) |
| `user-invocable` | `false` hides from `/` menu (background knowledge only) |
| `model` | Override model for this skill's turn |
| `effort` | Override effort level (`low` / `medium` / `high` / `xhigh` / `max`) |
| `context` | `fork` runs the skill in an isolated subagent context |
| `paths` | Glob patterns that gate auto-activation to matching files |
| `hooks` | Skill-scoped lifecycle hooks |

Adding these to `SkillMetadata` (with `#[serde(skip_serializing_if)]`) and surfacing them in the Claude `skill.hbs` template would give users full control. The most impactful additions are `argument-hint` (already modelled as `argument_hint` in commands), `disable-model-invocation`, and `paths`.

---

## 2. `paths` glob-scoping on skills

Claude Code's skill `paths:` frontmatter limits when a skill is automatically activated to files matching the glob patterns — the same concept as Amp's `globs` in `AGENTS.md` frontmatter (see `docs/providers/amp.md`, point 1), but applied per-skill rather than per-instruction block.

Adding `paths: Option<Vec<String>>` to `SkillMetadata` and rendering it in the `skill.hbs` template for Claude (and Amp if they add it) would allow fine-grained auto-activation control.

---

## 3. Rules — a new feature type

Claude Code has a `.claude/rules/` directory for topic-scoped instructions. Rules files support YAML frontmatter to gate activation to specific file paths:

```yaml
---
globs: ["src/api/**", "src/routes/**"]
---
Always validate request bodies with Zod before processing.
```

This is identical in concept to Amp's glob-scoped `AGENTS.md` blocks but modelled as individual files. Could be implemented as a new `FeatureTrait` (`RuleFeature`) with a target of `.claude/rules/{{rule.name}}.md`. Relevant frontmatter fields: `globs`.

---

## 4. Subagents — a new feature type

Claude Code supports subagent definitions at `.claude/agents/<name>.md` with YAML frontmatter (`name`, `description`, `tools`, `disallowedTools`, `model`). Subagents run in isolated context windows and return summarised results to the main session.

This is the same concept noted for Augment (`docs/providers/augment.md`, point 2). A shared `SubagentFeature` FeatureTrait would cover both providers. Target for Claude: `.claude/agents/{{agent.name}}.md`.

---

## 5. Output styles — a new feature type

Claude Code has `.claude/output-styles/*.md` files that inject custom sections into the system prompt to control response formatting. Each file is a plain markdown document (no frontmatter).

This could be modelled as a new `FeatureTrait` (`OutputStyleFeature`), with a target of `.claude/output-styles/{{style.name}}.md` for Claude. No other provider has an equivalent yet, but it's a natural extension once the feature framework supports it.
