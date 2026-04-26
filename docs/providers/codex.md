# Codex — Integration Notes

Source: https://developers.openai.com/codex/

Things observed in Codex that could be integrated into dotagents:

---

## 1. MCP config is TOML, not JSON

Codex stores MCP server configuration in `config.toml` under `[mcp_servers.<name>]` sections — the only provider so far that uses TOML rather than JSON for MCP config:

```toml
[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]

[mcp_servers.context7.env]
MY_ENV_VAR = "value"

[mcp_servers.figma]
url = "https://mcp.figma.com/mcp"
bearer_token_env_var = "FIGMA_OAUTH_TOKEN"
```

The `codex/mcp.hbs` template handles this by using TOML section headers (`[mcp_servers.name]`) and nested subtables for `env` and `http_headers`. One limitation: the existing `{{json}}` helper renders JSON syntax, which is valid for TOML arrays (`args`) but not for TOML inline tables. A `{{toml_table}}` Handlebars helper would remove this limitation and open the door for more TOML-based providers.

---

## 2. Codex-specific MCP server fields

Beyond `command`/`args`/`url`/`env`, Codex MCP servers support additional configuration fields not modelled in `ServerConfig`:

| Field | Type | Purpose |
|---|---|---|
| `enabled` | `bool` | Disable a server without deleting it |
| `required` | `bool` | Fail startup if this server can't initialize |
| `enabled_tools` | `string[]` | Tool allow-list |
| `disabled_tools` | `string[]` | Tool deny-list (applied after `enabled_tools`) |
| `startup_timeout_sec` | `number` | Override startup timeout (default 10s) |
| `tool_timeout_sec` | `number` | Override per-tool timeout (default 60s) |
| `bearer_token_env_var` | `string` | HTTP only — env var name holding a bearer token |
| `env_vars` | `string[]` | Forward named env vars from parent process to server |

`enabled` and `disabled_tools` already have counterparts in `CommonConfig`. Adding `required`, `enabled_tools`, `startup_timeout_sec`, `tool_timeout_sec`, and `bearer_token_env_var` would give full coverage for Codex MCP configs.

---

## 3. Command-execution rules — a new feature type

Codex has a `.rules` DSL (`.codex/rules/default.rules`) for controlling which shell commands the agent can run outside the sandbox. This is different from all other providers: instead of instructions to the model, these are programmatic allow/deny/prompt rules evaluated at execution time:

```
prefix_rule(
    pattern = ["gh", "pr", "view"],
    decision = "prompt",
    justification = "Viewing PRs is allowed with approval",
)
```

This could be modelled as a new `FeatureTrait` (`ExecRuleFeature`) with a target of `.codex/rules/{{rule.name}}.rules`. No other provider has an equivalent yet.

---

## 4. Custom subagents as TOML files

Codex defines custom subagents as standalone TOML files in `.codex/agents/` with required fields `name`, `description`, and `developer_instructions`, plus optional fields like `model`, `sandbox_mode`, `mcp_servers`, and `skills.config`.

This is the same subagent concept noted for Claude, Augment, and Cline, but with a TOML file format rather than a markdown-with-frontmatter format. A shared `SubagentFeature` would need to handle both formats, or the template layer could render a provider-appropriate format from a common markdown source.

---

## 5. Skills path uses `.agents/skills/` — cross-provider standard

Codex places skills at `.agents/skills/<name>/SKILL.md` — the same path used by Amp (`.agents/skills/`). This is the path defined by the [Agent Skills open standard](https://github.com/agentskills/spec). The existing codex `provider.toml` had the wrong path (`.codex/skills/`); it has been corrected to `.agents/skills/`.

If more providers converge on `.agents/skills/`, a single dotagents skill deployment could target multiple providers simultaneously by writing to the shared path once.
