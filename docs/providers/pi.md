# Pi — Integration Notes

Sources:
- Inline documentation provided by user (skills, settings, prompt-templates, packages pages)

Things observed in Pi that could be integrated into dotagents:

---

## 1. Commands are called "Prompt Templates" — deployed to `.pi/prompts/`, not `.pi/commands/`

Pi does not have a "commands" concept. The equivalent feature is *prompt templates*: Markdown files at `.pi/prompts/<name>.md` (project) or `~/.pi/agent/prompts/<name>.md` (global). The frontmatter supports `description` (optional, shown in autocomplete) and `argument-hint` (optional, displayed as parameter hint in the dropdown). Command name is derived from the filename; there is no `name` field. `CommandFeature.description` maps cleanly to the `description` frontmatter field. `argument-hint` is Pi-specific and has no counterpart in `CommandFeature` — users who need it must add it manually after deployment. The `command.hbs` template deploys to `.pi/prompts/{{command.name}}.md`.

---

## 2. Argument injection syntax is `$@`/`$1`/`$2`, not `{{args}}`

Pi prompt templates use POSIX-style argument variables: `$1`, `$2`, `$@` (all args joined), `$ARGUMENTS` (alias for `$@`), `${@:N}` (args from Nth position), and `${@:N:L}` (L args starting at N). The `{{args}}` Handlebars variable that some other providers use is not supported. Since dotagents renders `command.content` through the Handlebars engine before writing it, any `{{args}}` in the source would be expanded to an empty string (no such variable in the namespace). Users writing commands for Pi should use `$@` syntax directly in the source `.dotagents/commands/` files. This is also compatible with Claude Code's slash command format, which does support `$ARGUMENTS`.

---

## 3. Skills implement the full Agent Skills spec — including `allowed-tools`, `disable-model-invocation`

Pi loads skills from `.pi/skills/<name>/SKILL.md` (project) or `~/.pi/agent/skills/<name>/SKILL.md` (global). It also scans `.agents/skills/` in the project directory and its ancestors (up to the git repo root) — a cross-harness discovery path shared with Claude Code's `~/.claude/skills`. Pi validates skills against the Agent Skills specification and warns on violations. Supported frontmatter: `name`, `description`, `license`, `compatibility`, `allowed-tools` (space-delimited, experimental), `metadata` (arbitrary key-value map), and `disable-model-invocation` (boolean). The `skill.hbs` template emits `name`, `description`, and the three fields available from `SkillMetadata` (`license`, `compatibility`, `allowed-tools`). The Pi-specific `metadata` and `disable-model-invocation` fields are not in `SkillMetadata` and cannot be emitted; users who need them must add them manually or use `/settings`.

---

## 4. MCP not covered by provided docs — McpFeature not deployable

The provided documentation (skills, settings, prompt-templates, packages pages) does not describe an `mcpServers` configuration key or any MCP server setup. Pi likely supports MCP through extensions or a separate config page not included in the provided sources. Until the MCP configuration format is confirmed, `McpFeature` is not included in `provider.toml`. If Pi uses the standard `mcpServers` JSON format (as used by Claude Code, Kimi, Junie, Qoder CLI), an `mcp.hbs` can be added trivially once the target file path is known.

---

## 5. No instruction file documented — InstructionFeature not deployable

The settings, skills, and prompt-template docs do not mention a project-level instruction or memory file (no AGENTS.md, no `.pi/INSTRUCTIONS.md`, no equivalent). Pi does not appear to load a static instruction blob from a well-known path the way Claude Code (CLAUDE.md), Gemini (GEMINI.md), or Qwen Code (AGENTS.md) do. If a future release documents an instruction file path, an `instruction.hbs` pass-through can be added.

---

## 6. Skill discovery spans multiple paths including `.agents/skills/` — cross-harness sharing

Pi discovers skills from six locations with varying rules: `~/.pi/agent/skills/` and `.pi/skills/` (direct root `.md` files as individual skills, plus recursive `SKILL.md` directories); `~/.agents/skills/` and `.agents/skills/` (only `SKILL.md` directories, root `.md` files ignored); packages (`skills/` dirs); and `settings.json` `skills` array. The cross-harness `.agents/skills/` path means skills deployed to `.agents/skills/<name>/SKILL.md` (e.g. the cross-provider Gemini CLI path) are picked up by Pi automatically. A future cross-provider `AgentSkillsFeature` could deploy once to `.agents/skills/` and cover Pi, Gemini, and other spec-compliant providers simultaneously. The provider targets `.pi/skills/` specifically for now.

---

## 7. Packages extend Pi with skills, extensions, prompts, and themes — not deployable via dotagents

Pi has a rich package system (`pi install npm:...`, `pi install git:...`) that bundles skills, extensions (TypeScript/JS), prompt templates, and themes. Packages are declared in `~/.pi/agent/settings.json` or `.pi/settings.json` under the `packages` key. These are installation-time concerns, not file-deploy concerns, and are outside the scope of `dotagents deploy`. Extensions (arbitrary TypeScript/JS that runs inside Pi) are also not deployable. The `settings.json` `skills` array can point to directories of additional skills; this is a user-level concern that dotagents does not manage.
