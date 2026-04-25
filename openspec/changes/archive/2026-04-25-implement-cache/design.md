## Context

`dotagents deploy` currently re-renders and re-writes every target file on every run. The codebase already has a `CacheConfig` struct and `cache.toml` path constant (`src/schema/config/cache.rs`, `src/constants/file.rs`), but they are not wired into the deploy pipeline. `CacheConfig` currently mirrors the grouped `Providers` struct and stores a single `hash: Option<String>` per `(provider, feature)` pair inside `FeatureSettings` — a shape that cannot distinguish per-item hashes for multi-file features like `commands`. The deploy loop in `src/cli/deploy.rs` uses `rayon::par_iter` over providers, so any cache-write path must be thread-safe.

## Goals / Non-Goals

**Goals:**
- Skip writing target files whose rendered content hasn't changed since the last deploy
- Detect user-manually-edited target files and warn + skip instead of overwriting them
- `--force` flag: overwrite all files regardless of cache; still update cache on write
- `--no-cache` flag: skip reading and writing cache entirely for this run
- Cache file (`.dotagents/cache.toml`) is gitignored — never shared across a team

**Non-Goals:**
- Template fetch caching (separate future proposal)
- Three-way merge of user edits with new rendered output
- Per-field diff or smart merging of TOML/JSON target files
- Backward compatibility with the existing `CacheConfig` shape (it will be redesigned)

## Decisions

### 1. New dedicated `CacheConfig` data model (not reusing `FeatureSettings.hash`)

**Decision**: Replace the current `CacheConfig` (which mirrors the grouped `Providers` struct) with a purpose-built cache data model:

```rust
struct CacheEntry {
    hash: String,    // SHA-256 hex of the content we last wrote
    target: String,  // absolute path of the target file
}

struct CacheConfig {
    // providers.<name>.<feature>.<item> = CacheEntry
    // For singletons (mcp, instructions): item key is "" (empty string)
    providers: HashMap<String, HashMap<String, HashMap<String, CacheEntry>>>,
}
```

TOML shape:
```toml
[providers.claude.commands.hello]
hash   = "a1b2c3..."
target = "/path/.claude/commands/hello.md"

[providers.claude.mcp]
hash   = "d4e5f6..."
target = "/path/.mcp.json"
```

For singletons, the third-level key is omitted (the table itself holds `hash` and `target`). In Rust, singletons use a fixed sentinel key `"_"` internally, and serialization collapses it.

**Alternatives considered**:
- Reuse `FeatureSettings.hash` (single `Option<String>`) — cannot handle per-item commands without encoding a map as a string. Rejected.
- Store all hashes in a flat `HashMap<String, String>` keyed by a composite `"provider:feature:item"` string — workable but awkward to query and TOML-unfriendly. Rejected.

### 2. One stored hash per entry (rendered = written)

**Decision**: Store only the hash of what was last written. At deploy time:

```
rendered = render(provider, feature, item)
rendered_hash = sha256(rendered)
stored = cache.get(provider, feature, item)

if stored is None:
    write + cache.set(rendered_hash)          // first time
else if sha256(read(target)) != stored.hash:
    warn("target manually edited, skipping") // user edited
else if rendered_hash == stored.hash:
    skip                                      // nothing changed
else:
    write + cache.set(rendered_hash)          // inputs changed
```

When `--force` is set: always write, always update cache.
When `--no-cache` is set: always write, never read/write cache.toml.

**Why one hash works**: at write time, rendered content == written content, so one hash captures both. Divergence between the stored hash and the on-disk file hash is unambiguous evidence of a user edit.

### 3. SHA-256 via the `sha2` crate

**Decision**: Add `sha2` (+ `hex` for formatting) to `[dependencies]`. Produce a 64-char lowercase hex string. The `sha2` crate is small, widely used, and produces stable output across Rust versions.

**Alternatives considered**:
- `std::collections::hash_map::DefaultHasher` — not stable across Rust versions (explicitly not guaranteed by std). Rejected.
- `blake3` crate — faster but not necessary for this use case; adds a heavier dependency. Rejected.
- MD5 — deprecated for content integrity. Rejected.

### 4. Load-render-collect-flush pattern for thread safety

**Decision**: Load `cache.toml` into memory once before deploy begins. During the parallel provider iteration (rayon), cache lookups are read-only against the loaded snapshot. Pending writes are collected into a `Mutex<Vec<CacheUpdate>>`. After all providers for a feature are done, the mutex is drained, the in-memory cache is updated, and `cache.toml` is written once per feature (or once at the very end of deploy).

**Alternatives considered**:
- `DashMap` (concurrent hashmap) — adds a dependency and is heavier than needed. Rejected.
- `Arc<Mutex<CacheConfig>>` with per-entry locking — fine but `Mutex<Vec<update>>` + single flush is simpler. Accepted.
- Write `cache.toml` atomically at the very end only — preferred for simplicity; risk is if deploy crashes mid-way, the cache is never updated (acceptable: next run just re-renders everything).

### 5. Cache file lives at `.dotagents/cache.toml`, listed in `.gitignore`

**Decision**: Use the existing `CACHE_CONFIG_FILE` constant path. The mock `.gitignore` written by `init` will be updated to include `cache.toml`. No other location is needed.

## Risks / Trade-offs

- **Cache becomes stale if `cache.toml` is deleted or corrupted** → Treat any read error as a cache miss; log a debug warning. No data loss — worst case is a full re-render.
- **Parallel rayon writes to the same `Mutex` contend** → Contention is minimal: the mutex only guards a `Vec` append; render work (the slow path) is fully parallel. Acceptable.
- **`--force` and `--no-cache` are easy to confuse** → `--force` = "overwrite even user-edited files but still track in cache"; `--no-cache` = "don't touch cache.toml at all". Both are documented in `--help`.
- **Cache is keyed by provider+feature+item, not by target path** → If the user changes the `target` path in config, the cache entry becomes orphaned (old entry never matched). This is harmless — next deploy writes the new path and adds a new entry. Old entries accumulate but don't cause errors. A future `dotagents cache prune` command could clean up.
- **Must be updated after `flatten-providers` lands** → `cache.rs` currently uses the grouped `Providers` struct. The new `CacheConfig` is independent, but if `implement-cache` is merged before `flatten-providers`, the existing `has_valid_hash` method (which branches on "ide"/"cli"/"custom") needs a stub fix. Recommended: implement after `flatten-providers` to avoid the conflict.

## Open Questions

*(none)*
