# Gemini CLI — Integration Notes

Sources:
- https://geminicli.com/docs/cli/tutorials/skills-getting-started/
- https://geminicli.com/docs/hooks/reference/
- https://geminicli.com/docs/hooks/
- https://geminicli.com/docs/tools/mcp-server/
- https://geminicli.com/docs/core/subagents/
- https://geminicli.com/docs/cli/custom-commands/
- https://geminicli.com/docs/cli/gemini-md/

Things observed in Gemini CLI that could be integrated into dotagents:

---

## 1. Commands are TOML files, not Markdown — a unique format requiring a wrapping template

Gemini CLI commands live in `.gemini/commands/<name>.toml` with required `prompt` (string) and optional `description` (string) fields. This is the only provider that uses TOML for commands rather than Markdown. The existing `command.hbs` already wraps `{{command.content}}` inside a TOML `prompt = """..."""` block. This works but means the command source (`.dotagents/commands/<name>.md`) body becomes the TOML `prompt` value verbatim, and the `description` is pulled from `{{command.description}}` — a frontmatter field not currently guaranteed by `CommandFeature`. Confirming that `CommandFeature` exposes arbitrary frontmatter fields via `to_value()` would let this description field work reliably.

---

## 2. MCP has two distinct HTTP transport types with different JSON keys

Gemini CLI differentiates between SSE (`url` key) and streamable HTTP (`httpUrl` key) in `mcpServers`. All other providers use a single `url` field for any HTTP/remote endpoint. The current `ServerConfig::Http` variant maps to `url` in the mcp.hbs template (SSE-compatible). Adding an `Http` sub-type or a separate `StreamableHttp` variant to `ServerConfig` would let users express `httpUrl`-style servers in `mcp.jsonc` and render them correctly for Gemini. Until then, users needing streamable HTTP must use a custom template.

---

## 3. MCP config is embedded in `settings.json`, not a standalone file

Unlike every other provider, Gemini CLI does not have a dedicated MCP config file — the `mcpServers` object lives inside `.gemini/settings.json` alongside hooks, agent overrides, model config, and other settings. The current mcp.hbs writes `{ "mcpServers": { ... } }` to `.gemini/settings.json` — this is valid if the user has no other settings, but will overwrite any existing config on re-deploy. A merge-aware deploy mode (read-modify-write instead of overwrite) would solve this across all providers where config shares a file with non-dotagents settings. This is the same concern as for hooks in Factory Droid and Deep Agents.

---

## 4. Instructions file is `GEMINI.md` by default, but configurable to `AGENTS.md`

Gemini CLI loads `GEMINI.md` as the context/instructions file by default. However, `settings.json` supports `context.fileName` as an array, meaning users can configure it to also load `AGENTS.md`, `CONTEXT.md`, or any other name. Since `AGENTS.md` is the cross-provider standard, an alternative provider entry targeting `AGENTS.md` could be offered alongside the default `GEMINI.md` target.

---

## 5. Subagents use rich frontmatter not in current `SubagentFeature` designs

Gemini custom subagents (`.gemini/agents/<name>.md`) have a richer frontmatter schema than Claude/Codex agents: `name`, `description`, `kind` (`local`/`remote`), `tools` (array with wildcard support like `mcp_*`), `mcpServers` (inline MCP config object), `model`, `temperature`, `max_turns`, `timeout_mins`. The `mcpServers` field embedded directly in an agent definition frontmatter is especially novel — it isolates MCP servers to a single subagent. A `SubagentFeature` would need to handle this as an optional nested object rendered in YAML frontmatter.

---

## 6. Hooks use `BeforeTool`/`AfterTool` regex matchers — most expressive hook system seen so far

Gemini hooks live in `.gemini/settings.json` under `"hooks"` (same file as MCP). Hook events: `SessionStart`, `SessionEnd`, `BeforeAgent`, `AfterAgent`, `BeforeModel`, `AfterModel`, `BeforeToolSelection`, `BeforeTool`, `AfterTool`, `PreCompress`, `Notification`. The `BeforeTool`/`AfterTool` matchers are full regex (e.g., `"write_.*"`). Hooks can rewrite tool arguments via `hookSpecificOutput.tool_input`, inject a synthetic LLM response via `hookSpecificOutput.llm_response` (skipping the real LLM call entirely), and chain tool calls via `hookSpecificOutput.tailToolCallRequest`. This is the most capable hook system across all providers reviewed.
