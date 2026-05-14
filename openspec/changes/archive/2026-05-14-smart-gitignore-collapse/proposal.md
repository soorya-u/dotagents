## Why

The current `.gitignore` fence lists every generated file individually. With 8 skills and 3 providers, the fence already has 51 entries and grows linearly with each new skill/command/provider. This makes `.gitignore` noisy and hard to read. A smart collapsing algorithm can reduce 51 entries to ~7 by using directory patterns when all contents of a directory are generated.

## What Changes

- Add a tree-based collapsing algorithm in `src/utils/gitignore.rs` that groups generated paths and collapses directories whose entire contents are generated files, walking up the tree with filesystem checks to find the highest safe collapse point.
- **BREAKING**: Replace the additive `write_gitignore` approach with a `rebuild_fence_from_cache` approach that reads all cached target paths and rewrites the fence from scratch using collapsed patterns.
- The `update_gitignore` function changes from additive (append-only) to full-rebuild (rewrite fence from cache each time).
- `undeploy_item` (used by `commands rm` / `skills rm`) calls the same rebuild function after removing cache entries, instead of the current `remove_paths_from_fence` which would fail to match collapsed patterns.
- Full `undeploy` continues to use `clear_gitignore_fence` unchanged.
- Directory patterns use trailing-slash gitignore convention (e.g. `.claude/commands/`).

## Capabilities

### New Capabilities
- `gitignore-collapse`: The tree-based algorithm that collapses individual file paths into directory patterns when all directory contents are generated.

### Modified Capabilities
- `deploy-gitignore-update`: Entries are no longer always individual file paths — directories whose contents are entirely generated are collapsed to directory patterns. The fence is rebuilt from cache rather than appended to additively. Stale-entry accumulation behavior is removed (fence is rebuilt each time).
- `rm-cleanup`: `undeploy_item` uses fence rebuild from cache instead of `remove_paths_from_fence`, so collapsed patterns are handled correctly after item removal.

## Impact

- `src/utils/gitignore.rs`: Core logic changes — new collapse algorithm, `write_gitignore` replaced with cache-aware rebuild.
- `src/cli/deploy.rs`: Calls `rebuild_fence_from_cache` instead of `write_gitignore` after cache save.
- `src/cli/undeploy.rs`: `undeploy_item` calls `rebuild_fence_from_cache` instead of `remove_paths_from_fence`. Full `undeploy` unchanged.
- `src/core/config/cache.rs`: May need a helper to extract all target paths.
- `tests/integration/gitignore.rs`: Existing tests need updating for new collapsed output format.
- `openspec/specs/deploy-gitignore-update/spec.md`: Requirements change (directory patterns allowed, rebuild instead of append).
- `openspec/specs/rm-cleanup/spec.md`: Gitignore cleanup behavior changes.
