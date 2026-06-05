## 1. Add dependencies

- [x] 1.1 Add `jsonc-parser` crate via `cargo add jsonc-parser`
- [x] 1.2 Add `toml_edit` crate via `cargo add toml_edit`

## 2. Format detection utility

- [x] 2.1 Create `src/utils/format.rs` with `MergeFormat` enum (Json, Jsonc, Toml, Yaml) and `from_extension(path: &Path) -> Option<MergeFormat>` function
- [x] 2.2 Add unit tests for format detection: `.json` → Json, `.jsonc` → Jsonc, `.toml` → Toml, `.yaml`/`.yml` → Yaml, `.md` → None

## 3. JSON merge implementation

- [x] 3.1 Implement `merge_json(existing: &Value, rendered: &Value) -> Value` in `src/utils/merge_config.rs` — recursive object merge, arrays replaced wholesale, rendered wins on scalar conflicts
- [x] 3.2 Add unit tests: existing keys preserved, rendered keys win, arrays replaced, nested object merge, empty existing

## 4. JSONC merge implementation

- [x] 4.1 Implement `merge_jsonc(existing_content: &str, rendered: &serde_json::Value) -> Result<String>` using `jsonc-parser` CST edits to preserve comments
- [x] 4.2 Add unit tests: comments preserved, keys updated, new keys added, existing keys outside rendered scope untouched

## 5. TOML merge implementation

- [x] 5.1 Implement `merge_toml(existing_content: &str, rendered_content: &str) -> Result<String>` using `toml_edit` to preserve formatting and comments
- [x] 5.2 Add unit tests: sections preserved, keys updated, comments preserved, array-of-tables replaced

## 6. YAML merge implementation

- [x] 6.1 Implement `merge_yaml(existing_content: &str, rendered_content: &str) -> Result<String>` using `serde_yaml` (note: does not preserve comments)
- [x] 6.2 Add unit tests: sections preserved, keys updated, arrays replaced, nested objects merged
- [x] 6.3 Add integration test: deploy MCP to existing YAML file preserves other sections

## 7. Wire merge into renderer.rs

- [x] 7.1 Add Phase 3 in `render_feature_with_settings()` between template rendering and `write_file()`: detect format, read existing file, merge, compute hash on merged output
- [x] 7.2 Add `CacheUpdate::MergeSkipped { path, reason }` variant for parse-error skip cases
- [x] 7.3 Update cache hash computation to use merged content instead of raw rendered content

## 8. Update provider.toml targets

- [x] 8.1 Update `public/v1/templates/qwen/provider.toml` — change MCP target from `.qwen/mcp.json` to `.qwen/settings.json`, remove "manual merge" comment
- [x] 8.2 Update `public/v1/templates/kilocode/provider.toml` — change MCP target from `.kilo/mcp.json` to `.kilo/kilo.jsonc`, remove "manual merge" comment
- [x] 8.3 Update `public/v1/templates/mistral-vibe/provider.toml` — change MCP target from `.vibe/mcp.toml` to `.vibe/config.toml`, remove "manual merge" comment

## 9. Integration tests

- [x] 9.1 Add integration test: deploy MCP to existing JSON file preserves non-MCP keys
- [x] 9.2 Add integration test: deploy MCP to existing JSONC file preserves comments
- [x] 9.3 Add integration test: deploy MCP to existing TOML file preserves other sections
- [x] 9.4 Add integration test: deploy to malformed existing file skips with warning
- [x] 9.5 Add integration test: deploy to non-existent file writes directly (no merge)

## 10. Verification

- [x] 10.1 Run `mise check` (cargo fmt + cargo clippy) and fix any failures
- [x] 10.2 Run `mise tests` (cargo test) and fix any failures
