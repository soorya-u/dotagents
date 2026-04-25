# Cursor — Integration Notes

Source: https://cursor.com/docs/rules, https://cursor.com/docs/skills, https://cursor.com/docs/hooks, https://cursor.com/docs/mcp, https://cursor.com/docs/subagents

Things observed in Cursor that could be integrated into dotagents:

---

## 1. Rules have structured frontmatter — a richer instruction target

Cursor project rules live in `.cursor/rules/*.mdc` with YAML frontmatter fields `description`, `alwaysApply: bool`, and `globs: [...]`. The four rule modes (Always Apply, Apply Intelligently, Apply to Specific Files, Apply Manually) are driven entirely by which frontmatter combination is set.

The current dotagents `InstructionFeature` is a plain content blob with no frontmatter. A `RuleFeature` (or an optional frontmatter layer on `InstructionFeature`) could render `.mdc` files with `description` and `alwaysApply` fields populated from config variables, unlocking per-file rule mode control for Cursor.

---

## 2. `disable-model-invocation` skill field not in `SkillMetadata`

Cursor's SKILL.md frontmatter supports a `disable-model-invocation: bool` field. When `true`, the skill is only included when explicitly invoked via `/skill-name` rather than auto-applied by the agent. The current `SkillMetadata` struct does not model this field. Adding it as `disable_model_invocation: Option<bool>` (serialized as `"disable-model-invocation"`) would let dotagents render Cursor-specific skills that behave like traditional slash commands.

---

## 3. Hooks — a new feature type

Cursor stores hooks in `.cursor/hooks.json` — a JSON file mapping event names (`afterFileEdit`, `beforeShellExecution`, `sessionStart`, etc.) to arrays of `{ "command": "...", "timeout": N, "matcher": "..." }` entries. This is conceptually distinct from instructions or MCP: hooks are event-driven scripts rather than model context. A new `HookFeature` with a target of `.cursor/hooks.json` would let users manage hook configs from dotagents.

---

## 4. Subagents — same pattern as Claude and Codex

Custom Cursor subagents are markdown files with YAML frontmatter (`name`, `description`, `model`, `readonly`) stored in `.cursor/agents/`. This is the same subagent concept seen in Claude (`.claude/agents/`) and Codex (`.codex/agents/`). A shared `SubagentFeature` using a markdown-with-frontmatter format could deploy to all three paths simultaneously.

---

## 5. MCP `envFile` field not in `ServerConfig`

Cursor's stdio MCP server config supports an `envFile` field (path to a `.env` file to load variables from). The existing `ServerConfig::Stdio` variant does not include `env_file`. Adding `env_file: Option<String>` to `Stdio` would allow dotagents to express this field in `mcp.jsonc` and render it for Cursor.
