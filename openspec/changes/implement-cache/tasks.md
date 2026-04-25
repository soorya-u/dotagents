## 1. Dependencies and Hashing Utility

- [ ] 1.1 Add `sha2` and `hex` crates to `[dependencies]` in `Cargo.toml`
- [ ] 1.2 Add `hash_content(content: &str) -> String` helper in `src/utils/fs.rs` that returns a SHA-256 hex string of the given string content
- [ ] 1.3 Add `hash_file(path: &PathBuf) -> Result<Option<String>>` helper in `src/utils/fs.rs` that returns `None` if the file doesn't exist and `Some(sha256_hex)` otherwise
- [ ] 1.4 Write unit tests for both hash helpers

## 2. New CacheConfig Data Model

- [ ] 2.1 Define `CacheEntry { hash: String, target: String }` in `src/schema/config/cache.rs`
- [ ] 2.2 Rewrite `CacheConfig` as `CacheConfig { providers: HashMap<String, HashMap<String, HashMap<String, CacheEntry>>> }` — keyed by `(provider, feature, item)`; use `"_"` as the sentinel item key for singleton features
- [ ] 2.3 Implement `CacheConfig::get(provider, feature, item) -> Option<&CacheEntry>` lookup method
- [ ] 2.4 Implement `CacheConfig::set(provider, feature, item, entry: CacheEntry)` insert/update method
- [ ] 2.5 Implement `CacheConfig::load() -> Result<Self>` — reads `cache.toml` from the config dir; returns `Self::default()` (empty) on missing file or parse error, emitting a debug log
- [ ] 2.6 Implement `CacheConfig::save(&self) -> Result<()>` — serializes to TOML and writes to `cache.toml`
- [ ] 2.7 Write unit tests for `get`, `set`, `load` (missing file), and `load` (corrupt file) behaviours

## 3. CLI Flags

- [ ] 3.1 Add `--force` boolean flag to the `Deploy` subcommand in `src/cli/options.rs`
- [ ] 3.2 Add `--no-cache` boolean flag to the `Deploy` subcommand in `src/cli/options.rs`
- [ ] 3.3 Pass both flags through to `deploy()` in `src/cli/deploy.rs`

## 4. Cache-Aware Deploy Logic

- [ ] 4.1 At the start of `deploy()`, load `CacheConfig` (skipped if `--no-cache`) and wrap in `Arc<Mutex<CacheConfig>>`
- [ ] 4.2 Refactor `render_feature_with_settings` in `src/templates/renderer.rs` (or the call site in `deploy.rs`) to accept a `cache: Option<&CacheEntry>` and `force: bool` and return a `CacheUpdate` enum (`Written(hash)` | `Skipped` | `UserEditedSkipped(path)`)
- [ ] 4.3 Implement skip logic: if `rendered_hash == stored_hash` AND `file_on_disk_hash == stored_hash` → return `Skipped`
- [ ] 4.4 Implement user-edit detection: if `rendered_hash == stored_hash` AND `file_on_disk_hash != stored_hash` → log warning with file path → return `UserEditedSkipped(path)` (unless `--force`)
- [ ] 4.5 Implement normal write path: write file, return `Written(rendered_hash)`
- [ ] 4.6 After each provider's feature iteration, drain `CacheUpdate` results and apply `Written` entries to the in-memory `CacheConfig`
- [ ] 4.7 After all features are deployed, call `cache.save()` (skipped if `--no-cache`)

## 5. Init Scaffold Update

- [ ] 5.1 Add `cache.toml` to the mock `.gitignore` content in `src/mocks/` so `dotagents init` gitignores it by default

## 6. Verification

- [ ] 6.1 Run `cargo build` — no compilation errors
- [ ] 6.2 Run `cargo test` — all tests pass
- [ ] 6.3 Run `dotagents deploy` twice in succession; confirm second run logs "skipped" for all files and `cache.toml` is written
- [ ] 6.4 Manually edit a deployed target file, then run `dotagents deploy`; confirm a warning is logged and the file is not overwritten
- [ ] 6.5 Run `dotagents deploy --force`; confirm the manually-edited file is overwritten and the warning is not shown
- [ ] 6.6 Run `dotagents deploy --no-cache`; confirm `cache.toml` is not modified
- [ ] 6.7 Run `cargo fmt && cargo clippy` — no warnings
