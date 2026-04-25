# Goose — Integration Notes

Sources:
- https://goose-docs.ai/docs/guides/context-engineering/using-goosehints
- https://goose-docs.ai/docs/guides/context-engineering/using-skills
- https://goose-docs.ai/docs/guides/context-engineering/slash-commands
- https://goose-docs.ai/docs/guides/using-persistent-instructions
- https://goose-docs.ai/docs/guides/recipes/recipe-reference
- https://goose-docs.ai/docs/guides/subagents
- https://goose-docs.ai/docs/guides/using-gooseignore
- https://goose-docs.ai/docs/guides/config-files
- https://goose-docs.ai/docs/guides/acp-clients

Things observed in Goose that could be integrated into dotagents:

---

## 1. Two native instruction file names: `.goosehints` and `AGENTS.md`

Goose loads both `AGENTS.md` and `.goosehints` by default (configured via `CONTEXT_FILE_NAMES=["AGENTS.md", ".goosehints"]`). `AGENTS.md` is the cross-provider standard and is the primary target. `.goosehints` is the goose-native format and supports `@filename` syntax for automatically including referenced file content into context. The two serve subtly different purposes: `.goosehints` is the idiomatic goose file; `AGENTS.md` is the cross-provider compatibility path. The current template targets `AGENTS.md`. Users who want goose-native `.goosehints` behavior can add a custom provider entry with `target = "{{dir.workspace}}/.goosehints"`.

---

## 2. Commands map to recipe YAML files — but only a minimal subset is expressible

Goose slash commands are shortcuts that reference recipe YAML files, not standalone prompt files. The recipe format (`version`, `title`, `description`, `instructions`, `prompt`, `parameters`, `extensions`, `settings`, `sub_recipes`, `response`, `retry`) is far richer than dotagents' `CommandFeature`. The `command.hbs` template generates a minimal valid recipe using `title` (from `command.name`), `description` (from `command.description`), and `instructions` (from `command.content`). The rich recipe features — typed parameters with Jinja `{{ variable }}` substitution, per-recipe extension lists, retry logic, structured JSON response schemas, and subrecipes — cannot be expressed through `CommandFeature`. Users who need those features must edit the generated `.goose/recipes/<name>.yaml` files by hand after deploy.

Additionally, goose recipe templates use Jinja-style `{{ variable }}` syntax, which conflicts with dotagents' own Handlebars `{{ variable }}` rendering pass. Any `{{ var.foo }}` references in a command body will be resolved by dotagents before writing to the recipe file, preventing Jinja pass-through to goose.

---

## 3. MCP/extensions config is embedded in `~/.config/goose/config.yaml` with different field names

Goose calls MCP servers "extensions" and stores them in `~/.config/goose/config.yaml` (a YAML file, not a standalone JSON config). The field names differ from the MCP standard: `cmd` (not `command`) for the executable, `envs` (not `env`) for environment key-value pairs, `env_keys` for prompting the user for missing secrets at runtime, and `streamable_http` + `uri` (not `http` + `url`) for HTTP endpoints. Because targeting `~/.config/goose/config.yaml` directly would overwrite the user's model/provider settings and all other configuration, the `mcp.hbs` template generates a standalone `extensions:` YAML snippet to `.goose/extensions.yaml`. This file is not automatically read by goose — users must manually copy the `extensions:` block into their `~/.config/goose/config.yaml`. A merge-aware deploy mode (read-modify-write) would resolve this across all providers that embed feature config in a shared file.

---

## 4. HTTP extension headers are not supported

Goose's `streamable_http` extension type documents only a `uri` field. There is no documented `headers` field for HTTP extensions in `config.yaml` or in recipe extension entries. The `ServerConfig::Http { headers }` field in dotagents cannot be represented for goose — headers are silently dropped in the template. Users needing authentication for remote MCP servers must configure it outside dotagents (e.g. via `env_keys` secrets prompting, which is also not expressible through `ServerConfig`).

---

## 5. Three extension types have no `ServerConfig` analog: `builtin`, `inline_python`, `platform`

Beyond `stdio` (mapped) and `streamable_http` (mapped as `http`), goose supports `builtin` (built-in goose extension by name), `inline_python` (Python code embedded directly in the recipe with `code:` and `dependencies:` fields), and `platform`/`frontend` extension types. None of these can be expressed with the current `ServerConfig::Http` / `ServerConfig::Stdio` enum. Adding new `ServerConfig` variants (e.g., `Builtin { name }`, `InlinePython { code, dependencies }`) would allow recipe extension fields to be fully specified in `mcp.jsonc` for goose-specific use cases.

---

## 6. Skills are fully compatible — standard Agent Skills format with `.agents/skills/` path

Goose skills use the exact Agent Skills open standard: YAML frontmatter with `name` and `description`, followed by Markdown content, stored at `.agents/skills/<name>/SKILL.md`. Goose explicitly documents compatibility with Claude Desktop and other Agent Skills consumers. The `skill.hbs` template and the `.agents/skills/` target path work without any goose-specific adjustments. Goose also discovers skills from the legacy `.goose/skills/` path for backward compatibility — users migrating from an older goose setup may want to add a second provider entry targeting `.goose/skills/{{skill.name}}/SKILL.md`.

---

## 7. Persistent instructions (`GOOSE_MOIM_MESSAGE_FILE`) are a distinct, re-injected context mechanism

Beyond `.goosehints` and `AGENTS.md`, goose has a "persistent instructions" system (MOIM — Model-Observed Internal Memory) that re-injects a file's content into the model context on every turn, not just at session start. This is configured via the `GOOSE_MOIM_MESSAGE_FILE` environment variable. It is not a file that dotagents can usefully deploy (it's an env var path, not the file itself), but a user could set `GOOSE_MOIM_MESSAGE_FILE` to point to a file managed by a custom dotagents provider entry. This is an advanced use case not covered by the current templates.
