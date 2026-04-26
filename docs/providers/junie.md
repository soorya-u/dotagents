# Junie (JetBrains) — Integration Notes

Sources:
- https://junie.jetbrains.com/docs/junie-cli-configuration.html#add-extra-configuration-files
- https://junie.jetbrains.com/docs/action-allowlist-junie-cli.html
- https://junie.jetbrains.com/docs/agent-skills.html
- https://junie.jetbrains.com/docs/junie-cli-mcp-configuration.html#add-an-mcp-server-from-json-configuration
- https://junie.jetbrains.com/docs/junie-cli-subagents.html
- https://junie.jetbrains.com/docs/custom-slash-commands.html

Things observed in Junie CLI that could be integrated into dotagents:

---

## 1. Instructions file path is not auto-discovered — requires explicit `guidelines-location` in config.json

Junie has no default file name for project guidelines. The path is configured via `"guidelines-location"` in `.junie/config.json`, resolved relative to the config file's directory. The `instruction.hbs` template writes to `{{dir.workspace}}/.junie/guidelines.md`, which maps to the config value `"./guidelines.md"` (since config.json lives in `.junie/`). Users must add this entry to their `.junie/config.json` manually after deploying:

```json
{ "guidelines-location": "./guidelines.md" }
```

Unlike every other provider reviewed that auto-loads a well-known filename (e.g. `AGENTS.md`, `.goosehints`, `GEMINI.md`), Junie's guidelines require this explicit wiring step. A future `PostDeployHook` feature in dotagents could automate patching JSON/TOML config files, which would make this seamless.

---

## 2. Skills use Junie-native `.junie/skills/` path, not the cross-provider `.agents/skills/`

Junie's primary skills directory is `.junie/skills/<name>/SKILL.md`, not the cross-provider `.agents/skills/` standard. The SKILL.md format is identical — YAML frontmatter with required `name` and `description`, followed by Markdown content — and Junie explicitly advertises Agent Skills compatibility. The template targets `.junie/skills/` since that's the path Junie auto-loads. Users who also want skills at `.agents/skills/` (for other agents that load from there) can add a second provider entry. Junie also detects skills from `.cursor/skills/`, `.claude/skills/`, and `.codex/skills/` and suggests importing them, so dotagents skills deployed to any of those paths would also be auto-discovered.

---

## 3. Commands are plain Markdown with only a `description` frontmatter field

Junie slash commands use a simpler frontmatter schema than most other providers: only `description` is defined in frontmatter; `name` is inferred from the filename (no `name:` key in the YAML). The `command.hbs` template outputs only `description: {{command.description}}` in the frontmatter block and `{{{command.content}}}` as the body. Since `CommandFeature` always populates `command.name` from frontmatter and uses it in the target path (`{{command.name}}.md`), this works correctly — the file name carries the command name and the frontmatter carries the description.

Junie commands also support `$argumentName` placeholders in the body for runtime argument injection. These `$arg` tokens pass through dotagents' Handlebars rendering unchanged (Handlebars only processes `{{ }}` delimiters), so users can safely include `$argumentName` in their command bodies.

---

## 4. MCP format is the cleanest reviewed — no `type` field, discrimination by `command` vs `url`

Junie's `mcp.json` uses pure structural discrimination: stdio servers have `command`/`args`/`env`; HTTP servers have `url`/`headers`. No `"type"` field is present for either variant. This is cleaner than Claude (which outputs `"type": "http"`), Factory Droid (requires `"type": "stdio"` explicitly), and Copilot (uses `"type": "local"`). The `mcp.hbs` template omits the `type` field for both branches. The target file `.junie/mcp/mcp.json` is a dedicated MCP config file scoped to the project, making it safe to overwrite on redeploy without risk to any other configuration.

---

## 5. Subagent frontmatter has fields with no `SkillFeature` analog

Junie subagents (`.junie/agents/<name>.md`) use a richer frontmatter than Agent Skills: `tools` (allowlist of tool groups like `["Read", "Grep", "Edit"]`), `disallowedTools` (denylist), `model` (per-subagent model override), `skills` (list of skill IDs to inject), and `allowPromptArgument` (boolean). The current `SkillFeature` / `SkillMetadata` struct does not model these fields. A future `SubagentFeature` or `AgentFeature` would need to capture at minimum `tools`, `disallowedTools`, `model`, and `skills` to generate valid Junie subagent files. Junie also auto-imports subagent files from `.cursor/agents/`, `.claude/agents/`, and `.codex/agents/`, so a cross-provider agent format is feasible.

---

## 6. Action Allowlist (`~/.junie/allowlist.json`) is a personal permissions file — not suitable for deploy

The allowlist defines per-user rules for which file edits, shell commands, and MCP tools Junie can run autonomously. It contains four action categories (`fileEditing`, `executables`, `mcpTools`, `readOutsideProject`), each with an array of `prefix`/`pattern` + `action` rules. This file is intentionally personal and machine-specific (it lives in `~/.junie/`), so deploying it via dotagents would be inappropriate. No template is created for it.

---

## 7. `config.json` `skill-locations` / `command-locations` fields enable custom discovery paths

Junie's `config.json` supports `skill-locations`, `command-locations`, `agent-locations`, and `mcp-locations` as arrays of extra search paths. This means users could configure Junie to load skills from `.agents/skills/` (the cross-provider standard) alongside `.junie/skills/` by adding `"skill-locations": ["./.agents/skills"]` to their `config.json`. This is purely a user-configuration concern, not a template one, but worth noting because it makes Junie compatible with the cross-provider standard without requiring file duplication.
