## Context

The deploy pipeline renders feature content through Handlebars templates and writes the result to a target path via `write_file()` (pure `fs::write`). For providers whose MCP config lives inside a shared file (Gemini `settings.json`, OpenCode `opencode.json`, etc.), this overwrites the entire file, destroying user-managed keys.

The `.gitignore` fence system (`src/utils/gitignore.rs`) is the only existing read-modify-write precedent — it uses fenced regions to preserve user content. Structured config files need a different approach: format-aware parsing and key-level merging.

## Goals / Non-Goals

**Goals:**
- Read-modify-write merge for JSON, JSONC, TOML, and YAML target files
- Rendered output wins on key conflicts; arrays replaced wholesale; unknown existing keys preserved
- Comment/formatting preservation for JSONC and TOML
- Update provider targets from sidecar files to shared config files where applicable
- Graceful degradation: malformed existing files → skip with warning, no write

**Non-Goals:**
- Merging into global/home-directory config files (Goose `~/.config/goose/config.yaml`, Kimi `~/.kimi-code/mcp.json`)
- Deep merge for arrays (element-wise, by-identifier) — arrays are always replaced wholesale
- `merge_keys` or per-provider merge configuration — the merge is automatic and format-driven
- Hooks deployment (future feature, but the merge infrastructure will support it)

## Decisions

### 1. Merge happens in renderer.rs, between Phase 2 and write_file()

```
render_feature_with_settings()
  ├── Phase 1: render target path (existing)
  ├── Phase 2: render template → rendered_content (existing)
  ├── Phase 3 (NEW): merge-aware write
  │     if target exists AND format is mergeable:
  │       parse existing file → existing_doc
  │       parse rendered_content → rendered_doc
  │       merge(existing_doc, rendered_doc) → merged_doc  (rendered wins)
  │       serialize merged_doc → final_content
  │     else:
  │       final_content = rendered_content
  └── write_file(target, final_content)
```

**Rationale:** Keeps the merge logic at the single point where content meets disk. No changes needed to the template rendering or feature loading pipelines.

### 2. Format detection by file extension

| Extension | Format | Parser crate | Writer | Comment preservation |
|---|---|---|---|---|
| `.json` | JSON | `serde_json` (existing) | `serde_json::to_string_pretty` | N/A |
| `.jsonc` | JSONC | `jsonc-parser` (new) | `jsonc-parser` AST edits | Yes |
| `.toml` | TOML | `toml_edit` (new) | `toml_edit::Document` | Yes |
| `.yaml` / `.yml` | YAML | TBD | TBD | Partial |

**Rationale:** Extension-based detection is simple and matches how provider templates declare targets. No provider-specific configuration needed.

### 3. Merge semantics: recursive object merge, array replace

```
merge(existing, rendered):
  for each key in rendered:
    if both values are objects → recurse
    else → rendered value wins (including arrays)
  keys only in existing → preserved as-is
```

**Rationale:** Matches the user's stated approach: "combine, if any conflicts prioritize newly generated." Arrays replaced wholesale avoids the removal problem and keeps semantics simple. Users manage all MCP through `.dotagents/mcp.jsonc`.

### 4. JSONC via jsonc-parser AST edits

`jsonc-parser` provides a CST/AST that preserves comments and formatting. We modify only the keys present in the rendered output, leaving comments and other keys untouched.

**Rationale:** KiloCode uses `.kilo/kilo.jsonc`. Naive JSON parse → serialize would strip comments. AST-level edits preserve the user's file structure.

### 5. TOML via toml_edit

`toml_edit` preserves formatting, comments, and key ordering. We merge at the document level, replacing only the keys present in rendered output.

**Rationale:** Mistral Vibe uses `.vibe/config.toml`. `toml_edit` is the standard format-preserving TOML crate in Rust.

### 6. Error handling: skip and warn

If the existing file cannot be parsed:
- Log a warning with the file path and parse error
- Skip writing for this provider (do NOT fall back to overwrite or sidecar)
- Return `CacheUpdate::Skipped` (or a new variant `CacheUpdate::MergeError`)

**Rationale:** Overwriting a malformed file risks data loss. Skipping is safe — the user can fix their file and re-run deploy.

### 7. Cache hashes the merged output

The SHA-256 hash stored in `CacheEntry` is computed on the final merged content (what actually gets written to disk), not the raw template render.

**Rationale:** This ensures:
- If user adds keys to the shared file, next deploy detects the disk hash mismatch and re-merges
- If only dotagents content changed, the merged hash changes → re-deploy
- If nothing changed (same merge result), skip as before

## Risks / Trade-offs

- **jsonc-parser maturity** → Mitigation: well-maintained crate, used by VS Code's JSON language service. Test extensively with comment-heavy JSONC files.
- **YAML complexity** → Mitigation: YAML is the hardest format. Can ship JSON/JSONC/TOML first and add YAML later. No current workspace-level YAML targets need merge (Goose is global-only).
- **Parallel deploy + merge** → Mitigation: `rayon::par_iter` is already used. Merge is pure computation on file content, no shared mutable state. Each provider writes to a different target path (dedup already ensures this).
- **Stale keys from removed servers** → Not a risk with current semantics: dotagents renders the full `mcpServers` object as one key. The merge replaces the entire `mcpServers` key (rendered wins), so removed servers disappear. User-added keys *outside* `mcpServers` are preserved, which is correct.
