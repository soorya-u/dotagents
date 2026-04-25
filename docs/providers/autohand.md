# Autohand — Integration Notes

Source: https://autohand.ai/docs/

Things observed in Autohand that could be integrated into dotagents:

---

## 1. `allowed-tools` field in skill frontmatter

Autohand skills support an `allowed-tools` frontmatter field (space-separated tool names) that restricts which tools the agent may use when executing the skill:

```yaml
---
name: deploy-staging
description: Build and deploy to the staging environment.
allowed-tools: read_file run_command
---
```

The `allowed_tools` field already exists in dotagents' `SkillMetadata` struct but was not used in the Amp or Augment `skill.hbs` templates (those providers do not support it). The autohand `skill.hbs` exposes it via `{{skill.[allowed-tools]}}`. If Amp or Augment ever add this field, their templates already have the data available.

---

## 2. MCP `autoConnect` field

Autohand's MCP config supports an optional `autoConnect: boolean` (default `true`) per server. Setting it to `false` defers connection until the user explicitly runs `/mcp connect <name>`, which is useful for heavy or rarely-used servers:

```json
{
  "name": "heavy-server",
  "transport": "stdio",
  "command": "node dist/index.js",
  "autoConnect": false
}
```

Adding `auto_connect: Option<bool>` to `ServerConfig` (both variants) and surfacing it in the autohand `mcp.hbs` template would let users control startup connection behavior per provider.

---

## 3. SSE transport for MCP

Autohand supports three MCP transport types: `stdio`, `http`, and `sse`. The current `ServerConfig` enum only models `Http` and `Stdio`. Adding an `Sse` variant:

```rust
#[serde(rename = "sse")]
Sse {
    url: String,
    headers: Option<HashMap<String, String>>,
}
```

would enable full coverage for Autohand (and Augment, which also supports SSE). The autohand `mcp.hbs` template uses `{{this.type}}` as the `transport` value, so an `Sse` variant would flow through correctly.

---

## 4. MCP config uses array-of-servers format with named entries

Autohand's `config.json` stores MCP servers as an array rather than a keyed object:

```json
{
  "mcp": {
    "servers": [
      { "name": "my-server", "transport": "stdio", "command": "...", "args": [...] }
    ]
  }
}
```

The current `McpFeature` serialises `servers` as a `HashMap<String, ServerConfig>`. The template bridges this by using `{{@key}}` as the `name` field and rendering an array. If more providers adopt this pattern, it may be worth modelling the array format natively in `McpFeature` or adding a second serialisation path.
