# Tasks: Symlink Deploy Mode

## 1. FeatureTrait Extensions
- [x] Add `is_symlinkable(&self) -> bool` method with default `false` to `FeatureTrait`
- [x] Add `get_source_path(&self) -> Option<PathBuf>` method with default `None` to `FeatureTrait`
- [x] Override `is_symlinkable() -> true` on `SkillFeature`, `IgnoreFeature`
- [x] Override `is_symlinkable() -> false` (default) on `CommandFeature`, `InstructionFeature`, `McpFeature`
- [x] Add `source_path: PathBuf` field to `SkillFeature`, set during `from_application()`
- [x] Add `source_path: PathBuf` field to `IgnoreFeature`, set during `from_application()`
- [x] Add `source_path: PathBuf` field to `CommandFeature` for future use

## 2. Config Schema
- [x] Define `FeatureMode` enum (`Link`, `Template`) with `Serialize + Deserialize`
- [x] Define `FeatureModeConfig` struct: `mode: Option<FeatureMode>`, `mode_override: Option<HashMap<String, FeatureMode>>`
- [x] Add `feature_maps: HashMap<String, FeatureModeConfig>` to `GlobalConfig` and `LocalConfig`
- [x] Implement merge logic for `FeatureModeConfig` (local overrides global per-feature, override maps merged)
- [x] Add `serde(rename_all = "kebab-case")` to the new structs
- [x] Validate mode values on deserialization (reject anything not `"link"` or `"template"`)
- [x] Validate mode_override values similarly

## 3. Mode Resolution
- [x] Add `resolve_mode(feature: &str, item_name: Option<&str>) -> FeatureMode` to `AppConfig`
- [x] Resolution: item override → feature-level → hardcoded `Link` default
- [x] Unit test all resolution paths

## 4. Template Field Optionality
- [x] Add `is_provider_agnostic()` associated fn to `FeatureTrait` (default `false`; `SkillFeature` and `IgnoreFeature` override to `true`)
- [x] In `resolve_provider_defaults()`: when resolving defaults from registry, don't error if Type 1 has no `template`
- [x] In `render_feature_with_settings()`: require `template` only for Type 2 or mode=template Type 1
- [x] Error message when Type 2 feature has no `template` configured

## 5. Symlink Write Utility
- [x] Add `write_symlink(source: &Path, target: &Path) -> Result<()>` to `src/utils/fs.rs`
- [x] Create parent directories for target if needed
- [x] Handle existing target (remove first, then symlink)
- [x] Guard: `#[cfg(unix)]` with compile error on non-Unix
- [x] Add matching `#[cfg(windows)]` implementation

## 6. Link Feature Renderer
- [x] Implement `link_feature_with_settings<T: FeatureTrait>()` in `src/templates/renderer.rs`
- [x] Resolve target path from settings (Phase 1 only)
- [x] Call `feature.get_source_path()` for symlink source
- [x] Call `write_symlink(source, target)`
- [x] Return `CacheUpdate::Linked { target: String }` (new variant)

## 7. Render Feature Modifications
- [x] Add `mode: FeatureMode` parameter to `render_feature_with_settings()`
- [x] Skip `populate_with_values()` (Phase 2) when mode=Link
- [x] Skip template loading/rendering (Phase 3) when Type 1 (no template needed)
- [x] Keep full pipeline for Type 2 + mode=Template (existing behavior)

## 8. Deploy Loop Branching
- [x] In `deploy_feature()`: resolve mode before dispatching to renderer
- [x] Branch: `is_symlinkable() && mode == Link` → `link_feature_with_settings()`
- [x] Else → `render_feature_with_settings()` with mode parameter

## 9. Cache Changes
- [x] Add `Linked { target: String }` variant to `CacheUpdate` enum
- [x] In `process_cache_update()`: skip `cache.set()` for `Linked` variant (no cache entry)
- [x] In `DeployStats`: track linked items for `.gitignore` fence
- [x] In `finalize_deploy()`: collect linked targets alongside `cache.all_targets()`

## 10. Skills Extra Files (#163)
- [x] In skill deploy path: after deploying SKILL.md, enumerate skill directory
- [x] For each entry not named `SKILL.md`:
  - [x] Compute target path relative to SKILL.md's target directory
  - [x] Create symlink from source to target
  - [x] Track in deploy stats for `.gitignore` fence
- [x] Handle nested directories (mirror structure)
- [ ] Test with empty extra dirs, deep nesting, symlink overwrite

## 11. Provider Registry Updates
- [x] Remove `template` field from `[providers.<p>.skills]` in all `provider.toml` files under `public/v1/templates/`
- [x] Remove `template` field from `[providers.<p>.ignore]` in all `provider.toml` files that have it
- [x] Verify `resolve_provider_defaults()` handles missing `template` gracefully for Type 1
- [ ] Regenerate `registry.json`

## 12. Testing
- [ ] Test with tui-devtools: run `dotagents init` flow, verify no config changes
- [ ] Test with tui-devtools: run `dotagents deploy` flow with skills in link mode, verify symlinks created
- [x] Unit tests: `FeatureMode` serialization/deserialization
- [x] Unit tests: mode resolution with all override combinations
- [ ] Unit tests: `link_feature_with_settings()` happy path and error cases
- [ ] Unit tests: `render_feature_with_settings()` with mode branching
- [ ] Unit tests: cache skips `Linked` variant
- [ ] E2E tests: deploy skills with mode=link creates symlinks
- [ ] E2E tests: deploy skills with mode=template writes files
- [ ] E2E tests: skills extra files symlinked
- [ ] E2E tests: dedup works with symlink deploy
- [ ] E2E tests: `--dry-run` reports symlink operations
- [ ] E2E tests: `.gitignore` fence includes symlinked paths

## Verification
- [x] `mise check` passes (format + lint)
- [x] `mise tests` passes (unit + integration + e2e)
