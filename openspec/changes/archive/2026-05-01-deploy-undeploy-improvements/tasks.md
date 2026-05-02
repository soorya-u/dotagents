## 1. Gitignore — Remove Directory Glob Support

- [x] 1.1 Remove `GitignoreScope::Directory` and `GitignorePath::Directory` variants from `src/utils/gitignore.rs`
- [x] 1.2 Update `gitignore_path_to_pattern()` to handle only `GitignorePath::File`
- [x] 1.3 Update `CommandFeature::gitignore_scope()` to return `GitignoreScope::File`
- [x] 1.4 Update `SkillFeature::gitignore_scope()` to return `GitignoreScope::File`
- [x] 1.5 Remove the `dir_entries: HashSet<PathBuf>` deduplication block from `deploy_feature()` in `src/cli/deploy.rs`; push one `GitignorePath::File` per written file directly
- [x] 1.6 Update or remove gitignore tests that reference `Directory` variant
- [x] 1.7 Run `mise check` and fix any compiler errors or clippy warnings

## 2. Gitignore — Add Fence Removal Utility

- [x] 2.1 Add `clear_gitignore_fence(workspace_root: &Path) -> Result<()>` to `src/utils/gitignore.rs` — reads `.gitignore`, strips lines from `FENCE_START` through `FENCE_END` inclusive (plus any leading blank line), writes back only if content changed
- [x] 2.2 Add unit tests for `clear_gitignore_fence`: fence present, no fence, fence with surrounding user content, already-clean file

## 3. Fix `--no-cache` Semantics

- [x] 3.1 In `src/cli/deploy.rs`, remove the `opts.no_cache` guard around cache initialisation — always initialise `Arc<Mutex<CacheConfig>>`
- [x] 3.2 Pass `force = true` to `render_feature_with_settings` when `opts.no_cache` is set (skip hash comparison) instead of passing `None` for the cache
- [x] 3.3 Remove the `opts.no_cache` guard around `cache.save()` — cache is always persisted
- [x] 3.4 Update the `--no-cache` help text in `src/cli/options.rs` to reflect new semantics ("skip hash comparison; cache is still written")
- [x] 3.5 Update or add tests in `src/schema/config/cache.rs` and deploy integration tests to cover new `--no-cache` behaviour

## 4. Deploy Stats and Outro

- [x] 4.1 Define `DeployStats { written: usize, skipped: usize, paths: Vec<GitignorePath> }` in `src/cli/deploy.rs` (or a shared types module)
- [x] 4.2 Change `deploy_feature()` return type from `Result<Vec<GitignorePath>>` to `Result<DeployStats>`; increment `written`/`skipped` counts based on `CacheUpdate` variant
- [x] 4.3 Accumulate a single `DeployStats` in `deploy()` by merging results from all four feature calls
- [x] 4.4 Add `print_deploy_summary(stats: &DeployStats)` to `src/cli/ui/deploy.rs` — prints "✓ N written, M skipped" (or "✓ Nothing deployed") only when `is_tty()`
- [x] 4.5 Call `print_deploy_summary()` in `deploy()` after cache save and before the gitignore block
- [x] 4.6 Update all `all_paths` references in `deploy()` to use `stats.paths`
- [x] 4.7 Run `mise check` and fix any compiler errors or clippy warnings

## 5. File System — Delete Utility

- [x] 5.1 Add `delete_file(path: &Path) -> Result<()>` to `src/utils/fs.rs` — thin wrapper around `std::fs::remove_file` with `.context()`
- [x] 5.2 Add `prune_empty_dir(path: &Path) -> Result<()>` to `src/utils/fs.rs` — removes the immediate parent directory if it exists and is empty after the file is deleted; ignores "not empty" errors silently
- [x] 5.3 Add unit tests for `delete_file` (existing file, missing file) and `prune_empty_dir` (empty parent, non-empty parent)

## 6. Cache — Iterator Utility

- [x] 6.1 Add `iter_entries(&self) -> impl Iterator<Item = (&str, &str, &str, &CacheEntry)>` to `CacheConfig` in `src/schema/config/cache.rs` — yields `(provider, feature, item, entry)` tuples
- [x] 6.2 Add `clear(&mut self)` to `CacheConfig` — sets `providers` to an empty `HashMap`
- [x] 6.3 Add unit tests for `iter_entries` and `clear`

## 7. Undeploy Command — Core

- [x] 7.1 Create `src/cli/ui/undeploy.rs` with `prompt_confirm_undeploy(count: usize) -> bool` (TTY yes/no prompt) and `prompt_delete_edited(path: &Path) -> bool` (per-file TTY prompt for user-edited files)
- [x] 7.2 Create `src/cli/undeploy.rs` with `pub(super) fn undeploy(opts: UndeployOptions) -> Result<()>` implementing the full undeploy flow:
  - Load cache; exit early if empty
  - Collect all `(target, hash)` pairs from `cache.iter_entries()`
  - TTY confirmation prompt (skip if `--force` or non-TTY)
  - For each target: check existence → check hash → delete or skip/warn/prompt based on TTY + force
  - Call `prune_empty_dir` after each deletion
  - Clear and save cache
  - Call `clear_gitignore_fence` unless `--no-gitignore`
  - Print TTY summary via `print_undeploy_summary()`
- [x] 7.3 Add `print_undeploy_summary(removed: usize, skipped: usize)` to `src/cli/ui/undeploy.rs`

## 8. Undeploy Command — CLI Wiring

- [x] 8.1 Define `UndeployOptions` struct in `src/cli/options.rs` with `--force` (`bool`) and `--no-gitignore` (`bool`) flags
- [x] 8.2 Add `Undeploy(UndeployOptions)` variant to the `Action` enum in `src/cli/options.rs`
- [x] 8.3 Add `mod undeploy; mod ui` (or extend existing `mod ui`) in `src/cli/mod.rs`
- [x] 8.4 Add `Action::Undeploy(opts) => { undeploy(opts)?; true }` arm in `src/cli/runner.rs`
- [x] 8.5 Verify `dotagents undeploy --help` displays correct description and flags

## 9. Tests and Verification

- [x] 9.1 Add integration tests in `tests/integration/` for undeploy: deploy then undeploy, verify files removed and cache cleared
- [x] 9.2 Add integration test: undeploy with no cache → "Nothing to undeploy" exit 0
- [x] 9.3 Add integration test: undeploy with `--no-gitignore` → `.gitignore` fence preserved
- [x] 9.4 Add integration test: deploy `--no-cache` → verify `cache.toml` is written
- [x] 9.5 Run `mise test-all` and fix any failures
