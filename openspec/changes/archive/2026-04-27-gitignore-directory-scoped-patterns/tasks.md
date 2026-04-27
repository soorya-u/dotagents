## 1. Feature Trait — `GitignoreScope`

- [x] 1.1 Add `GitignoreScope` enum to `src/schema/features/traits.rs` with two variants: `File` and `Directory`
- [x] 1.2 Add `gitignore_scope(&self) -> GitignoreScope` method to `FeatureTrait` with default impl returning `GitignoreScope::File`
- [x] 1.3 Override `gitignore_scope` in `src/schema/features/command.rs` to return `GitignoreScope::Directory`
- [x] 1.4 Override `gitignore_scope` in `src/schema/features/skill.rs` to return `GitignoreScope::Directory`
- [x] 1.5 Write unit tests in `command.rs` and `skill.rs` confirming `gitignore_scope()` returns `Directory`
- [x] 1.6 Write unit tests in `instruction.rs` and `mcp.rs` (or `traits.rs`) confirming the default returns `File`

## 2. Gitignore Utility — `GitignorePath` Enum

- [x] 2.1 Add `GitignorePath` enum to `src/utils/gitignore.rs` with variants `File(PathBuf)` and `Directory(PathBuf)`
- [x] 2.2 Update `write_gitignore` signature to accept `&[GitignorePath]` instead of `&[PathBuf]`
- [x] 2.3 Update the path-to-string conversion inside `write_gitignore`:
  - `GitignorePath::File(p)` → `make_workspace_relative(p)` (e.g. `.kilo/mcp.json`)
  - `GitignorePath::Directory(p)` → `make_workspace_relative(p).map(|s| format!("{s}/*"))` (e.g. `.kilo/commands/*`)
- [x] 2.4 Write unit tests for `write_gitignore` with `Directory` entries — confirm `/*` suffix is written, duplicates are collapsed
- [x] 2.5 Write unit test confirming `File` and `Directory` entries can coexist in one `write_gitignore` call

## 3. Deploy Pipeline — Collect `GitignorePath`

- [x] 3.1 Change `deploy_feature` return type from `Vec<PathBuf>` to `Vec<GitignorePath>` in `src/cli/deploy.rs`
- [x] 3.2 Inside `deploy_feature`, after a file is written, branch on `item.gitignore_scope()`:
  - `GitignoreScope::File` → push `GitignorePath::File(target_path)` to accumulator
  - `GitignoreScope::Directory` → collect unique parent directories in a local `HashSet<PathBuf>`, then extend accumulator with `GitignorePath::Directory(parent)` entries at the end of each provider iteration
- [x] 3.3 Update `all_paths` in `deploy()` to be `Vec<GitignorePath>` and extend it with the results of each `deploy_feature` call
- [x] 3.4 Update the new-count calculation (used for the interactive prompt) to compare gitignore patterns — `Directory` entries become `"dir/*"` strings, `File` entries become exact paths — against the existing fenced section, so previously-written directory globs are not double-counted

## 4. Prompt Message

- [x] 4.1 Verify the existing prompt wording `"Add {n} deployed path(s) to .gitignore? [y/N]"` still makes sense for directory entries (it does — a `dir/*` pattern is a path entry). No wording change required unless `n` calculation was previously based on raw file count rather than entry count; fix the count if so (see task 3.4)

## 5. Verification

- [x] 5.1 Run `mise check` — `cargo fmt` and `cargo clippy` exit 0
- [x] 5.2 Run `mise test-all` — all unit, integration, and e2e tests pass
- [ ] 5.3 Deploy a workspace with 3+ commands and 3+ skills to two providers; verify `.gitignore` fenced section contains `provider/commands/*` and `provider/skills/*` entries (not individual file paths)
- [ ] 5.4 Deploy the same workspace again; verify no duplicate entries are added
- [ ] 5.5 Deploy a workspace with only MCP and Instructions configured; verify exact file paths are still written (not globs)
- [ ] 5.6 Verify that a pre-existing fenced section with individual command file paths (written by an older version) is preserved without duplication when the new glob pattern is added
