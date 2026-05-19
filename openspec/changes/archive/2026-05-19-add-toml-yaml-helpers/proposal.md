## Why

No Handlebars helpers exist for rendering TOML or YAML syntax. The existing `{{json}}` helper works for JSON-based providers (Claude, Gemini, Copilot, Cursor), but TOML-based providers (Codex, Gemini, Mistral Vibe) need inline-table and section-style rendering for MCP config — especially `env` maps. Currently, templates use manual `{{#each}}` loops to build TOML inline tables, which is fragile and hard to maintain.

## What Changes

- Add `{{toml value}}` Handlebars helper — renders a JSON value as TOML table-style key-value lines (for use inside `[table.section]` blocks)
- Add `{{toml-inline value}}` Handlebars helper — renders a JSON value as a TOML inline table `{ KEY = "val" }` (for use inside existing key-value assignments)
- Add `{{yaml value}}` Handlebars helper — renders a JSON value as YAML block syntax
- Update `mistral-vibe/mcp.hbs` to use `{{toml-inline this.env}}` instead of the manual `{{#each}}` loop
- Update `codex/mcp.hbs` to use `{{toml this.env}}` instead of the manual `{{#each}}` loop
- Register all three helpers in `Templater::new`

## Capabilities

### New Capabilities
- `template-helpers`: New Handlebars helpers (`{{toml}}`, `{{toml-inline}}`, `{{yaml}}`) available in all template rendering contexts

### Modified Capabilities
- None

## Impact

- `src/templates/helpers.rs` — new helper structs (`TomlHelper`, `TomlInlineHelper`, `YamlHelper`)
- `src/templates/templater.rs` — register three new helpers
- `src/constants/helpers.rs` — add helper name constants
- `public/v1/templates/mistral-vibe/mcp.hbs` — simplify env rendering
- `public/v1/templates/codex/mcp.hbs` — simplify env rendering
- `Cargo.toml` — `toml` and `serde_yaml` already present, no new deps needed
