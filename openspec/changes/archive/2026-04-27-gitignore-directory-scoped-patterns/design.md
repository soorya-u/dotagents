## Context

The existing gitignore pipeline collects deployed paths as `Vec<PathBuf>` and writes each one as a workspace-relative string into the fenced section. Features are one of two kinds:

- **Singleton** (`McpFeature`, `InstructionFeature`): one file per provider. Five providers → five entries. Manageable.
- **Per-item** (`CommandFeature`, `SkillFeature`): one file per item per provider. 20 commands × 3 providers = 60 entries that will all land inside `.kilo/commands/`, `.windsurf/workflows/`, etc. The feature *owns* those directories entirely — no user files live alongside them — so a glob pattern like `.kilo/commands/*` is safe and far more readable.

The key signal already in the codebase: `get_file_name()` returning `Some(...)` vs `None` implicitly distinguishes per-item from singleton features. This change makes that distinction explicit and routes it into the gitignore pipeline.

## Goals / Non-Goals

**Goals:**
- Add an explicit `gitignore_scope` method to `FeatureTrait` that communicates whether a feature writes many files into an owned directory
- Use that scope in `deploy_feature` to collect either `File(PathBuf)` or `Directory(PathBuf)` entries, deduplicating directories
- Write `dir/*` glob patterns for directory-scoped entries, exact paths for file entries
- Update the interactive prompt count to reflect entries, not raw files

**Non-Goals:**
- Supporting recursive globs (`**`) — one level of wildcard is enough and less surprising
- Making scope configurable per-provider or per-target — the scope is a property of the feature type, not the deployment target
- Pruning stale entries (out of scope, same as the original change)
- Changing anything about singleton features — their behaviour is unchanged

## Decisions

### 1. `GitignoreScope` enum on `FeatureTrait` (Option C)

**Decision:** Add a `GitignoreScope` enum and a `gitignore_scope(&self) -> GitignoreScope` method with default `GitignoreScope::File`.

```rust
pub(crate) enum GitignoreScope {
    /// Write the exact deployed path into .gitignore.
    File,
    /// Write the parent directory as a glob pattern (parent/*) into .gitignore.
    Directory,
}
```

**Rationale:** An enum is more self-describing than a boolean (`is_directory_scoped: bool`) and gives a clear extension point if a third mode (e.g. `Recursive` for `**`) is ever needed. A boolean would require a rename-and-replace. The method is instance-level (`&self`) for consistency with the rest of the trait, even though the value doesn't depend on the instance — all commands have the same scope regardless of their content.

**Alternative considered:** Derive scope implicitly from `get_file_name()` returning `Some` vs `None`. Rejected because it couples two distinct concerns — target path interpolation and gitignore granularity — through one signal. A future feature might return `Some(filename)` for path interpolation but still want exact-file gitignore entries. Explicit is better.

**Alternative considered:** Boolean `is_directory_scoped() -> bool`. Simpler, but forecloses future variants. Rejected in favour of the enum given negligible extra cost.

### 2. `GitignorePath` enum returned from `deploy_feature` (Approach II)

**Decision:** Introduce a `GitignorePath` enum in `src/utils/gitignore.rs`:

```rust
pub(crate) enum GitignorePath {
    /// Gitignore the exact file.
    File(PathBuf),
    /// Gitignore everything inside this directory (written as "dir/*").
    Directory(PathBuf),
}
```

`deploy_feature` returns `Vec<GitignorePath>` instead of `Vec<PathBuf>`. For each written path, if `item.gitignore_scope()` is `Directory`, push `GitignorePath::Directory(target.parent())` into a `HashSet` (dedup); if `File`, push `GitignorePath::File(target)` directly. The two sets are merged before returning.

**Rationale:** The enum carries the scope intent all the way from the feature through to the formatter, without any ambient context needed. Callers of `write_gitignore` don't need to know feature types — they just pass the tagged paths. `write_gitignore` then converts:
- `File(p)` → `make_workspace_relative(p)` → `".kilo/mcp.json"`
- `Directory(p)` → `make_workspace_relative(p) + "/*"` → `".kilo/commands/*"`

**Alternative considered:** Pass a separate `HashSet<PathBuf>` of "directory parents" alongside `Vec<PathBuf>` of file paths to `write_gitignore`. Rejected — two parallel collections with implicit coupling. The enum makes the relationship explicit.

**Alternative considered:** Return `Vec<String>` pre-formatted patterns from `deploy_feature`. Rejected — formatting concerns (workspace-relative conversion, glob suffix) belong in `gitignore.rs`, not in the deploy pipeline.

### 3. Deduplication happens inside `deploy_feature`

**Decision:** When collecting `Directory` entries, use a `HashSet<PathBuf>` keyed on the parent directory. Multiple command files in `.kilo/commands/` produce one `Directory(.kilo/commands)` entry, not one per file.

**Rationale:** `deploy_feature` iterates items × providers. For 20 commands × 3 providers, all 20 files per provider share the same parent. Deduplicating here keeps `all_paths` in `deploy()` clean — one entry per directory per provider.

### 4. Prompt message reflects entries, not raw file count

**Decision:** The interactive prompt now shows the number of *gitignore entries* that would be added (directories + files), not the number of rendered files. For 20 new commands across 2 providers both deploying for the first time:

```
Add 2 new path(s) to .gitignore? [y/N]
```

rather than `40 new path(s)`. The phrasing "path(s)" remains unchanged — a directory pattern is still a path entry in `.gitignore`.

**Rationale:** The count is used to decide whether to prompt at all (`new_count == 0` → skip). With the glob approach, adding 20 commands to a provider already in `.gitignore` would show `0 new path(s)` and correctly skip the prompt. Counting raw files would incorrectly report `20` even if the directory glob already covers them.

### 5. Commands and Skills are always `Directory`-scoped at the type level

**Decision:** `gitignore_scope` is overridden to return `GitignoreScope::Directory` in both `CommandFeature` and `SkillFeature` unconditionally. There is no per-provider or per-target override.

**Rationale:** Commands and Skills are designed to deploy many files into an owned directory. The target path template always places files inside a dedicated directory (`.claude/commands/`, `.kilo/skills/`, etc.). There is no realistic case where a user would configure multiple commands to write to the same flat file — and even if they did, adding a `dir/*` glob is harmless (it just covers more than necessary).

## Risks / Trade-offs

- **`dir/*` is slightly over-broad**: if a user coincidentally has a non-dotagents file inside `.kilo/commands/`, it would be gitignored. Accepted — per-feature directories are considered owned by dotagents by convention.
- **Existing `.gitignore` entries from before this change**: users who deployed before this change will have specific file paths in their fenced section. The new code does not clean them up (accumulate-only policy). This is consistent with the existing stale-entry policy and is harmless.
- **Deduplication loses per-file granularity in the fence**: you can no longer look at `.gitignore` to see exactly which commands were deployed. Accepted — the cache (`cache.toml`) is the right place for that level of detail, not `.gitignore`.

## Open Questions

*(none)*
