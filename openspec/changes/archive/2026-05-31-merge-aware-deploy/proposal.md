## Why

Several providers (Gemini, OpenCode, Qwen, KiloCode, Mistral Vibe) store MCP config inside a shared file (`settings.json`, `opencode.json`, `kilo.jsonc`, `config.toml`) alongside user-managed keys (API keys, model settings, permissions). Current deploy is pure `fs::write` — it either clobbers the entire file (Gemini, OpenCode) or outputs to a sidecar file requiring manual copy-paste (Qwen, KiloCode, Mistral Vibe). Both strategies are broken or user-hostile.

## What Changes

- **Format-aware read-modify-write merge**: After template rendering, if the target file already exists, parse both the existing file and the rendered output, deep-merge them (rendered wins on key conflicts, arrays replaced wholesale), and write back the merged result. Supports JSON, JSONC (comment-preserving via `jsonc-parser`), TOML (format-preserving via `toml_edit`), and YAML.
- **Provider target path updates**: Qwen, KiloCode, and Mistral Vibe targets change from sidecar files (`.qwen/mcp.json`, `.kilo/mcp.json`, `.vibe/mcp.toml`) to the actual shared config files (`.qwen/settings.json`, `.kilo/kilo.jsonc`, `.vibe/config.toml`).
- **Error handling**: If the existing shared file has a syntax error, skip that provider's deploy with a warning — do not touch the file.
- **Cache hashes merged output**: The deploy cache stores the hash of the merged output (what actually gets written), not just the rendered template.

## Capabilities

### New Capabilities
- `deploy-merge-write`: Format-aware read-modify-write merge logic in the deploy pipeline. Reads existing target file, parses by format (JSON/JSONC/TOML/YAML), deep-merges rendered output on top (new wins on conflict, arrays replaced), writes back.

### Modified Capabilities
- `deploy-pipeline`: The write step changes from pure overwrite to merge-aware when the target file exists and is a structured config format.
- `mcp-provider-template-rendering`: Provider target paths for Qwen, KiloCode, and Mistral Vibe change from sidecar files to shared config files.

## Impact

- **New dependencies**: `jsonc-parser` (JSONC comment-preserving edits), `toml_edit` (format-preserving TOML edits). YAML crate TBD.
- **Renderer** (`src/templates/renderer.rs`): New Phase 3 between template rendering and `write_file()`.
- **Provider templates** (`public/v1/templates/{qwen,kilocode,mistral-vibe}/provider.toml`): Target path updates, removal of "manual merge" comments.
- **Utils** (`src/utils/`): New merge module(s) for format-aware deep merge.
- **Cache**: Hash computation moves to after merge.
- **No breaking changes**: Providers that write to dedicated files (Claude, Cursor, Codex, Factory Droid, Kimi) are unaffected.
