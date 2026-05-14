## Context

The `.gitignore` fence currently lists every deployed file individually. With 8 skills deployed across 3 providers, the fence has 51 entries. The deploy flow uses an additive `update_gitignore` function that appends new paths, and `undeploy_item` uses `remove_paths_from_fence` to do exact-string removal. Both functions operate on individual file paths.

The `CacheConfig` stores every deployed file's absolute target path, and `cache.save()` runs before the gitignore step in deploy. This makes the cache a reliable source of truth for rebuilding the fence.

## Goals / Non-Goals

**Goals:**
- Reduce fence size by collapsing directories whose entire contents are generated
- Produce correct fence output for deploy, commands rm, skills rm, and undeploy flows
- Keep all logic in `src/utils/gitignore.rs`

**Non-Goals:**
- Changing the fence markers (`#region dotagents` / `#endregion dotagents`)
- Changing the deploy prompt behavior (--gitignore / --no-gitignore flags)
- Changing how `undeploy` (full) works — it still calls `clear_gitignore_fence`

## Decisions

**D1 — Rebuild fence from cache instead of additive updates**

Replace the current additive `update_gitignore` with a rebuild approach: read all target paths from cache, run collapse, rewrite the fence. This eliminates the problem where additive writes create redundant entries alongside collapsed directory patterns.

*Alternative: patch `update_gitignore` to check if a new path is already covered by a collapsed parent pattern.* Rejected because it adds complexity without solving the `undeploy_item` path, and the rebuild approach is simpler and always correct.

**D2 — Tree-based collapse algorithm with filesystem checks**

Build a trie from all generated workspace-relative paths. Walk the tree bottom-up (post-order). At each directory node, call `fs::read_dir` to check whether the on-disk directory contains only generated entries. If yes, mark as collapsible and continue up. If no (non-generated files/dirs exist), stop collapsing at the children.

Cache `read_dir` results in a `HashMap<PathBuf, bool>` to avoid redundant filesystem calls.

*Alternative: pure path-based collapse without filesystem checks.* Rejected because it would incorrectly collapse directories containing non-generated files (e.g. `.claude/` with `settings.json`).

*Alternative: per-path bottom-up walk.* Same result but redundant `read_dir` calls. The tree approach does one `read_dir` per unique directory.

**D3 — Directory patterns use trailing slash**

Collapsed directories are written as `path/to/dir/` (trailing slash), following standard `.gitignore` convention where trailing slash means "match only directories."

**D4 — `rebuild_fence_from_cache` as the single fence-writing function**

One new public function: `rebuild_fence_from_cache(cache_targets: &[PathBuf], workspace_root: &Path) -> Result<()>`. It:
1. Converts absolute paths to workspace-relative
2. Runs the collapse algorithm
3. Reads current `.gitignore`
4. Rewrites the fenced section (preserving content outside the fence)
5. Skips write if content unchanged

Called from:
- `deploy` (after `cache.save()`) — replaces `write_gitignore`
- `undeploy_item` (after `cache.remove()`) — replaces `remove_paths_from_fence`

**D5 — Collapse algorithm details**

```
fn collapse_paths(paths: &[String], workspace_root: &Path) -> Vec<String>
```

1. Separate root-level files (no parent dir) — these are never collapsible, emit as-is
2. Build a set of all generated paths (for O(1) lookup)
3. Collect all unique parent directories from the paths
4. For each leaf directory (deepest first):
   a. `read_dir` on the filesystem
   b. Check every entry: is it in the generated set, or is it a directory that is itself fully collapsible?
   c. If all entries are generated → mark directory as collapsible
5. Walk up: for each collapsible directory, check its parent the same way
6. Emit the highest collapsible ancestor for each group as `dir/` pattern
7. Emit individual paths for anything that couldn't be collapsed

**D6 — `undeploy_item` skips fence update when cache is empty after removal**

After `cache.remove()`, if the remaining cache has targets, call `rebuild_fence_from_cache`. If cache is now empty, call `clear_gitignore_fence` instead (same as full undeploy).

**D7 — Remove `remove_paths_from_fence` and `remove_paths_from_content`**

These functions become dead code after the switch to rebuild-from-cache. Remove them along with their tests.

## Risks / Trade-offs

**[Filesystem dependency]** The collapse algorithm reads the filesystem to determine if directories can be collapsed. If the filesystem is in an inconsistent state (e.g. files deleted outside dotagents), collapse results may differ from expectations. → Mitigation: This is acceptable per issue requirements; redeploy corrects any drift.

**[Breaking change to fence format]** Existing `.gitignore` files with individual entries will be rewritten to collapsed format on next deploy. → Mitigation: The fence is machine-managed; users interact with content outside the fence. The change is idempotent.

**[Performance on large workspaces]** Each unique directory in the generated tree gets one `read_dir` call. → Mitigation: Even with 10 providers × 10 features, this is ~100 `read_dir` calls — negligible.
