## Context

`dotagents deploy` renders provider config files and tracks them in `.dotagents/cache.toml` (workspace-local). The cache maps `(provider, feature, item) → { hash, target }` where `target` is the absolute path of the deployed file. Today:

- `deploy_feature()` returns `Vec<GitignorePath>` — only written paths, no counts.
- `--no-cache` skips both reading and writing the cache entirely.
- The gitignore managed section uses directory globs (`commands/*`, `skills/*`) for multi-file features.
- There is no `undeploy` command, no `delete_file` utility, and no fence-removal function in `gitignore.rs`.

The `GitignoreScope::Directory` variant exists on `FeatureTrait` and is used by `CommandFeature` and `SkillFeature`. All other features use `GitignoreScope::File`.

## Goals / Non-Goals

**Goals:**
- Print a rich TTY-only deploy summary (written / skipped counts).
- Fix `--no-cache` to mean "skip hash comparison" rather than "skip cache entirely".
- Add `dotagents undeploy` that reverses a deploy using `cache.toml` as the sole source of truth.
- Record individual file paths (not directory globs) in the `.gitignore` managed section.

**Non-Goals:**
- Undeploy without a cache (re-deriving targets from config at undeploy time).
- Removing empty grandparent directories (only immediate parent is pruned).
- Multiple levels of undo / undo history.
- Cross-workspace undeploy.

## Decisions

### 1. Replace `Vec<GitignorePath>` return with `DeployStats`

`deploy_feature()` currently returns only the paths of written files as `Vec<GitignorePath>`. To support the deploy outro, skipped counts are also needed.

**Decision:** Introduce `DeployStats { written: usize, skipped: usize, paths: Vec<GitignorePath> }`. The `deploy()` function accumulates a `DeployStats` across all feature calls; the total counts are printed after cache save.

*Alternative considered: add a separate counter alongside the existing Vec.* Rejected — a named struct is more legible at the call site and easier to extend.

### 2. `--no-cache` suppresses comparison only; cache always written

The original `--no-cache` implementation skipped both the hash-comparison read and the end-of-deploy write. This prevented undeploy from having reliable cache state.

**Decision:** `--no-cache` passes `force = true` implicitly to `render_feature_with_settings` (skipping the hash comparison) but the cache `Arc<Mutex<CacheConfig>>` is always initialised and always saved. The flag is renamed semantically to "skip comparison" at the option-parsing layer.

*Alternative considered: keep --no-cache as-is and add a separate --no-cache-write flag.* Rejected — two flags for one concept adds complexity. The new meaning is a strict superset of `--force`; existing users who passed `--no-cache` get the same observable output-file behaviour.

### 3. Undeploy reads cache exclusively — no config re-derivation

The cache stores the absolute target path of every file written. Undeploy only needs to delete those paths.

**Decision:** `undeploy` loads `cache.toml`, iterates all entries, and deletes each `target`. If the cache is empty or missing, it exits early with a "nothing to undeploy" message. No config loading, registry fetch, or template rendering is performed.

*Alternative considered: re-run deploy in dry-run mode to discover targets.* Rejected — this couples undeploy to the full deploy pipeline and would fail if config has changed since the last deploy.

### 4. User-edited files during undeploy

A file is "user-edited" when its on-disk hash no longer matches the stored `CacheEntry.hash`.

**Decision:**
- **Non-TTY (CI)**: warn and skip the file; continue with others.
- **TTY (interactive)**: prompt once per file: "This file has been edited. Delete anyway? [y/N]".
- **`--force`**: delete without prompting or checking hash.

This mirrors the existing deploy-side handling of user-edited files (warn-and-skip / force-override).

### 5. `clear_gitignore_fence()` removes the entire managed block

Undeploy removes all dotagents-managed entries from `.gitignore` at once (the full `BEGIN…END` fence), rather than removing individual patterns.

**Decision:** Add `clear_gitignore_fence(workspace_root: &Path) -> Result<()>` to `gitignore.rs`. It reads `.gitignore`, strips the fenced section and the blank line preceding it, and writes back (skipping write if nothing changed). Implemented as a line-by-line pass: skip lines from `FENCE_START` through `FENCE_END` inclusive.

### 6. `GitignoreScope::Directory` variant removed

Directory globs (`commands/*`) were a shortcut to avoid enumerating individual command files. With individual paths now stored in cache (and required by the `deploy-gitignore-update` spec), the `Directory` variant serves no purpose.

**Decision:** Remove `GitignoreScope::Directory` and `GitignorePath::Directory` variants. All feature implementations return `GitignoreScope::File`. `deploy_feature()` pushes one `GitignorePath::File` entry per written file, directly. The `dir_entries` deduplication `HashSet` is also removed.

## Risks / Trade-offs

- **`--no-cache` is a behaviour change** → Existing scripts that passed `--no-cache` expecting no `cache.toml` side-effect will now find a written cache. This is unlikely to cause breakage (the cache file is already gitignored by `init`) but is technically non-additive. Mitigation: document in CHANGELOG.
- **Undeploy reliability depends on cache completeness** → Deploys run before this change (when `--no-cache` wrote nothing) leave gaps in cache. Mitigation: the fix lands together with undeploy in one release; any deploy after the release will populate the cache correctly. Undeploy warns "cache may be incomplete" when it finds an empty cache.
- **TTY detection at undeploy time** → The `is_tty()` helper already exists and is used by deploy. No new risk.
- **Empty directory pruning** → Pruning only the immediate parent minimises the chance of removing a directory the user cares about. If the parent still contains other files, it is left alone.

## Migration Plan

All changes are additive or fix bugs; no data migration is required. The `cache.toml` format is unchanged. The `.gitignore` fence format is unchanged (individual paths were already the spec-correct format).

Rollout: ship as a single release. No feature flags or phased rollout needed.
