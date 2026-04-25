# Kimi (Moonshot) — Integration Notes

Sources:
- https://www.kimi-cli.com/en/customization/mcp.html
- https://www.kimi-cli.com/en/customization/plugins.html
- https://www.kimi-cli.com/en/customization/hooks.html
- https://www.kimi-cli.com/en/customization/skills.html
- https://www.kimi-cli.com/en/customization/agents.html
- https://www.kimi-cli.com/en/configuration/config-files.html

Things observed in Kimi Code CLI that could be integrated into dotagents:

---

## 1. MCP config is `~/.kimi/mcp.json` with the simplest standard format — no type discriminator

MCP server configuration lives in a dedicated `~/.kimi/mcp.json` file (separate from the main `config.toml`), using the `mcpServers` top-level key. The format is the most minimal of any surveyed provider: no `type` field on either server variant — HTTP servers are identified by the presence of `url`, stdio servers by `command`. The fields map directly to `ServerConfig`: `command`, `args`, `env` for stdio; `url`, `headers` for HTTP. The `mcp.hbs` template is structurally identical to Junie's. Because `mcp.json` is a dedicated, standalone file, targeting it directly is safe with no overwrite risk.

---

## 2. No commands concept — skills serve as the invocable-workflow mechanism

Kimi Code CLI has no dedicated "commands" or "slash commands" directory. Instead, skills double as invocable prompt templates: `/skill:<name>` injects the full `SKILL.md` content into the conversation as a prompt, and `/flow:<name>` executes a flow skill as a multi-step automated workflow. There is no `commands` entry in `provider.toml` for Kimi. Users who want reusable prompt templates should use the `skills` feature instead and invoke them with `/skill:name`.

---

## 3. Skills fully follow the Agent Skills standard; `allowed-tools` is not documented

Kimi skills use the canonical `<name>/SKILL.md` layout under `.kimi/skills/` with YAML frontmatter fields `name`, `description`, `license`, `compatibility`, and `metadata`. Kimi does not document an `allowed-tools` field (unlike Copilot and CodeBuddy), so `skill.hbs` omits it. Kimi also supports a "flat" variant (a single `.md` file directly in the skills directory), but dotagents always emits the canonical subdirectory layout, which takes precedence when both forms share a name. Kimi loads skills from multiple directories — `.kimi/skills/`, `.claude/skills/`, `.codex/skills/`, `.agents/skills/` — so skills deployed to `.kimi/skills/` are available only to Kimi, while deploying to `.agents/skills/` (the generic group) would also be visible to other tools.

---

## 4. Flow skills add `type: flow` frontmatter and a Mermaid/D2 diagram — not modelled in `SkillMetadata`

A special skill variant called a "flow skill" embeds a `type: flow` frontmatter field and a fenced Mermaid or D2 code block as the skill body, which Kimi executes as a multi-step automated workflow via `/flow:<name>`. `SkillMetadata` has no `type` field, so the flow diagram would pass through in `skill.content` verbatim and `type: flow` would need to be included by the user in their skill body frontmatter manually. A future `SkillMetadata` extension could add an optional `type` field to support this.

---

## 5. Hooks are in `~/.kimi/config.toml` alongside API keys — cannot be safely deployed

Kimi hooks are defined as `[[hooks]]` array entries in `~/.kimi/config.toml`. The same file contains API keys, provider connection details, model definitions, loop control, and background task settings. Targeting it directly would overwrite all user configuration. Kimi's hook system supports 13 lifecycle events — a superset of Claude Code's 7: it adds `PostToolUseFailure`, `StopFailure`, `SubagentStart`, `SubagentStop`, `PostCompact`, and `Notification`. The hook field structure (`event`, `command`, `matcher`, `timeout`) is similar to Claude Code's but uses `event` instead of `events`, and hooks are stored in a TOML array rather than a JSON object keyed by event name. If a `HookFeature` is ever added to dotagents, a Kimi-specific TOML template would require the `[[hooks]]` syntax rather than JSON — a different serialization format from every other provider.

---

## 6. Plugins are a Kimi-specific executable-tool system with no analog in dotagents

Kimi Code CLI has a plugin system (`~/.kimi/plugins/<name>/plugin.json`) that declares executable tools the AI can directly invoke — distinct from skills, which provide knowledge-based guidance. A plugin defines `name`, `version`, `description`, and a `tools` array where each tool specifies a `command` (array of strings), `parameters` (JSON Schema), and optional `inject` for credential injection from Kimi's LLM provider config. Plugins are installed via `kimi plugin install`, not by dropping files into a directory. There is no `FeatureTrait` implementation for plugins, and the install-based deployment model is incompatible with dotagents' file-render approach. Users who need plugins must install them manually via the CLI.

---

## 7. Custom agents are bespoke YAML files with no auto-discovery path

Kimi custom agents are YAML files loaded explicitly with `kimi --agent-file /path/to/agent.yaml`. They define `name`, `system_prompt_path`, `tools` (Python module:class strings), `exclude_tools`, `subagents`, and an optional `extend` key to inherit from a built-in or another file. There is no auto-discovered directory (like `.kimi/agents/`) — agents must be explicitly referenced at startup. This makes them incompatible with dotagents' deploy model, which writes files to fixed target paths. Subagent definitions nest within the agent YAML via relative paths. No `AgentFeature` is planned; users must author and manage agent files manually.
