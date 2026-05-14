## 1. Collapse Algorithm

- [x] 1.1 Add `collapse_paths(paths: &[String], workspace_root: &Path) -> Vec<String>` in `src/utils/gitignore.rs` that takes workspace-relative paths, builds a trie, walks bottom-up with `read_dir` filesystem checks (caching results in a HashMap), and returns collapsed patterns (directories with trailing slash, individual files otherwise)
- [x] 1.2 Add unit tests for `collapse_paths`: all files in a directory collapse to `dir/`; non-generated file in directory prevents collapse; nested collapse (children collapse, then parent collapses); root-level files remain individual; empty input returns empty output; directory with mix of collapsible subdirs and non-generated files stops at subdirs

## 2. Rebuild Fence From Cache

- [x] 2.1 Add `rebuild_fence_from_cache(cache_targets: &[PathBuf], workspace_root: &Path) -> Result<()>` in `src/utils/gitignore.rs` that converts absolute paths to workspace-relative, calls `collapse_paths`, reads current `.gitignore`, rewrites the fenced section with collapsed patterns, and skips write if content unchanged
- [x] 2.2 Refactor `update_gitignore` to accept the full set of patterns and rewrite the fence section (not append-only) — or replace it with an internal helper used by `rebuild_fence_from_cache`
- [x] 2.3 Add unit tests for `rebuild_fence_from_cache`: creates fence when none exists; rewrites existing fence with new collapsed content; preserves user content outside fence; skips write when content unchanged; handles missing `.gitignore` file

## 3. Integrate With Deploy

- [x] 3.1 In `src/cli/deploy.rs`, replace the `write_gitignore` call with `rebuild_fence_from_cache`, passing all target paths from the saved cache
- [x] 3.2 Add a helper on `CacheConfig` (or in deploy.rs) to extract all target paths as `Vec<PathBuf>` from the cache for passing to `rebuild_fence_from_cache`

## 4. Integrate With Undeploy Item

- [x] 4.1 In `src/cli/undeploy.rs` `undeploy_item`, replace the `remove_paths_from_fence` call with `rebuild_fence_from_cache` using remaining cache targets after removal; if cache is empty after removal, call `clear_gitignore_fence` instead

## 5. Cleanup Dead Code

- [x] 5.1 Remove `remove_paths_from_fence`, `remove_paths_from_content`, and their unit tests from `src/utils/gitignore.rs`
- [x] 5.2 Remove `write_gitignore`, `GitignorePath` enum, and `gitignore_path_to_pattern` if they are no longer used after the refactor
- [x] 5.3 Remove any now-unused imports in `deploy.rs` and `undeploy.rs`

## 6. Update Existing Tests

- [x] 6.1 Update `tests/integration/gitignore.rs` to expect collapsed directory patterns instead of individual file entries
- [x] 6.2 Run `mise check` and `mise tests` — both must exit 0

## 7. Manual Testing

- [x] 7.1 Run `dotagents deploy` and verify `.gitignore` fence contains collapsed directory patterns
- [x] 7.2 Run `dotagents commands rm <name>` and verify fence is correctly rebuilt
- [x] 7.3 Run `dotagents undeploy` and verify fence is fully cleared
