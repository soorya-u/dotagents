# Qoder CLI — Integration Notes

Sources:
- https://docs.qoder.com/en/cli/user-guide/command
- https://docs.qoder.com/en/cli/user-guide/subagent
- https://docs.qoder.com/cli/Skills
- https://docs.qoder.com/cli/hooks
- https://docs.qoder.com/cli/using-cli
- https://docs.qoder.com/cli/quick-start

Things observed in Qoder CLI that could be integrated into dotagents:

---

## 1. Commands require both `name` AND `description` in frontmatter — unique among surveyed providers

Qoder CLI command files at `.qoder/commands/<name>.md` must include both a `name` field and a `description` field in their YAML frontmatter. The `name` is the slash-command identifier (e.g. `name: git-commit` is invoked as `/git-commit`), and the filename must match it. Most other surveyed providers (OpenCode, KiloCode, Junie) only include `description` in the frontmatter and derive the command name from the filename alone. The `command.hbs` template emits both `name: {{command.name}}` and `description: {{command.description}}` — the only template in this project that renders `command.name` into the frontmatter body rather than using it solely for the target path.

---

## 2. Skills only document `name` and `description` — `allowed-tools` and `license`/`compatibility` not explicitly listed

Qoder CLI skills at `.qoder/skills/<name>/SKILL.md` follow the Agent Skills open standard but the docs only call out `name` and `description` as frontmatter fields. Unlike Copilot, CodeBuddy, and Mistral Vibe, Qoder does not document `allowed-tools`. The `skill.hbs` template conditionally emits `license` and `compatibility` (which align with the Agent Skills spec that Qoder references) but omits `allowed-tools`. If Qoder's parser is spec-compliant, the optional fields will be accepted; unknown fields are explicitly noted as silently ignored per the Agent Skills specification.

---

## 3. MCP project config is `.mcp.json` at the project root — standard `mcpServers` format, no type discriminator

Qoder CLI stores project-level MCP servers in `${project}/.mcp.json`, which is the Claude Code-compatible standard MCP configuration file. The format uses the `mcpServers` top-level key with `command`/`args`/`env` for stdio servers and `url`/`headers` for HTTP servers — no `type` discriminator field, identical to Junie and Kimi. The user-level MCP config is merged into `~/.qoder.json` (the global settings file that also stores model selection, update preferences, and API keys), so we cannot safely target that file. Only the project-level `.mcp.json` is deployed.

---

## 4. Hooks closely mirror Claude Code's `settings.json` format — but `settings.json` also contains permissions, and no HookFeature exists yet

Qoder CLI hooks are stored in `~/.qoder/settings.json` (user-level) and `${project}/.qoder/settings.json` (project-level) under a `"hooks"` key. The structure — event name → array of matcher groups, each with a `matcher` regex and a nested `hooks` array of `{ "type": "command", "command": "...", "timeout": N }` objects — is structurally identical to Claude Code's `settings.json` format. Qoder extends Claude Code's 7 events with 5 additional ones: `PostToolUseFailure`, `SubagentStart`, `SubagentStop`, `Notification`, and `PermissionRequest`. The same `settings.json` file also holds the `permissions` configuration, so targeting it directly for hooks would overwrite user permission rules. If a `HookFeature` is ever added, Qoder would be among the easiest providers to support with a near-identical template to Claude's.

---

## 5. Subagents (`.qoder/agents/<name>.md`) are structurally similar to skills but are a distinct concept

Qoder subagents at `.qoder/agents/<name>.md` use YAML frontmatter with `name`, `description`, and `tools` (a comma-separated list of tool names, defaulting to `*`). The body is a system prompt. This format is structurally very close to the Agent Skills `SKILL.md` format — `tools` plays the same role as `allowed-tools` in the spec. However, subagents are distinct from skills: they run in independent context windows with their own tool permissions, are invoked explicitly or implicitly by natural language, and can be chained. Deploying a skill as a subagent (or vice versa) would be semantically incorrect. A future `SubagentFeature` or `AgentFeature` would need to model at minimum `name`, `description`, and `tools`, with the body as the system prompt.

---

## 6. AGENTS.md is the primary instruction file — project-level and user-level both supported

Qoder CLI loads `AGENTS.md` from the project root (`${project}/AGENTS.md`) as the primary memory/instruction file. A user-level `~/.qoder/AGENTS.md` applies across all projects. The `/init` command generates `AGENTS.md` from codebase analysis; `#` in TUI appends directly to the project `AGENTS.md`; `/memory` opens a picker to edit either level. The `instruction.hbs` template is a plain pass-through targeting the project-root `AGENTS.md`, consistent with the standard cross-provider file.

---

## 7. The `.mcp.json` format does not expose `type`, but Qoder MCP CLI supports `sse` and `streamable-http` transports

When adding MCP servers via the CLI (`qodercli mcp add -t sse ...`), Qoder supports three transport types: `stdio`, `sse`, and `streamable-http`. The project-level `.mcp.json` file uses the standard `mcpServers` format without a `type` field — only `command`/`args` for stdio or `url` for HTTP transports are written to the file. There is no way to represent `sse` vs `streamable-http` vs plain `http` in the file format. `ServerConfig` models a single HTTP variant, so all HTTP servers are rendered without a type discriminator. Users who need SSE or streaming HTTP servers must add them manually via `qodercli mcp add -t sse ...`.
