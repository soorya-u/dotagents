# GitHub Copilot CLI — Integration Notes

Sources:
- https://docs.github.com/en/copilot/concepts/agents/copilot-cli/comparing-cli-features
- https://docs.github.com/en/copilot/concepts/agents/about-agent-skills
- https://docs.github.com/en/copilot/concepts/context/mcp
- https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks
- https://docs.github.com/en/copilot/reference/custom-agents-configuration
- https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-mcp-servers
- https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-custom-instructions
- https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/create-custom-agents-for-cli

Things observed in GitHub Copilot CLI that could be integrated into dotagents:

---

## 1. MCP uses `"type": "local"` instead of `"type": "stdio"`

Copilot CLI uses `"type": "local"` for stdio-based MCP servers, while most other providers use `"type": "stdio"` (or omit the type). The docs explicitly note that `"stdio"` from Claude Code and VS Code is mapped to `"local"` for compatibility. The current `mcp.hbs` template outputs `"type": "local"` in the else branch (non-http), which is correct. If dotagents ever adds a per-provider type override mechanism, this could be expressed more cleanly than hardcoding the string in a template.

---

## 2. MCP servers have a `"tools"` field for per-server tool filtering

Each MCP server entry in `~/.copilot/mcp-config.json` includes a `"tools"` array that restricts which tools from that server are made available to Copilot. The current `ServerConfig` does not model this field, so the template hardcodes `["*"]` (all tools). Adding an optional `tools` array to `ServerConfig::Http` and `ServerConfig::Stdio` would let users filter per-server tools in `mcp.jsonc` and have them rendered correctly for Copilot (and similar providers like Factory Droid).

---

## 3. Commands target `.github/prompts/` with `.prompt.md` extension

Copilot CLI prompt files live in `.github/prompts/<name>.prompt.md`, not in a `commands/` subdirectory and not using a plain `.md` extension. The `command.hbs` template is a pass-through (`{{command.content}}`), so the content is identical to other providers — only the target path and extension differ. This is already handled correctly by the `provider.toml` target string.

---

## 4. Custom agents use `.agent.md` files with a distinct frontmatter schema

Copilot custom agents live in `.github/agents/<name>.agent.md` or `~/.copilot/agents/<name>.agent.md`. Their YAML frontmatter differs from the Agent Skills format: `name`, `description`, `target` (vscode or github-copilot), `tools` (list), `model`, `disable-model-invocation` (bool), `user-invocable` (bool), `mcp-servers` (inline YAML object), `metadata` (object). The `mcp-servers` block inside an agent profile is a YAML representation of the `mcpServers` JSON and uses `type: 'local'` for stdio. If a future `AgentFeature` is added to dotagents, Copilot agent profiles share the `disable-model-invocation` and `user-invocable` fields with Factory Droid, suggesting these belong in a common `AgentMetadata` struct.

---

## 5. Instructions have path-specific variants with `applyTo` frontmatter

Beyond the repo-wide `.github/copilot-instructions.md` (current template target), Copilot supports path-specific instruction files at `.github/instructions/*.instructions.md`, each requiring an `applyTo` frontmatter field (glob pattern). The current `InstructionFeature` is a single content blob with no mechanism to express path scoping or the `applyTo` / `excludeAgent` frontmatter fields these files require. Supporting path-specific instructions would need either a new feature variant or additional frontmatter fields on `InstructionFeature`.

---

## 6. Hooks use dual `bash`/`powershell` fields and live in `.github/hooks/hooks.json`

Copilot hooks are stored in `.github/hooks/hooks.json` with a `"version": 1` wrapper. Each hook entry has `type: "command"`, `bash` (Unix shell command), `powershell` (Windows command), `cwd`, `timeoutSec`, and `env`. The dual-script approach (`bash` + `powershell`) is unique to Copilot among providers reviewed — it enables cross-platform hook definitions in a single file. If a future `HookFeature` is added, Copilot's format would need both shell fields to be modelled, along with the `version` envelope.

---

## 7. Skills are loaded from multiple paths including `.agents/skills/`

Copilot CLI loads skills from `.github/skills/` (project), `~/.copilot/skills/` (personal), `.claude/skills/`, and `.agents/skills/`. The template targets `.github/skills/<name>/SKILL.md` as the primary project-level location. If a user also wants skills deployed to the cross-provider `.agents/skills/` path, they can add a second provider entry or configure a custom target. No code changes are needed, but a note in user docs would be helpful since the cross-provider standard path is `.agents/skills/` while the Copilot-native path is `.github/skills/`.
