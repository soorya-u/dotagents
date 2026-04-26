# Augment (Auggie) — Integration Notes

Source: https://docs.augmentcode.com/cli/

Things observed in Auggie that could be integrated into dotagents:

---

## 1. Rules frontmatter (`type` and `description`)

Workspace rules files under `.augment/rules/` support YAML frontmatter with two fields:

```yaml
---
type: agent_requested
description: Apply when working on API routes
---
```

- `type`: `always_apply` (default) or `agent_requested` (agent auto-attaches based on description)
- `description`: required when `type` is `agent_requested`

The current `InstructionFeature` passes content as-is. If we expose these frontmatter fields as structured metadata on `InstructionFeature`, users could configure how their instructions are applied per-provider (Augment only currently).

---

## 2. Subagents — a new feature type

Auggie supports subagents: specialized custom agents stored as `.augment/agents/<name>.md` with YAML frontmatter:

```yaml
---
name: code-review
description: Reviews staged changes
model: claude-3-7-sonnet
tools: [view, codebase-retrieval, github-api]
disabled_tools: [save-file]
---
```

This could be modelled as a new `FeatureTrait` implementation (`SubagentFeature`), with a target of `.augment/agents/{{agent.name}}.md` for Augment. Relevant frontmatter fields: `name`, `description`, `color`, `model`, `tools`, `disabled_tools`.

---

## 3. Hooks — a new feature type

Auggie supports lifecycle hooks (`PreToolUse`, `PostToolUse`, `Stop`, `SessionStart`, `SessionEnd`) configured as JSON in `settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "launch-process",
        "hooks": [{ "type": "command", "command": "/path/to/script.sh", "timeout": 5000 }]
      }
    ]
  }
}
```

This could be modelled as a new `FeatureTrait` implementation (`HookFeature`), rendered into the auggie `settings.json` alongside the MCP block. Key fields: `event`, `matcher`, `command`, `timeout`.

---

## 4. SSE transport for MCP

Augment supports three MCP transport types: `stdio`, `http`, and `sse`. The current `ServerConfig` enum only models `Http` and `Stdio`. Adding an `Sse` variant:

```rust
#[serde(rename = "sse")]
Sse {
    url: String,
    headers: Option<HashMap<String, String>>,
}
```

would give full coverage for Augment MCP configs and future providers that use SSE.

---

## 5. Plugins bundle multiple feature types

Augment's plugin system (`.augment-plugin/plugin.json`) lets a single package bundle commands, agents, rules, hooks, skills, and MCP servers together. When dotagents' skills feature is fully implemented, a "bundle" concept that deploys multiple feature types from one source directory would map naturally onto this model.
