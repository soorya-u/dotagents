# Deep Agents — Integration Notes

Sources:
- https://docs.langchain.com/oss/python/deepagents/customization
- https://docs.langchain.com/oss/python/deepagents/cli/configuration
- https://docs.langchain.com/oss/python/deepagents/cli/mcp-tools
- https://docs.langchain.com/oss/python/deepagents/skills
- https://docs.langchain.com/oss/python/deepagents/deploy
- https://docs.langchain.com/oss/python/deepagents/subagents

Things observed in Deep Agents that could be integrated into dotagents:

---

## 1. MCP supports SSE as a distinct transport type

Deep Agents MCP config (`mcpServers`) supports three transport types: `stdio` (no `type` field needed), `http`, and `sse`. The `sse` type has the same shape as `http` (`url` + `headers`) but is its own named variant. The current `ServerConfig::Http` variant serializes to `"type": "http"`, which maps only to streamable HTTP. Adding an `Sse` variant (or extending `Http` with a `subtype` field) would allow dotagents to express SSE servers in `mcp.jsonc` and render `"type": "sse"` for providers that distinguish the two.

---

## 2. Hooks are fire-and-forget, array-based — different from Cursor hooks

Deep Agents CLI hooks live at `~/.deepagents/hooks.json` with a flat `hooks` array where each entry has `command: [str, ...]` and optional `events: [str, ...]` filter. Unlike Cursor hooks (which can block or gate actions), these are purely observational — fire-and-forget in a background thread. Events include `session.start`, `session.end`, `user.prompt`, `permission.request`, `tool.error`, `task.complete`, `context.compact`. If a `HookFeature` is ever added to dotagents, both the blocking (Cursor) and fire-and-forget (Deep Agents) patterns would need to be expressible, likely via different template targets and formats.

---

## 3. Subagents use `deepagents.toml` + `AGENTS.md` — a different format than Claude/Codex

Deep Agents deploy subagents live in `subagents/<name>/` subdirectories. Each requires a `deepagents.toml` (TOML with `[agent].name`, `[agent].description`, optional `[agent].model`) alongside an `AGENTS.md` system prompt, and optionally a `skills/` folder and `mcp.json`. This is a two-file pattern (config TOML + instructions markdown) rather than the single markdown-with-frontmatter file used by Claude (`.claude/agents/*.md`) and Codex (`.codex/agents/*.md`). A `SubagentFeature` would need to handle both patterns, or provide a separate template per provider.

---

## 4. Skills path is `skills/` (project-relative), not `.agents/skills/`

For the deploy workflow, Deep Agents scans `skills/<name>/SKILL.md` relative to `deepagents.toml`. The CLI-level auto-discovery path is not explicitly documented as `.agents/skills/`. Targeting `skills/{{skill.name}}/SKILL.md` (without a leading dot) works for the deploy convention. If the Agent Skills standard path `.agents/skills/` is also supported by the CLI, a future config option or second provider entry could target both.

---

## 5. `deepagents.toml` deploy config is a new file type

The deploy workflow uses a `deepagents.toml` at the project root with `[agent]` (name, model), optional `[sandbox]` (provider, image, scope), and optional `[auth]` (provider) sections. This is not an instruction, MCP, or skill file — it is agent identity and infrastructure config. No existing `FeatureTrait` covers this. A `DeployConfigFeature` could render it from dotagents variables (`var.agent_name`, `var.model`, etc.), letting users manage deploy config alongside their other agent config files.
