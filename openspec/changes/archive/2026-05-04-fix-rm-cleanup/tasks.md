## 1. Add CacheConfig::remove method

- [x] 1.1 In `src/schema/config/cache.rs`, add `pub fn remove(&mut self, provider: &str, feature: &str, item: &str) -> Option<CacheEntry>` that removes and returns the entry at `(provider, feature, item)`
- [x] 1.2 Add unit test `cache_remove_returns_entry_and_deletes_it` — set an entry, remove it, assert it returns the entry and a subsequent get returns None
- [x] 1.3 Add unit test `cache_remove_returns_none_when_absent`

## 2. Add remove_paths_from_fence to gitignore utils

- [x] 2.1 In `src/utils/gitignore.rs`, add `pub(crate) fn remove_paths_from_fence(paths: &[String], workspace_root: &Path) -> Result<()>` that reads `.gitignore`, removes the given paths from inside the fence, removes the fence markers if empty, and writes back only if changed
- [x] 2.2 Add unit test `remove_paths_from_fence_removes_specified_paths`
- [x] 2.3 Add unit test `remove_paths_from_fence_removes_fence_markers_when_empty`
- [x] 2.4 Add unit test `remove_paths_from_fence_is_noop_when_path_not_present`

## 3. Add undeploy_item helper

- [x] 3.1 In `src/cli/deploy.rs`, add `pub(crate) fn undeploy_item(feature: &str, item_key: &str, cache: &mut CacheConfig, workspace_dir: &Path) -> Result<()>` with the algorithm from design D4: collect matching cache entries, delete files, remove gitignore paths, remove cache entries, save cache, warn if nothing found
- [x] 3.2 Handle `NotFound` file deletion errors silently; warn on all other errors
- [x] 3.3 Add unit test `undeploy_item_warns_when_no_cache_entries`
- [x] 3.4 Add unit test `undeploy_item_deletes_deployed_file_and_clears_cache`
- [x] 3.5 Add unit test `undeploy_item_continues_when_file_already_deleted`

## 4. Wire into skills rm

- [x] 4.1 In `src/cli/skills.rs` `rm_skill()`, load `CacheConfig::load()` before `fs::remove_dir_all`; on load error, `warn!` and skip cleanup
- [x] 4.2 After `fs::remove_dir_all`, call `undeploy_item("skills", &opts.name, &mut cache, &workspace_dir)`
- [x] 4.3 Add `get_workspace_dir` import if not already present

## 5. Wire into commands rm

- [x] 5.1 In `src/cli/commands.rs` `rm_command()`, load `CacheConfig::load()` before `fs::remove_file`; on load error, `warn!` and skip cleanup
- [x] 5.2 After `fs::remove_file`, call `undeploy_item("commands", &opts.name, &mut cache, &workspace_dir)`
- [x] 5.3 Add `get_workspace_dir` import if not already present

## 6. E2E tests

- [x] 6.1 Add e2e test: `skills rm` removes deployed file — create skill, deploy, rm, assert deployed file gone and gitignore entry removed
- [x] 6.2 Add e2e test: `commands rm` removes deployed file — same pattern for commands
- [x] 6.3 Add e2e test: `skills rm` with never-deployed skill — assert warning message appears and command still exits 0

## 7. Verification

- [x] 7.1 Run `mise check && mise tests` and fix any failures
