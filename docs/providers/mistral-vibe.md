# Mistral Vibe — Integration Notes

Sources:
- https://docs.mistral.ai/mistral-vibe/agents-skills
- https://docs.mistral.ai/mistral-vibe/overview
- https://docs.mistral.ai/mistral-vibe/terminal
- https://docs.mistral.ai/mistral-vibe/local
- https://docs.mistral.ai/mistral-vibe/terminal/configuration

Things observed in Mistral Vibe that could be integrated into dotagents:

---

## 1. MCP configuration is TOML `[[mcp_servers]]` inside `config.toml` — cannot be safely targeted directly

Mistral Vibe stores MCP servers as a TOML array-of-tables under `[[mcp_servers]]` in `~/.vibe/config.toml` (global) or `.vibe/config.toml` (project). The same file contains API keys, provider presets, model settings, tool permissions, session config, and other runtime parameters — targeting it directly is destructive. The `mcp.hbs` template outputs valid TOML `[[mcp_servers]]` blocks to `.vibe/mcp.toml` as an intermediate file. Users must copy the generated blocks into the appropriate `[[mcp_servers]]` section of their `config.toml` manually. This is the same pattern used for goose's `extensions.yaml`.

---

## 2. MCP template outputs TOML — `env` rendered as TOML inline table; `headers` dropped

Mistral Vibe's config format is TOML. The `env` field for stdio servers is documented as a TOML inline table: `env = { "KEY" = "value", "KEY2" = "value2" }`. The `mcp.hbs` template renders this with an `{{#each}}` loop producing valid TOML inline-table syntax — `{{json this}}` produces JSON-quoted string values which are also valid TOML basic strings. The `headers` field for HTTP servers uses the same inline-table syntax but is not rendered by the template because `ServerConfig.Http` has a `headers` map that would need the same treatment; this is a known gap. The `args` field is a TOML array that accepts JSON array literal syntax, so `{{json this.args}}` is valid. `cwd` and `env_file` are not documented for Mistral Vibe's `[[mcp_servers]]` and are not emitted.

---

## 3. MCP `transport` has a third value `"streamable-http"` not modelled in `ServerConfig`

Mistral Vibe supports three transport types: `"stdio"`, `"http"`, and `"streamable-http"`. The `ServerConfig` enum has only two variants (`Http` and `Stdio`), so there is no way to distinguish a regular HTTP server from a streaming-HTTP server via dotagents' data model. The template maps all HTTP `ServerConfig` entries to `transport = "http"`. Users who need `transport = "streamable-http"` must edit the generated `.vibe/mcp.toml` file manually before merging.

---

## 4. Skills follow the Agent Skills standard; `user-invocable` is a Mistral Vibe-specific field not modelled

Mistral Vibe skills use the canonical `<name>/SKILL.md` layout under `.vibe/skills/` with standard YAML frontmatter: `name`, `description`, `license`, `compatibility`, and `allowed-tools`. Mistral Vibe's own documentation also shows a `user-invocable: true` field that controls whether the skill can be invoked explicitly via `/skill:<name>`. This field has no analog in `SkillMetadata` and is not rendered by `skill.hbs`. Users who need it must include it in the skill body's leading content or edit the generated file. `allowed-tools` is shown as a YAML list in the docs but the `SkillMetadata` field is a string; the template outputs it as a comma-separated string which Mistral Vibe may parse differently depending on its YAML reader.

---

## 5. AGENTS.md only loaded from project root — no subdirectory support

Mistral Vibe loads `AGENTS.md` only from the workspace root. The docs explicitly note: "This feature is currently only functional when an AGENTS.md file is in the root of the workspace." Subdirectory `AGENTS.md` files (supported by KiloCode and other tools) are not loaded. The `instruction.hbs` template is a plain pass-through targeting `AGENTS.md` at the project root, consistent with the standard cross-provider usage.

---

## 6. No commands or hooks concept — no templates needed

Mistral Vibe does not have a user-definable commands directory or a file-based hooks system. Slash commands are built-in CLI commands (`/config`, `/mcp`, `/skill:<name>`, etc.). Skills serve as the invocable prompt-template mechanism via `/skill:<name>`. There is no hooks configuration section documented in `config.toml`. No `commands` or `hooks` entries are included in `provider.toml`.

---

## 7. Custom agents are global TOML files in `~/.vibe/agents/` with no per-project auto-discovery

Custom agent profiles are `.toml` files in `~/.vibe/agents/` loaded with `vibe --agent <name>`. Agent TOML fields include `display_name`, `description`, `safety` (`"safe"` / `"neutral"` / `"destructive"` / `"yolo"`), `auto_approve`, `enabled_tools`, `disabled_tools`, `agent_type` (`"subagent"` for subagents), `active_model`, and `system_prompt_id`. There is no per-project agents directory, and the `--agent` flag requires explicit invocation — there is no auto-discovery at startup. These TOML agent files are incompatible with dotagents' deploy model (they combine with a `system_prompt_id` pointing to a separate `~/.vibe/prompts/` markdown file). No agent feature is planned.
