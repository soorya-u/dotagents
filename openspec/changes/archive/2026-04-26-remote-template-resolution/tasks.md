## 1. Dependencies and Path Utilities

- [x] 1.1 Run `cargo add dirs` to add the `dirs` crate as a runtime dependency
- [x] 1.2 Add `get_global_template_cache_dir() -> Result<PathBuf>` to `src/utils/path.rs` using `dirs::config_dir()` joined with `"dotagents/cache/templates"`
- [x] 1.3 Add `TEMPLATE_CACHE_SUBDIR: &str = "templates"` constant to `src/constants/dir.rs`
- [x] 1.4 Write unit tests for `get_global_template_cache_dir()` covering: returns a path ending in `dotagents/cache/templates`, errors gracefully when home directory is unavailable

## 2. Registry Schema and CI Script

- [x] 2.1 Update `scripts/ci/generate_registry.sh` to compute `sha256sum` for each file (`provider.toml`, `command.hbs`, `instruction.hbs`, `mcp.hbs`, `skill.hbs`) that exists in each provider directory and embed results as a `checksums` object in the provider entry
- [x] 2.2 Verify `public/v1/schemas/registry.schema.json` (already created) validates correctly against the updated `registry.json` format by running a local generate and checking output structure
- [x] 2.3 Regenerate `public/v1/templates/registry.json` locally using the updated script and commit the result with checksums populated

## 3. Registry Deserialisation Module

- [x] 3.1 Create `src/schema/registry.rs` with `Registry` struct (`providers: HashMap<String, ProviderRegistryEntry>`) and `ProviderRegistryEntry` struct (`path: String`, `checksums: Option<HashMap<String, String>>`)
- [x] 3.2 Derive `Deserialize` (serde) on both structs; use `#[serde(rename_all = "kebab-case")]` where appropriate
- [x] 3.3 Add a `Registry::fetch(url: &str) -> Result<Registry>` method that calls `remote::do_get()` and deserialises the JSON response
- [x] 3.4 Expose the module via `src/schema/mod.rs`
- [x] 3.5 Write unit tests for `Registry` deserialisation covering: full entry with checksums, entry without checksums field, unknown extra fields are ignored

## 4. Template-Source Cache Module

- [x] 4.1 Create `src/templates/template_cache.rs` with a `TemplateCache` struct wrapping the `get_global_template_cache_dir()` path
- [x] 4.2 Implement `TemplateCache::checksum_matches(provider: &str, filename: &str, expected: &str) -> bool` — reads the cached file (if present), computes SHA-256, compares to `expected`
- [x] 4.3 Implement `TemplateCache::read(provider: &str, filename: &str) -> Result<Option<String>>` — returns `None` if file absent or unreadable (debug-level log on error)
- [x] 4.4 Implement `TemplateCache::write(provider: &str, filename: &str, content: &str) -> Result<()>` — creates parent dirs as needed, writes file
- [x] 4.5 Expose `TemplateCache` from `src/templates/mod.rs`
- [x] 4.6 Write unit tests using a `tempfile::TempDir` for: cache miss returns None, write then read round-trips correctly, checksum_matches returns true on match and false on mismatch

## 5. Provider.toml Resolution Helper

- [x] 5.1 Create `src/templates/registry_resolver.rs` with `resolve_provider_defaults(app_config: &mut AppConfig, registry: Option<&Registry>, cache: &TemplateCache, offline: bool) -> Result<()>`
- [x] 5.2 Implement the per-provider-feature loop: for each provider in `app_config` whose `FeatureSettings` is missing `template` or `target`, call the resolution chain (registry → cache → skip)
- [x] 5.3 Implement `fetch_or_cache_file(provider: &str, filename: &str, url: &str, registry_checksum: Option<&str>, cache: &TemplateCache, no_cache: bool) -> Result<Option<String>>` — the shared download-or-serve-from-cache logic for both `provider.toml` and `.hbs` files
- [x] 5.4 Implement `parse_provider_toml(content: &str, provider: &str, feature: &Feature) -> Result<Option<FeatureSettings>>` — deserialises provider.toml as `GlobalConfig` and extracts the relevant `FeatureSettings`
- [x] 5.5 Add warning log when: provider not found in registry, provider found but feature absent, registry fetch failed, cache cold for offline mode
- [x] 5.6 Add hard error for offline mode when cache is cold (per spec)
- [x] 5.7 Write unit tests using `mockito` for: registry fetch failure falls back to cache, checksum match uses cache, checksum mismatch triggers re-download, offline + cold cache returns error, provider absent from registry logs warning and skips

## 6. Deploy Integration

- [x] 6.1 Add `--offline` boolean flag to `DeployOptions` in `src/cli/options.rs` with doc comment "Skip the remote registry fetch; resolve missing templates from the local cache only"
- [x] 6.2 In `deploy.rs`, after `AppConfig::from_application()` and before the first `deploy_feature()` call: fetch `registry.json` (unless `--offline` or all providers are fully configured), construct `TemplateCache`, call `resolve_provider_defaults()`
- [x] 6.3 Pass `opts.no_cache` through to `resolve_provider_defaults()` so `--no-cache` forces re-download of template source files
- [x] 6.4 Ensure `registry.json` is fetched at most once per `deploy` invocation regardless of provider count

## 7. Verification

- [x] 7.1 Run `mise check` (cargo fmt + cargo clippy) and fix all warnings
- [x] 7.2 Run `mise test-all` (unit + integration + e2e) and fix all failures
- [x] 7.3 Manual smoke test: create a fresh project with `targets = ["claude"]` and no `[providers.*]` block; run `dotagents deploy` and confirm `.claude/commands/` is populated (automated as `#[ignore]` e2e test `deploy_auto_resolves_template_and_target_for_known_provider`)
- [x] 7.4 Manual smoke test: run `dotagents deploy --offline` on a cold cache and confirm a clear error message; run once online to warm cache, then run `--offline` again and confirm success (cold-cache case automated as always-on e2e tests; warm-cache case automated as `#[ignore]` e2e test `deploy_offline_with_warm_cache_succeeds_without_network`)
- [x] 7.5 Manual smoke test: run `dotagents deploy --no-cache` and confirm template files are re-downloaded even when checksum would otherwise match (automated as `#[ignore]` e2e test `deploy_no_cache_forces_re_download_even_when_cached`)
