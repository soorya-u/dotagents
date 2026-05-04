## Why

`dotagents skills rm` and `dotagents commands rm` remove the source file from `.dotagents/` but leave three artefacts behind: the deployed file on disk (e.g. `.gemini/skills/my-skills/SKILL.md`), the cache entry in `cache.toml`, and the path in the `.gitignore` managed fence. The user must manually hunt and delete these, and the cache entry is never pruned — subsequent deploys accumulate stale state indefinitely.

## What Changes

- Add a shared `undeploy_item(feature, item_key, cache, workspace_dir)` helper that, after a source is removed:
  1. Finds all cache entries matching `(*, feature, item_key)` — one per provider.
  2. For each entry: deletes the deployed file (ignores not-found, warns on other errors); removes the path from the `.gitignore` `#region dotagents` fence; removes the cache entry.
  3. Saves the updated cache.
  4. If no cache entries are found, logs a warning: `"No deployed files found for '<name>' — was it ever deployed?"`.
- Call `undeploy_item` in `skills rm` after `fs::remove_dir_all`.
- Call `undeploy_item` in `commands rm` after `fs::remove_file`.

**Behaviour contract:**
- Cleanup is unconditional — it runs regardless of the `--deploy` flag.
- Deployed file missing on disk → silently continue.
- No cache entry found → warn and continue; do not fail.
- File deletion error (other than not-found) → warn and continue; do not fail the overall `rm`.

## Capabilities

### New Capabilities

- `rm-cleanup`: Removing a skill or command also removes deployed files, cache entries, and `.gitignore` fence entries for that item across all providers.

### Modified Capabilities

- `rm-command`: `commands rm` now cleans up deployed output in addition to removing the source file.

## Impact

- `src/cli/skills.rs` — load cache before `fs::remove_dir_all`; call `undeploy_item` after removal.
- `src/cli/commands.rs` — same pattern after `fs::remove_file`.
- `src/cli/deploy.rs` or new `src/utils/cleanup.rs` — add `undeploy_item` helper.
- `src/utils/gitignore.rs` — verify or add `remove_paths_from_fence(paths, workspace_dir)` used by `undeploy_item`.
- `src/schema/config/cache.rs` — verify or add `remove(provider, feature, item_key)` method.
- Tests: unit tests for `undeploy_item`; e2e tests asserting deployed file is gone and gitignore entry removed after `skills rm` / `commands rm`.
