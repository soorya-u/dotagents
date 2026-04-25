# Factory Droid — Integration Notes

Sources:
- https://docs.factory.ai/cli/configuration/agents-md
- https://docs.factory.ai/cli/configuration/skills
- https://docs.factory.ai/cli/configuration/mcp
- https://docs.factory.ai/cli/configuration/custom-droids
- https://docs.factory.ai/cli/configuration/hooks-guide
- https://docs.factory.ai/cli/configuration/custom-slash-commands
- https://docs.factory.ai/cli/configuration/plugins

Things observed in Factory Droid that could be integrated into dotagents:

---

## 1. Skills have Factory-specific frontmatter fields not in `SkillMetadata`

Factory skill frontmatter supports two fields beyond the Agent Skills standard: `user-invocable: bool` (default `true` — set to `false` to hide from the slash command menu, keeping the skill available only for automatic Droid invocation) and `disable-model-invocation: bool` (default `false` — set to `true` to prevent the Droid from automatically loading the skill, making it user-invoke-only). Neither is in the current `SkillMetadata` struct. Adding both as `Option<bool>` would let dotagents express factory-specific invocation control. Note that `disable-model-invocation` is also used by Cursor for the same purpose.

---

## 2. MCP stdio servers require an explicit `"type": "stdio"` field

Factory's `mcp.json` uses `"type": "stdio"` explicitly for stdio servers (unlike Claude which omits the type and infers it from the presence of `command`). The factory `mcp.hbs` template handles this by always rendering `"type": "stdio"` in the else branch. This is already correct in the template and requires no Rust changes, but it highlights that `ServerConfig`'s type tag ("stdio"/"http") should be consistently exposed to all templates via `{{this.type}}`.

---

## 3. MCP server entries have `disabled` and `disabledTools` fields not in `ServerConfig`

Factory's MCP config supports per-server `disabled: bool` and `disabledTools: string[]` fields directly inside each `mcpServers` entry. The `disabled` field corresponds to dotagents' `CommonConfig.disabled`, and `disabledTools` corresponds to `CommonConfig.disabled_tools` — but these are on the dotagents config layer, not on `ServerConfig` itself. Adding `disabled` and `disabled_tools` as optional fields to `ServerConfig` (or an envelope type wrapping it) and rendering them in the MCP template would provide full coverage.

---

## 4. Hooks are embedded inside `settings.json`, not a separate file

Factory hooks are stored under the `"hooks"` key inside `~/.factory/settings.json` (or `<project>/.factory/settings.json`), not in their own file. The hook schema is:
```json
{
  "hooks": {
    "PreToolUse": [{ "matcher": "Execute", "hooks": [{ "type": "command", "command": "..." }] }]
  }
}
```
Events: `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Notification`, `Stop`, `SubagentStop`, `PreCompact`, `SessionStart`, `SessionEnd`. Each event entry has a `matcher` (tool name glob) and an array of `{ type, command }` hook objects. Unlike Cursor (blocking via exit code 2) and Deep Agents (fire-and-forget), Factory hooks can block via exit code 2 on `PreToolUse`. If a `HookFeature` is added, the settings.json embedding pattern would need to merge hook config into an existing JSON file rather than writing a dedicated file.

---

## 5. Custom Droids (subagents) have `reasoningEffort` and tool-category shorthand

Factory subagents live in `.factory/droids/<name>.md` with YAML frontmatter: `name`, `description`, `model`, `reasoningEffort` (`low`/`medium`/`high`), and `tools` (either a category string like `"read-only"` or an array of tool IDs). The `reasoningEffort` field has no equivalent in Claude/Codex subagents. The tool-category shorthand (`read-only`, `edit`, `execute`, `web`, `mcp`) is a factory-specific abstraction. A shared `SubagentFeature` would need to treat these as optional, provider-specific fields surfaced through the template layer.

---

## 6. Commands support executable files (shebang) in addition to Markdown

Factory `.factory/commands/` supports both Markdown files and executable scripts (any file starting with `#!`). Executable commands receive `$ARGUMENTS` as positional shell args and stream stdout/stderr back to the chat transcript. The current `CommandFeature` models commands as Markdown-with-frontmatter only. Executable commands are a different execution model that dotagents cannot represent as a template output — the shebang-based script must be authored directly by the user.
