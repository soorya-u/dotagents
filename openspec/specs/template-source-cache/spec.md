### Requirement: Downloaded template files are cached at the user-level config directory
`dotagents deploy` SHALL cache downloaded `provider.toml` and `.hbs` template files at `dirs::config_dir()/dotagents/cache/templates/<provider>/<filename>`. This cache is shared across all projects on the machine.

#### Scenario: Template file cached after first download
- **WHEN** `command.hbs` for the `claude` provider is downloaded for the first time
- **THEN** the file is written to `dirs::config_dir()/dotagents/cache/templates/claude/command.hbs`

#### Scenario: Cached file used on subsequent deploy
- **WHEN** `command.hbs` for the `claude` provider is already cached and its checksum matches the registry entry
- **THEN** the cached file is read from disk; no HTTP request is made for that file

### Requirement: Cache validity is determined by SHA-256 checksum from registry.json
For each file to be resolved, `dotagents deploy` SHALL compare the SHA-256 checksum from `registry.json` against the content of the locally cached file. A match means the cache is valid; a mismatch or a missing cache file means the file must be re-downloaded.

#### Scenario: Checksum match — cache hit, no download
- **WHEN** the SHA-256 of the locally cached `mcp.hbs` matches the checksum in `registry.json`
- **THEN** the cached file is used directly; no HTTP request is made

#### Scenario: Checksum mismatch — cache miss, file re-downloaded
- **WHEN** the SHA-256 of the locally cached `instruction.hbs` does not match the checksum in `registry.json`
- **THEN** the file is re-downloaded, the cache is overwritten with the new content, and the new content is used for rendering

#### Scenario: Cached file absent — treated as cache miss
- **WHEN** no cached file exists for `provider.toml` of the `cursor` provider
- **THEN** the file is downloaded from the URL in `registry.json` and stored in the cache

### Requirement: Registry.json includes per-file SHA-256 checksums for each provider
`registry.json` SHALL include a `checksums` map for each provider entry, keyed by filename (e.g., `"command.hbs"`, `"provider.toml"`), with values being the hex-encoded SHA-256 digest of the file contents. The `checksums` field is optional for backward compatibility with clients that do not use it.

#### Scenario: Registry entry includes checksums for all present files
- **WHEN** `generate_registry.sh` runs and `claude/` contains `provider.toml`, `command.hbs`, `instruction.hbs`, `mcp.hbs`, and `skill.hbs`
- **THEN** the `claude` entry in `registry.json` has a `checksums` object with all five filenames as keys and their 64-character hex SHA-256 digests as values

#### Scenario: Files absent from a provider directory are not included in checksums
- **WHEN** a provider directory does not contain `skill.hbs`
- **THEN** `"skill.hbs"` does not appear in that provider's `checksums` map

#### Scenario: Old client ignores checksums field
- **WHEN** a client that predates this change reads a `registry.json` with `checksums` entries
- **THEN** the client ignores the `checksums` field and behaves as before (using explicit `template`/`target` from config)

### Requirement: `--no-cache` bypasses both the rendered-output cache and the template-source cache
When `dotagents deploy --no-cache` is specified, `dotagents deploy` SHALL skip reading from the template-source cache and re-download all required template files from the remote. The template-source cache SHALL be updated with the freshly downloaded files.

#### Scenario: --no-cache forces re-download of all template files
- **WHEN** `dotagents deploy --no-cache` is run and all provider templates are cached locally
- **THEN** all template files are re-downloaded regardless of checksum match; the cache is updated with the new content

### Requirement: Partial or corrupt cached files are treated as cache misses
If a cached file cannot be read or its content cannot be hashed, `dotagents deploy` SHALL treat it as a cache miss and re-download the file.

#### Scenario: Unreadable cache file — re-download
- **WHEN** the cached `command.hbs` for a provider exists but cannot be read (permissions error or corruption)
- **THEN** a debug-level log is emitted, the file is re-downloaded, and the cache is overwritten
