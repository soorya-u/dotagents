## Why

The original gitignore implementation (change `2026-04-25-gitignore-deployed-files`) chose specific file-level paths for all features, with the rationale that directories like `.github/` are shared and wildcarding them would hide unrelated user files. That decision was correct for singleton-file features (MCP, Instructions). But for directory-scoped features — Commands and Skills — each feature *owns* its entire target directory exclusively. A repo with 20 commands deployed to 3 providers produces 60 individual entries in the fenced section. A repo with 20 commands and 20 skills across 3 providers produces 120 entries. This is noisy, hard to read, and grows without bound as users add more commands or skills.

The fix: features that deploy many files into an owned directory should contribute a single glob pattern (e.g. `.kilo/commands/*`) rather than one entry per file.

## What Changes

- `FeatureTrait` gains a `gitignore_scope(&self) -> GitignoreScope` method (default: `GitignoreScope::File`)
- A new `GitignoreScope` enum (`File` | `Directory`) lives in `src/schema/features/traits.rs` alongside the trait
- `CommandFeature` and `SkillFeature` override `gitignore_scope` to return `GitignoreScope::Directory`
- `InstructionFeature` and `McpFeature` keep the default (`GitignoreScope::File`) — they produce single files
- `deploy_feature` returns `Vec<GitignorePath>` (a new enum: `File(PathBuf)` | `Directory(PathBuf)`) instead of `Vec<PathBuf>`, using `gitignore_scope` on each rendered item to decide which variant to push. Parent directories are deduplicated so one directory produces one entry regardless of how many files were written into it
- `write_gitignore` is updated to accept `&[GitignorePath]`; `File` paths convert to exact workspace-relative strings, `Directory` paths convert to `"dir/*"` glob patterns
- The new-path count used in the interactive prompt reflects directory entries (1 per directory) rather than individual files

## Capabilities

### New Capabilities

*(none — this is a refinement of an existing capability)*

### Modified Capabilities

- `deploy-gitignore-update`: Extends the existing gitignore update behaviour. Directory-scoped features now contribute a single `dir/*` glob pattern per unique parent directory per provider, rather than one entry per rendered file.

## Impact

- `src/schema/features/traits.rs` — add `GitignoreScope` enum and `gitignore_scope` method on `FeatureTrait`
- `src/schema/features/command.rs` — override `gitignore_scope` → `GitignoreScope::Directory`
- `src/schema/features/skill.rs` — override `gitignore_scope` → `GitignoreScope::Directory`
- `src/cli/deploy.rs` — `deploy_feature` returns `Vec<GitignorePath>`; aggregate and pass to `write_gitignore`
- `src/utils/gitignore.rs` — add `GitignorePath` enum; update `write_gitignore` to convert variants to patterns; update new-count calculation for prompt
