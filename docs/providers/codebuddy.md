# CodeBuddy (Tencent) — Integration Notes

Sources:
- https://staging-codebuddy.tencent.com/docs/ide/User-guide/Slash-Commands
- https://staging-codebuddy.tencent.com/docs/ide/User-guide/Rules
- https://staging-codebuddy.tencent.com/docs/ide/User-guide/MCP
- https://staging-codebuddy.tencent.com/docs/ide/Features/Subagents
- https://staging-codebuddy.tencent.com/docs/ide/Features/Skills
- https://staging-codebuddy.tencent.com/docs/ide/Features/hooks

Things observed in CodeBuddy that could be integrated into dotagents:

---

## 1. Instructions target `CODEBUDDY.md`; falls back to `AGENTS.md` for backward compatibility

CodeBuddy auto-loads `CODEBUDDY.md` from the project root into every session's context. It is a plain Markdown file with no frontmatter or metadata — identical in structure to `AGENTS.md`. If `CODEBUDDY.md` is absent and `AGENTS.md` exists, CodeBuddy loads `AGENTS.md` instead. The `instruction.hbs` template targets `CODEBUDDY.md` (the native primary file). Users who want to maintain a single file shared with Claude, Gemini, and other `AGENTS.md` consumers can instead point the provider entry at `AGENTS.md` via a custom target.

---

## 2. Rules system is a separate, richer feature not covered by `InstructionFeature`

Beyond `CODEBUDDY.md`, CodeBuddy supports a structured project rules system at `.codebuddy/rules/<name>/RULE.mdc`. Rule files use `.mdc` extension and YAML frontmatter with fields `description`, `alwaysApply` (bool), `enabled` (bool), `updatedAt` (ISO timestamp), and `provider` (string). Three loading modes are supported: `Always` (injected every session), `Agent Requested` (lazy — only name/description surfaced until the model requests full content), and `Manual` (@-mention triggered). This three-tier progressive disclosure is more sophisticated than `InstructionFeature`'s single content blob. Modelling rules would require a new feature type with at minimum `description`, `alwaysApply`, and `enabled` fields, plus a `RULE.mdc` target extension that differs from standard `.md`.

---

## 3. Skills live in `.codebuddy/skills/` and support a CodeBuddy-specific `disable` field

CodeBuddy skills use the standard Agent Skills format (`name`, `description` in YAML frontmatter, Markdown body) at `.codebuddy/skills/<name>/SKILL.md`. Two extra optional frontmatter fields appear in CodeBuddy's own documentation: `allowed-tools` (same as the Agent Skills standard field, already modelled in `SkillMetadata`) and `disable: false` (a boolean to disable the skill without deleting it). The `disable` field is CodeBuddy-specific and has no analog in `SkillMetadata`. The `skill.hbs` template outputs `allowed-tools` when present but omits `disable` since there is no source field for it. Users who need to disable a specific skill should edit the file directly.

---

## 4. MCP uses explicit `"type": "stdio"` and a per-server `"description"` field

CodeBuddy's MCP JSON format (shown in the docs example) explicitly includes `"type": "stdio"` for local process servers, unlike Claude and Junie which omit the type discriminator. It also includes a `"description"` string field per server that other providers do not use. The `mcp.hbs` template outputs `"type": "stdio"` for non-HTTP servers and `"type": "http"` for HTTP servers, consistent with the Factory Droid pattern. The per-server `"description"` field is silently dropped since `ServerConfig` does not model it. The target file `.codebuddy/mcp.json` is inferred from the project directory structure (the docs only show UI-based configuration; no explicit file path is documented). Users should verify this path with their installed CodeBuddy version.

---

## 5. Hooks are fully Claude Code-compatible — stored in `.codebuddy/settings.json`

The docs explicitly state: "The Hook mechanism is fully compatible with the Claude Code Hooks specification." Hooks are stored in `.codebuddy/settings.json` under a `"hooks"` key, using the same event names (`SessionStart`, `SessionEnd`, `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Stop`, `PreCompact`), the same `matcher` regex field, and the same `type: "command"` / `command` / `timeout` hook structure as Claude. If a `HookFeature` is ever added to dotagents, CodeBuddy would be the easiest provider to support — the template would be nearly identical to Claude's. The shared-file concern (hooks in `settings.json` alongside other settings) is the same overwrite problem seen with Gemini's `settings.json` and goose's `config.yaml`.

---

## 6. Subagents have `agentMode` and `enabledAutoRun` fields not in `SkillFeature`

CodeBuddy subagents (`.codebuddy/agents/<name>.md`) use YAML frontmatter with `name`, `description`, `agentMode` (`"agentic"` or `"manual"`), `tools` (comma-separated string e.g. `WebFetch, WebSearch`), `model`, `enabled`, and `enabledAutoRun` (bool). The `agentMode` field controls whether the main agent auto-delegates to the subagent (`agentic`) or the user selects it manually (`manual`). Neither `agentMode` nor `enabledAutoRun` have analogs in `SkillMetadata`. A future `SubagentFeature` would need these fields. The `tools` field uses a comma-separated string format (not a YAML array), which differs from Junie's array format — this is a cross-provider inconsistency worth noting in any future common subagent schema.

---

## 7. Existing `command.hbs` is a plain pass-through; custom command format is undocumented

The existing `command.hbs` passes `{{command.content}}` directly without frontmatter. CodeBuddy's custom slash command docs describe name and description as attributes but do not publish the file format for `.codebuddy/commands/` files. The IDE creates these files via a wizard, and the on-disk format was not discoverable from the available documentation. The current plain pass-through is kept to avoid breaking existing users. If the file format turns out to include a `description` frontmatter header (as Junie does), a future update to `command.hbs` could add `---\ndescription: {{command.description}}\n---\n\n{{{command.content}}}`.
