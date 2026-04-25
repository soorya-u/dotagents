# KiloCode — Integration Notes

Sources:
- https://kilo.ai/docs/cli/overview
- https://kilo.ai/docs/mcp/kilocode
- https://kilo.ai/docs/mcp/cli
- https://kilo.ai/docs/customize/modes
- https://kilo.ai/docs/customize/rules
- https://kilo.ai/docs/customize/instructions
- https://kilo.ai/docs/customize/subagents
- https://kilo.ai/docs/customize/agents-md
- https://kilo.ai/docs/customize/workflows
- https://kilo.ai/docs/customize/skills
- https://kilo.ai/docs/customize/context/kilocodeignore

Things observed in KiloCode that could be integrated into dotagents:

---

## 1. Instructions target `AGENTS.md`; `kilo.jsonc` `instructions` key takes higher priority

KiloCode auto-loads `AGENTS.md` (falling back to `AGENT.md`) from the project root at priority 3 out of 4. The highest-priority project-level source is the `instructions` array key inside `.kilo/kilo.jsonc`, which accepts file paths to inject at priority 2. The `instruction.hbs` template is a plain pass-through targeting `AGENTS.md` — the cross-provider standard already consumed by Claude, Gemini, goose, and Copilot. Users who want to use the higher-priority `instructions` key must add the path manually to their `kilo.jsonc`. KiloCode also supports subdirectory `AGENTS.md` files, but dotagents has no concept of per-subdirectory instruction deployment; users wanting subdirectory files should create them manually.

---

## 2. MCP uses `type: "local"` / `type: "remote"`, an array `command`, and `environment` — all differ from the internal `ServerConfig` model

KiloCode's MCP format (stored under `"mcp" > "servers"` in `.kilo/kilo.jsonc`) has three structural differences from `ServerConfig`:

1. **Type names**: `"local"` (stdio) and `"remote"` (HTTP) instead of `"stdio"` / `"http"`.
2. **Command is an array**: `"command": ["npx", "-y", "@pkg/server"]` combines the executable and args into one array field; there is no separate `"args"` key. The template reconstructs this by prepending `{{this.command}}` then appending each element of `this.args`: `["{{this.command}}"{{#each this.args}}, {{json this}}{{/each}}]`.
3. **`environment` not `env`**: The environment variable map is keyed `"environment"` in KiloCode output, not `"env"`.

Additionally, each server entry includes `"enabled": true` — no analog in `ServerConfig`; hardcoded to `true` in the template.

Because `.kilo/kilo.jsonc` also contains rules, permissions, model settings, and other project config, targeting it directly would be destructive. The template writes to `.kilo/mcp.json` as an intermediate file. Users must manually copy the `"mcp"` block from that file into their `kilo.jsonc`.

---

## 3. HTTP MCP servers: `"type": "remote"`, no headers field documented

KiloCode remote MCP servers use `"type": "remote"` with a `"url"` field — consistent with `ServerConfig.Http.url`. However, KiloCode's documented remote server format does not include a `"headers"` field. The `mcp.hbs` template omits headers for remote servers; users needing auth headers must edit `kilo.jsonc` directly.

---

## 4. Skills use standard Agent Skills format at `.kilo/skills/`; `allowed-tools` is not documented

KiloCode skills follow the Agent Skills open specification: a `SKILL.md` file with `name`, `description`, optional `license`, `compatibility`, and `metadata` YAML frontmatter, plus Markdown body. KiloCode's own documentation does not mention an `allowed-tools` frontmatter field (unlike Copilot and CodeBuddy), so `skill.hbs` omits it. The project-level path `.kilo/skills/<name>/SKILL.md` is the canonical location; KiloCode also reads from `.claude/skills/` and `.agents/skills/` for cross-tool compatibility. The `name` field must match the parent directory name.

---

## 5. Commands / Workflows live at `.kilo/commands/`; legacy `.kilocode/workflows/` auto-migrated

KiloCode slash commands (called "workflows" in the legacy extension) are Markdown files at `.kilo/commands/<name>.md`. The file may include optional YAML frontmatter with `description` (shown in command picker), `agent` (which mode to invoke), `model` (model override), and `subtask` (boolean — runs as a sub-agent session). The `command.hbs` template outputs `description` frontmatter and passes through `command.content` unchanged. Users who need `agent`, `model`, or `subtask` frontmatter fields can include them directly in their command body since dotagents has no dedicated metadata fields for those. The legacy path `.kilocode/workflows/` is automatically migrated by the extension on startup; the old `provider.toml` entry targeted `.kilocode/workflows/` — this has been corrected to `.kilo/commands/`.

---

## 6. Pre-existing `command.hbs` contained a non-existent `handoffs` frontmatter block

The original `command.hbs` template iterated over `command.handoffs` (label/agent/prompt/send) — a field that does not exist in `CommandMetadata`. This would have rendered as empty YAML in the frontmatter for all deployed command files. The template has been replaced with a minimal `description` pass-through matching the documented KiloCode workflow format.

---

## 7. Agents and modes (`.kilo/agents/<name>.md`) are not modelled by any current feature type

KiloCode custom agents and modes share the same file format: `.kilo/agents/<name>.md` with YAML frontmatter including `description`, `mode` (`primary` / `subagent` / `all`), `color`, `permission`, `model`, `steps`, `temperature`, `hidden`, and `disable`. The `mode: subagent` variant is analogous to CodeBuddy's subagents. Neither `mode` nor the KiloCode-specific fields (`color`, `permission`, `steps`, `temperature`, `hidden`, `disable`) have analogs in `SkillMetadata`. A future `AgentFeature` or `SubagentFeature` would need to model these fields. For now, users must create `.kilo/agents/` files manually.
