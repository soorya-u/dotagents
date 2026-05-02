## Why

`dotagents deploy` gives no feedback when it finishes, leaving users uncertain whether anything happened. There is also no way to undo a deploy — once files are written and `.gitignore` is updated, there is no supported path to reverse the operation. Additionally, the `--no-cache` flag currently suppresses cache writes entirely (a side effect of its original implementation), and the gitignore managed section uses directory globs (`commands/*`) that are too coarse to remove surgically during undeploy.

## What Changes

- **Deploy outro**: After a successful deploy, print a rich TTY summary ("3 written, 2 skipped") so users know what happened. No output in non-TTY (CI) mode.
- **Fix `--no-cache` semantics**: `--no-cache` now means "skip hash comparison, always re-render" — cache.toml is still written after every deploy. Previously it suppressed both reading and writing, which meant subsequent deploys lost the ability to detect unchanged files.
- **`dotagents undeploy` command**: New subcommand that reads `cache.toml`, deletes every file listed there, removes the dotagents-managed fence from `.gitignore`, prunes empty parent directories, and clears the cache.
- **Gitignore individual paths**: The managed `.gitignore` section now records individual file paths (`.claude/commands/hello.md`) instead of directory globs (`.claude/commands/*`). This makes undeploy able to remove exactly the files it deployed without touching anything else.

## Capabilities

### New Capabilities

- `deploy-outro`: TTY-only deploy completion summary showing written/skipped counts per feature.
- `undeploy-command`: New CLI subcommand that reverses a deploy using cache.toml as the source of truth.

### Modified Capabilities

- `deploy-output-cache`: `--no-cache` changes meaning — cache is always written; the flag only disables hash-comparison skipping.
- `deploy-gitignore-update`: Managed section records individual file paths instead of directory globs.

## Impact

- `src/cli/deploy.rs` — `deploy_feature()` return type changes to `DeployStats`; outro printed after cache save.
- `src/cli/options.rs` — new `Undeploy(UndeployOptions)` variant; `DeployOptions` `--no-cache` semantics change.
- `src/cli/runner.rs` — new `Action::Undeploy` dispatch arm.
- `src/cli/undeploy.rs` — new file.
- `src/cli/ui/deploy.rs` — new `print_deploy_summary()` helper.
- `src/cli/ui/undeploy.rs` — new file with TTY confirmation prompt.
- `src/schema/config/cache.rs` — new `iter_entries()` method; `CacheConfig` always written on deploy.
- `src/utils/fs.rs` — new `delete_file()` helper.
- `src/utils/gitignore.rs` — new `clear_gitignore_fence()` helper; `GitignoreScope::Directory` variant replaced by per-file entries.
- All feature impls (`CommandFeature`, `SkillFeature`) — `gitignore_scope()` changes from `Directory` to `File`.
