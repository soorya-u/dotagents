# Qwen Code — Integration Notes

Sources:
- https://qwenlm.github.io/qwen-code-docs/en/users/features/commands/
- https://qwenlm.github.io/qwen-code-docs/en/users/features/skills/
- https://qwenlm.github.io/qwen-code-docs/en/users/features/mcp/
- https://qwenlm.github.io/qwen-code-docs/en/users/features/hooks/
- https://qwenlm.github.io/qwen-code-docs/en/users/features/sub-agents/
- https://qwenlm.github.io/qwen-code-docs/en/users/configuration/qwen-ignore/

Things observed in Qwen Code that could be integrated into dotagents:

---

## 1. No instruction file documented — InstructionFeature not deployable

Qwen Code has a `/memory` command that manages AI instruction context at runtime, but no static instruction file (such as `AGENTS.md`, `QWEN.md`, or `.qwen/INSTRUCTIONS.md`) is documented. The configuration pages describe `.qwen/settings.json`, `.qwenignore`, and theme/auth files, but none correspond to a project-level instruction blob that dotagents could write. InstructionFeature is not included in `provider.toml`. If a future release documents a memory file path, an `instruction.hbs` pass-through targeting it can be added trivially.

---

## 2. MCP HTTP transport uses `httpUrl` key — unique among all surveyed providers

Qwen Code discriminates MCP transport by field presence rather than a `type` discriminator: stdio servers have a `command` field, HTTP servers have an `httpUrl` field, and SSE servers have a `url` field. The `ServerConfig.Http` variant carries a `url` field, which `mcp.hbs` renders as `httpUrl` to target Qwen Code's HTTP transport. This is the only provider among all surveyed that uses `httpUrl` rather than `url` for the HTTP transport key. SSE transport (the `url` key) is not representable with the current `ServerConfig` model — users requiring SSE must add entries manually.

---

## 3. MCP lives in `.qwen/settings.json` alongside hooks — intermediate file used

Qwen Code loads MCP from the `mcpServers` key in `.qwen/settings.json` (project-level) or `~/.qwen/settings.json` (user-level). The same file also stores the `hooks` configuration. Writing only `{ "mcpServers": { ... } }` to `.qwen/settings.json` directly would silently discard all hook definitions. The `mcp.hbs` template therefore targets `.qwen/mcp.json` as an intermediate file; users must copy the `mcpServers` block into their actual `.qwen/settings.json` by hand. The `mcpServers` format (no type field, `command`/`args`/`env` for stdio, `httpUrl` for HTTP) is JSON.

---

## 4. Commands use Markdown format with `description` only — TOML format is deprecated

Qwen Code custom commands at `.qwen/commands/<name>.md` use YAML frontmatter with a single optional `description` field. The command name is derived from the filename; path separators are converted to colons for invocation (e.g. `git/commit.md` → `/git:commit`). There is no `name` field in the frontmatter. The older `.toml` command format (`description = "..."` / `prompt = """..."""`) is still parsed but documented as deprecated — the `qwen/command.hbs` template was originally written in this TOML format and has been updated to the modern Markdown format. The `command.hbs` template emits the modern Markdown format targeting `.qwen/commands/{{command.name}}.md`.

---

## 5. Skills follow Agent Skills standard — only `name` and `description` documented

Qwen Code skills at `.qwen/skills/<name>/SKILL.md` use the Agent Skills open standard with `name` and `description` as the documented frontmatter fields. No `allowed-tools`, `license`, or `compatibility` fields are mentioned in the Qwen Code docs. The `skill.hbs` template emits only `name` and `description` — more minimal than providers like Mistral Vibe (which includes all optional Agent Skills fields). Optional support files (scripts, templates, reference docs) can live alongside `SKILL.md` and are referenced via relative paths inside the skill body.

---

## 6. Sub-agents (`.qwen/agents/<name>.md`) are a distinct concept — not deployable

Qwen Code subagents at `.qwen/agents/<name>.md` use YAML frontmatter with `name`, `description`, `model` (optional, defaults to `inherit`), and `tools` (optional array of tool names). They run in independent context windows and are invoked automatically via description matching or explicitly by name in natural language. This is structurally similar to skills and Qoder CLI's subagents, but the `tools` field and independent execution context make them semantically distinct. A future `SubagentFeature` or `AgentFeature` would need to handle `model` and `tools`.

---

## 7. Hooks live in `.qwen/settings.json` alongside MCP — not deployable, 12 events

Qwen Code hooks are configured under the `"hooks"` key in `.qwen/settings.json` (project-level) or `~/.qwen/settings.json` (user-level). The structure uses event name → array of matcher groups, each with a `matcher` regex, optional `sequential` boolean, and a `hooks` array of `{ "type": "command", "command": "...", "name": "...", "description": "...", "timeout": N }` objects. Qwen Code supports 12 events: `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`, `SessionStart`, `SessionEnd`, `UserPromptSubmit`, `Stop`, `SubagentStart`, `SubagentStop`, `PreCompact`, and `Notification`. The shared `settings.json` file makes hook deployment unsafe without a merge strategy; no `HookFeature` currently exists.
