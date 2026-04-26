# Cline — Integration Notes

Source: https://docs.cline.bot/

Things observed in Cline that could be integrated into dotagents:

---

## 1. Commands map to Workflows, not a commands directory

Cline has no `.cline/commands/` directory. The closest analog to dotagents' `commands` feature is **Workflows** (`.clinerules/workflows/<name>.md`), which are invoked as `/filename.md` in chat. The dotagents `commands` feature is deployed to `.clinerules/workflows/{{command.name}}.md` for this provider.

This is a pattern worth tracking: if more providers use a "workflows" model instead of a "commands" model, `provider.toml` may eventually benefit from a feature-alias mechanism (e.g., `alias = "commands"`) so users don't need to configure separate feature blocks.

---

## 2. Conditional rules with `paths`

Cline rules (`.clinerules/*.md`) support a `paths` conditional that gates rule activation to specific file patterns. This is the same concept as Amp's `globs` in `AGENTS.md` frontmatter and Claude Code's `paths` in skill frontmatter — a recurring pattern across providers.

The `InstructionFeature` currently passes content as-is. If we add an optional `paths` or `globs` metadata field to `InstructionFeature` (parsed from YAML frontmatter), provider templates could render it appropriately per-provider.

---

## 3. `alwaysAllow` field on MCP servers

Cline's MCP config supports an `alwaysAllow: string[]` field per server listing tools that are auto-approved without a confirmation prompt:

```json
{
  "mcpServers": {
    "my-server": {
      "command": "node",
      "args": ["server.js"],
      "alwaysAllow": ["read_file", "list_directory"]
    }
  }
}
```

The existing `CommonConfig` already models `disabled` and `disabled_tools`. Adding `always_allow: Option<Vec<String>>` to `CommonConfig` would cover Cline (and any other provider that adopts the same pattern).

---

## 4. MCP is stored in VSCode global extension storage

Cline's MCP configuration (`cline_mcp_settings.json`) lives inside the VSCode extension's global storage directory (not the workspace), making it impractical to deploy via dotagents' workspace-scoped templates. No `mcp.hbs` is provided for Cline.

If Cline adds a project-level MCP config file in the future (analogous to Claude's `.mcp.json`), the template could be added then.

---

## 5. Hooks in `.clinerules/hooks/`

Cline supports 8 lifecycle hook types (`TaskStart`, `TaskResume`, `TaskCancel`, `TaskComplete`, `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `PreCompact`) as JS/TS scripts in `.clinerules/hooks/`. Both global (`~/Documents/Cline/Hooks/`) and project-level hooks are supported, and all matching hooks fire (global first, then workspace).

This is the same integration opportunity noted for Augment and autohand. A `HookFeature` FeatureTrait with a per-item target would cover all three providers.

---

## 6. `.clineignore` — a potential new feature

Cline reads a `.clineignore` file at the project root (same syntax as `.gitignore`) to exclude files and directories from its context. This reduces token usage and keeps Cline focused.

A lightweight `IgnoreFeature` (or even just treating `.clineignore` as a plain-text instruction file) could let users manage their ignore patterns through dotagents alongside their other agent configuration.
