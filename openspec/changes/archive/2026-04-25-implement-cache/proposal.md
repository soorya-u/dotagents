## Why

Every `dotagents deploy` re-renders and re-writes every target file, even when nothing has changed — causing spurious file-modification timestamps, unnecessary IDE file-watcher triggers, and silent overwrites of files the user may have edited manually. A lightweight output-hash cache eliminates all three problems with minimal complexity.

## What Changes

- After rendering each `(provider, feature, item)` tuple, compute a SHA-256 of the rendered output and compare it against a stored hash in `.dotagents/cache.toml`
- If the rendered hash matches the stored hash **and** the target file on disk matches the stored hash → skip the write (nothing changed)
- If the rendered hash matches but the target file content has diverged from the stored hash → the user manually edited the target → **warn and skip** (do not overwrite)
- New `--force` flag on `dotagents deploy` → overwrite all target files regardless of cache state
- New `--no-cache` flag on `dotagents deploy` → bypass cache entirely (render and write everything, do not read or update `cache.toml`)
- `cache.toml` is per-machine and gitignored; it is never shared across a team
- `CacheConfig` struct and `cache.toml` path already exist in the codebase — this change wires them into the deploy pipeline
- Template fetch caching is explicitly out of scope

## Capabilities

### New Capabilities

- `deploy-output-cache`: Defines the caching behaviour for deploy output — hash storage, skip logic, user-edit detection (warn + skip), `--force` override, and `--no-cache` bypass.

### Modified Capabilities

*(none — no existing specs)*

## Impact

- `src/cli/deploy.rs` — integrate cache read/write around `render_feature_with_settings`
- `src/cli/options.rs` — add `--force` and `--no-cache` flags to the `Deploy` subcommand
- `src/schema/config/cache.rs` — extend `CacheConfig` / `CacheEntry` with `output_hash` and `target_hash` fields; add read/write helpers
- `src/utils/fs.rs` — add a helper to hash a file's contents (SHA-256)
- `src/utils/` or `src/schema/config/cache.rs` — add cache load/save logic (TOML read/write)
- `.dotagents/.gitignore` (mock) — ensure `cache.toml` is listed
- No new external dependencies required (`sha2` crate or stdlib hashing — TBD in design)
