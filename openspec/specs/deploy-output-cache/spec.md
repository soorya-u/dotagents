## ADDED Requirements

### Requirement: Skip unchanged output files
`dotagents deploy` SHALL skip writing a target file when the rendered output is identical to what was last written, as determined by comparing the SHA-256 hash of the rendered content to the stored hash in `.dotagents/cache.toml`.

#### Scenario: No change — skip write
- **WHEN** the rendered output hash matches the stored cache hash and the target file content matches the stored hash
- **THEN** the target file is not written and deploy completes successfully with no file modification

#### Scenario: Inputs changed — write and update cache
- **WHEN** the rendered output hash does not match the stored cache hash
- **THEN** the target file is overwritten with the new rendered output and the cache entry is updated with the new hash

#### Scenario: No cache entry — write and populate cache
- **WHEN** no cache entry exists for the `(provider, feature, item)` tuple
- **THEN** the target file is written and a new cache entry is created

#### Scenario: Target file missing despite valid cache entry
- **WHEN** a valid cache entry exists but the target file does not exist on disk
- **THEN** the target file is written and the cache entry is updated

### Requirement: Detect and preserve user-edited target files
When the stored hash indicates the file should be unchanged but the on-disk content has diverged (user manually edited the target), `dotagents deploy` SHALL warn and skip rather than overwrite.

#### Scenario: User-edited file — warn and skip
- **WHEN** the rendered output hash matches the stored cache hash but the target file content does not match the stored hash
- **THEN** deploy logs a warning identifying the file and skips writing it; the cache entry is not updated; deploy continues to the next item

#### Scenario: Force flag overrides user-edit skip
- **WHEN** `--force` is passed and a target file has been manually edited
- **THEN** the file is overwritten with the rendered output and the cache entry is updated

### Requirement: `--force` flag overwrites all target files
When `--force` is passed to `dotagents deploy`, all target files SHALL be written regardless of cache state. The cache SHALL be updated after each write.

#### Scenario: Force deploy updates all files
- **WHEN** `dotagents deploy --force` is run
- **THEN** every target file is written and every cache entry is updated, even if rendered output is unchanged

### Requirement: `--no-cache` flag skips hash comparison only
When `--no-cache` is passed to `dotagents deploy`, the cache file SHALL NOT be read for hash comparison — all target files are rendered and written as if no cache existed. However, the cache SHALL still be written at the end of the run with the hashes and paths of every file deployed in that run.

#### Scenario: No-cache deploy skips reading cache for comparison
- **WHEN** `dotagents deploy --no-cache` is run
- **THEN** no hash comparison is performed; all target files are written unconditionally

#### Scenario: No-cache deploy still writes cache.toml
- **WHEN** `dotagents deploy --no-cache` completes
- **THEN** `cache.toml` is written with entries for every file that was deployed in that run

#### Scenario: No-cache and force produce identical observable file output
- **WHEN** both `--no-cache` and `--force` are passed
- **THEN** all target files are written (same as each flag alone); cache is written at the end

### Requirement: Cache file is per-machine and gitignored
The cache file SHALL be stored at `.dotagents/cache.toml` (`.dotagents-debug/cache.toml` in debug builds). The file SHALL be listed in the `.dotagents/.gitignore` scaffold written by `dotagents init` so it is not committed to version control.

#### Scenario: cache.toml is excluded from git by default
- **WHEN** `dotagents init` scaffolds a new `.dotagents/` directory
- **THEN** the generated `.gitignore` includes `cache.toml`

### Requirement: Cache read errors are treated as cache misses
If `cache.toml` cannot be read or parsed (missing file, corrupt TOML), deploy SHALL treat all entries as cache misses and proceed normally. A debug-level log message SHALL be emitted.

#### Scenario: Missing or corrupt cache.toml
- **WHEN** `cache.toml` does not exist or contains invalid TOML
- **THEN** deploy continues without error, writes all target files, and writes a fresh `cache.toml` at the end

### Requirement: Cache entries are keyed by provider, feature, and item name
Cache entries SHALL be keyed by the triple `(provider_name, feature_name, item_name)`. For singleton features (`mcp`, `instructions`), a fixed sentinel key SHALL be used in place of item name. For per-item features (`commands`), item name is the command name from frontmatter.

#### Scenario: Per-item cache entry for commands
- **WHEN** a `hello` command is deployed to the `claude` provider
- **THEN** a cache entry is stored under `providers.claude.commands.hello`

#### Scenario: Singleton cache entry for mcp
- **WHEN** the `mcp` feature is deployed to the `cursor` provider
- **THEN** a single cache entry is stored under `providers.cursor.mcp`

### Requirement: Single cache entry per unique target path for singleton features
After provider deduplication, `cache.toml` SHALL contain exactly one entry per unique target path for singleton features (instructions, MCP), not one per provider.

#### Scenario: Multiple providers deduped to single writer
- **WHEN** 3 providers target `AGENTS.md` for the instructions feature and dedup selects one winner
- **THEN** `cache.toml` contains one cache entry for `AGENTS.md` under the winning provider's key

#### Scenario: all_targets returns unique paths
- **WHEN** `CacheConfig::all_targets()` is called after a deduped deploy
- **THEN** the returned Vec contains each target path exactly once
