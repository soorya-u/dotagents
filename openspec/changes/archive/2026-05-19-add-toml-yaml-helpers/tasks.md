## 1. Add TOML and YAML Handlebars helpers to the binary

- [x] 1.1 Add `TomlHelper` struct in `src/templates/helpers.rs` — serializes JSON objects to TOML table-style key-value lines using `toml::to_string`, errors on non-object values
- [x] 1.2 Add `TomlInlineHelper` struct in `src/templates/helpers.rs` — serializes JSON objects to TOML inline table `{ KEY = "val" }` syntax, errors on non-object values
- [x] 1.3 Add `YamlHelper` struct in `src/templates/helpers.rs` — serializes any JSON value to YAML block syntax using `serde_yaml::to_string`
- [x] 1.4 Add helper name constants (`TOML_HELPER`, `TOML_INLINE_HELPER`, `YAML_HELPER`) in `src/constants/helpers.rs`
- [x] 1.5 Register all three helpers in `Templater::new` in `src/templates/templater.rs`
- [x] 1.6 Add unit tests for all three helpers in `src/templates/helpers.rs` (object input, non-object error cases, array/string/null for yaml)

## 2. Update provider templates to use new helpers

- [x] 2.1 Update `public/v1/templates/mistral-vibe/mcp.hbs` — replace manual `{{#each this.env}}` inline table construction with `{{toml-inline this.env}}`
- [x] 2.2 Update `public/v1/templates/codex/mcp.hbs` — replace manual `{{#each this.env}}` section content with `{{toml this.env}}`

## 3. Verify and test

- [x] 3.1 Run `mise check` (cargo fmt + cargo clippy) — must exit 0
- [x] 3.2 Run `mise tests` (unit + integration + e2e) — must exit 0
