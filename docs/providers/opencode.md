# OpenCode — Integration Notes

Sources:
- https://opencode.ai/docs/config/
- https://opencode.ai/docs/tools/
- https://opencode.ai/docs/rules/
- https://opencode.ai/docs/agents/
- https://opencode.ai/docs/commands/
- https://opencode.ai/docs/mcp-servers/
- https://opencode.ai/docs/skills/
- https://opencode.ai/docs/custom-tools/

Things observed in OpenCode that could be integrated into dotagents:

---

## 1. MCP uses `"type": "local"` / `"type": "remote"` and a combined command array — same shape as KiloCode

OpenCode's MCP format (under the `"mcp"` key in `opencode.json`) uses `"type": "local"` for stdio servers and `"type": "remote"` for HTTP servers — identical naming to KiloCode. Local servers use a single `"command"` array combining the executable and all arguments (`"command": ["npx", "-y", "@pkg/server"]`), with no separate `"args"` key. The template reconstructs this from `ServerConfig` by prepending the command string: `["{{this.command}}"{{#each this.args}}, {{json this}}{{/each}}]`. HTTP servers use `"url"` and optional `"headers"`. Both variants support an `"enabled"` boolean (hardcoded `true` in the template).

Additionally, OpenCode local servers use `"environment"` instead of `"env"` for environment variable maps. The `{{json this.env}}` helper maps directly to this field since it outputs the same JSON object representation; only the key name differs.

---

## 2. MCP template targets `opencode.json` directly — overwrites any existing project config

OpenCode's project config file (`opencode.json` at the project root) is designed to be committed to Git and typically does not contain API keys (those go in global config or environment variables). Because it is a dedicated, opencode-only file, the MCP template targets it directly rather than using an intermediate file.

The trade-off: `opencode.json` is a single file that also holds `agent`, `command`, `model`, `instructions`, `permission`, and other settings. Deploying the MCP feature will overwrite the entire file with only the `mcp` section. Users who have other project config settings in `opencode.json` must manually merge the deployed output with their existing config. To avoid this, they can keep their other opencode settings in the global `~/.config/opencode/opencode.json` and let the project config hold only MCP.

---

## 3. Skills use standard Agent Skills format; `metadata` map and `allowed-tools` are not rendered

OpenCode skills at `.opencode/skills/<name>/SKILL.md` follow the Agent Skills specification: `name`, `description`, `license`, `compatibility`, and `metadata` (string-to-string map) frontmatter fields. OpenCode does not document an `allowed-tools` field, so `skill.hbs` omits it. The `metadata` field (a map of arbitrary key-value pairs) has no analog in `SkillMetadata`, which models it as a flat struct. Metadata keys are silently dropped. OpenCode also loads skills from `.claude/skills/` and `.agents/skills/` for compatibility, so deploying to any of those paths would also work.

---

## 4. Pre-existing `command.hbs` was a bare pass-through with a wrong target path — both fixed

The original `command.hbs` emitted `{{command.content}}` with no frontmatter, and `provider.toml` targeted `.opencode/command/{{command.name}}.md` (singular `command`). The docs show the correct path as `.opencode/commands/` (plural) and document a `description` frontmatter field (shown in the TUI picker). Both issues are fixed: `command.hbs` now outputs a `description` YAML frontmatter header before the content, and the target path uses the plural `commands/` directory. The `agent` and `model` frontmatter fields documented for commands are not in `CommandMetadata` and are therefore not rendered; users who need them can include them in their command body's leading frontmatter.

---

## 5. The `instructions` config key in `opencode.json` accepts glob patterns and remote URLs — not modelled

Beyond `AGENTS.md`, OpenCode supports an `instructions` array in `opencode.json` that accepts relative file paths, glob patterns (e.g., `"packages/*/AGENTS.md"`), and remote URLs. This is a more powerful instruction aggregation mechanism than the single-file `InstructionFeature`. The `instruction.hbs` template targets only `AGENTS.md` — the primary auto-loaded instruction file. Users who need multi-file or glob-pattern instructions must manually add the `instructions` array to their `opencode.json`. If `opencode.json` is also managed by the MCP feature, users must merge both sections by hand.

---

## 6. Agents are Markdown files with rich YAML frontmatter — not deployable via any current feature

OpenCode custom agents live at `.opencode/agents/<name>.md` (or `~/.config/opencode/agents/`) with YAML frontmatter: `description`, `mode` (`"primary"` / `"subagent"` / `"all"`), `model`, `temperature`, `top_p`, `steps`, `hidden`, `color`, `permission` (per-tool map), and `tools` (deprecated). The body is the system prompt. This is structurally similar to Junie's command format, but the field set is far richer and agent-specific. A future `AgentFeature` would need to model at minimum `description`, `mode`, `model`, and `permission`. For now, users must create agent files manually. The `opencode agent create` CLI command can scaffold them interactively.

---

## 7. Custom tools are TypeScript/JavaScript files — completely outside dotagents' deployment model

OpenCode custom tools are `.ts` or `.js` files in `.opencode/tools/` that export objects using the `tool()` helper from `@opencode-ai/plugin`. Each file export becomes a callable tool with a name, description, Zod-typed argument schema, and an async `execute` function. These files can shell out to Python, bash, or any other language internally. There is no file-based manifest or YAML frontmatter — the definition IS the TypeScript code. No `FeatureTrait` implementation is feasible for executable tool definitions; users must manage these files manually.
