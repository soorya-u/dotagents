## Context

`CacheConfig` (in `src/schema/config/cache.rs`) has `get`, `set`, `iter_entries`, and `clear` — but no per-item `remove`. `iter_entries` yields `(provider, feature, item, entry)` tuples, which is sufficient to find all provider entries for a given `(feature, item)` pair.

`src/utils/gitignore.rs` has `clear_gitignore_fence` (removes the entire fence) and `write_gitignore` (adds paths) — but no function to remove specific paths from the fence. A new `remove_paths_from_fence(paths, workspace_root)` is needed.

`skills rm` and `commands rm` currently do not load the cache at all. Both are in `src/cli/skills.rs` and `src/cli/commands.rs` respectively.

## Goals / Non-Goals

**Goals:**
- After `skills rm` or `commands rm`, all deployed files, cache entries, and gitignore entries for the removed item are cleaned up across every provider.
- Cleanup failures are non-fatal — they warn but do not fail the overall `rm`.
- The `undeploy_item` logic is shared between skills and commands.

**Non-Goals:**
- Full undeploy of all items (that is `dotagents undeploy`).
- Prompting the user before cleanup — it is unconditional.
- Any changes to the deploy pipeline itself.

## Decisions

**D1 — `undeploy_item` lives in `src/cli/deploy.rs`**

It needs `CacheConfig`, `gitignore` utilities, and `get_workspace_dir` — all already imported in `deploy.rs`. Placing it there avoids a new file while keeping the function near the rest of the deploy/undeploy logic. It is `pub(crate)` so `skills.rs` and `commands.rs` can call it.

Alternative: new `src/utils/cleanup.rs`. Reasonable but adds a file for what is essentially ~40 lines of code. Deferred.

**D2 — `CacheConfig::remove(provider, feature, item)` method**

Add to `src/schema/config/cache.rs`. Implementation: `self.providers.get_mut(provider)?.get_mut(feature)?.remove(item)`. Returns `Option<CacheEntry>` (the removed entry, or `None` if absent).

**D3 — `remove_paths_from_fence` in `src/utils/gitignore.rs`**

Signature: `pub(crate) fn remove_paths_from_fence(paths: &[String], workspace_root: &Path) -> Result<()>`.

Implementation: read `.gitignore`, rebuild the fence without the given paths, write back if changed. Reuses `read_gitignore`, `write_file`. If the fence becomes empty after removal, the fence markers themselves are also removed (call the existing `remove_fence` logic).

**D4 — `undeploy_item` algorithm**

```text
fn undeploy_item(feature: &str, item_key: &str, cache: &mut CacheConfig, workspace_dir: &Path)
```

1. Collect all `(provider, target_path)` pairs from `cache.iter_entries()` where `feature` and `item` match.
2. If empty → `warn!("No deployed files found for '{}' — was it ever deployed?", item_key)` → return.
3. For each `(provider, target_path)`:
   a. Delete the file at `target_path`. `NotFound` → silently continue. Other errors → `warn!` and continue.
   b. Collect `target_path` for gitignore removal.
   c. `cache.remove(provider, feature, item_key)`.
4. Call `remove_paths_from_fence(&collected_paths, workspace_dir)`. On error → `warn!`.
5. `cache.save()`.

**D5 — Call site in skills rm and commands rm**

Both load `CacheConfig::load()` before the source removal, then call `undeploy_item` after it. `get_workspace_dir()` is already available. Cache load failure → `warn!` and skip cleanup (do not fail the `rm`).

## Risks / Trade-offs

- **`iter_entries` collects before mutating** — Rust borrow checker prevents mutating `cache` while iterating it. Collect `(provider, target)` pairs into a `Vec` first, then call `cache.remove` in a second pass. Straightforward.
- **Race condition: file deleted between deploy and rm** — handled by the `NotFound` silent-continue rule.
- **Partial cleanup on multi-provider setups** — if one provider's file fails to delete, the function warns and continues, so other providers are still cleaned. Cache entry for the failed provider is still removed (the source file is gone — retrying the delete later is pointless).
