# Amp — Integration Notes

Source: https://ampcode.com/manual

Things observed in Amp that could be integrated into dotagents:

---

## 1. Glob-scoped instructions (AGENTS.md frontmatter)

Amp's `AGENTS.md` supports YAML frontmatter with a `globs` field that scopes the instruction block to only be sent to the model when the agent is working on matching files:

```yaml
---
globs:
  - '**/*.ts'
  - '**/*.tsx'
---
Only apply these instructions for TypeScript files.
```

This would be a useful addition to dotagents' `instructions` feature — allow the user to declare a `globs` field in their `INSTRUCTIONS.md` frontmatter and have it passed through to providers that support it (currently Amp only).

---

## 2. `includeTools` on MCP servers

Both workspace MCP servers and skill-bundled MCP servers accept an `includeTools` field (array of tool names or glob patterns) to limit which tools are exposed from a server:

```json
"chrome-devtools": {
  "command": "npx",
  "args": ["-y", "chrome-devtools-mcp@latest"],
  "includeTools": ["navigate_*", "take_screenshot"]
}
```

Our `mcp.schema.json` and `McpFeature` don't model this field. Adding `include_tools: Option<Vec<String>>` to `ServerConfig` (both variants) and surfacing it in the Amp MCP template would give users control over tool exposure per provider.

---

## 3. Skills can bundle their own MCP servers

Each skill can ship a co-located `mcp.json` alongside its `SKILL.md`. Those servers are loaded automatically when the skill is activated, scoped only to that skill's context.

When the `skills` feature is implemented in dotagents, the renderer could optionally write a `mcp.json` next to each `SKILL.md` if the skill declares MCP servers. This is a natural extension of the Skills + MCP features together.

---

## 4. Checks — a new feature type

Amp has a `checks` feature: markdown files with YAML frontmatter (`name`, `description`, `severity-default`, `tools`) placed in `.agents/checks/`. These are code-review criteria the agent runs automatically on diffs.

The format is very similar to commands. This could be modelled as a new `FeatureTrait` implementation in dotagents (`CheckFeature`), with a target of `.agents/checks/{{check.name}}.md` for Amp. Relevant frontmatter fields: `name`, `description`, `severity`, `tools`.
