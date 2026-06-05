## Purpose

Defines how deploy performs read-modify-write merging when writing to existing structured config files (JSON, JSONC, TOML, YAML).

## Requirements

### Requirement: Deploy merges rendered output into existing structured config files
The system SHALL perform read-modify-write merge when deploying to a target file that already exists and has a structured config format (JSON, JSONC, TOML, YAML). The rendered output SHALL be deep-merged on top of the existing file content, with rendered values winning on key conflicts.

#### Scenario: JSON merge preserves existing keys
- **WHEN** the target file is `.gemini/settings.json` containing `{"model": "gemini-2.5", "mcpServers": {"old": {}}}` and the rendered output is `{"mcpServers": {"new": {}}}`
- **THEN** the merged result SHALL be `{"model": "gemini-2.5", "mcpServers": {"new": {}}}` — `model` preserved, `mcpServers` replaced

#### Scenario: JSONC merge preserves comments
- **WHEN** the target file is `.kilo/kilo.jsonc` containing comments and the rendered output modifies the `mcp` key
- **THEN** the merged result SHALL preserve all comments and formatting outside the modified keys

#### Scenario: TOML merge preserves formatting
- **WHEN** the target file is `.vibe/config.toml` containing `[model]\nname = "mistral"` and `[[mcp_servers]]` entries, and the rendered output provides new `[[mcp_servers]]` entries
- **THEN** the merged result SHALL preserve the `[model]` section and replace `[[mcp_servers]]` entries with the rendered output

#### Scenario: Arrays are replaced wholesale
- **WHEN** the existing file has a key with an array value `[A, B, C]` and the rendered output has the same key with `[D, E]`
- **THEN** the merged result SHALL have `[D, E]` for that key (no element-wise merge)

### Requirement: Deploy skips merge when target file does not exist
The system SHALL write the rendered output directly (no merge) when the target file does not exist on disk.

#### Scenario: New file written without merge
- **WHEN** the target file does not exist
- **THEN** the system SHALL create the file with the rendered content as-is

### Requirement: Deploy skips merge for non-structured formats
The system SHALL write the rendered output directly when the target file has a non-structured format (e.g., `.md`, `.txt`, `.ignore`).

#### Scenario: Markdown files written without merge
- **WHEN** the target file is `AGENTS.md` or a command `.md` file
- **THEN** the system SHALL write the rendered content as-is (pure overwrite)

### Requirement: Deploy skips write on parse error in existing file
The system SHALL NOT write to the target file if the existing file cannot be parsed. A warning SHALL be logged with the file path and parse error.

#### Scenario: Malformed JSON skips write
- **WHEN** the existing `.gemini/settings.json` contains invalid JSON
- **THEN** the system SHALL log a warning and skip writing for that provider

#### Scenario: Malformed TOML skips write
- **WHEN** the existing `.vibe/config.toml` contains invalid TOML
- **THEN** the system SHALL log a warning and skip writing for that provider

### Requirement: Cache stores hash of merged output
The system SHALL compute the deploy cache hash on the final merged content (what actually gets written to disk), not on the raw template-rendered content.

#### Scenario: Hash reflects merged content
- **WHEN** deploy merges rendered output into an existing file
- **THEN** the cache entry hash SHALL be the SHA-256 of the merged content

#### Scenario: Unchanged merge result skips rewrite
- **WHEN** deploy runs again with identical source and the existing file has not changed
- **THEN** the merged content hash SHALL match the cache and the write SHALL be skipped
