## Why

Users must `cd` into a project directory before running `dotagents init`, `deploy`, or `undeploy`, which makes scripting and multi-project workflows awkward. An optional positional path argument lets users target any workspace from any working directory.

## What Changes

- `dotagents init [PATH]` — scaffolds `.dotagents/` inside `PATH` instead of CWD; creates `PATH` (including parents) if it does not exist.
- `dotagents deploy [PATH]` — treats `PATH` as the workspace root (the directory that contains `.dotagents/`) instead of walking up from CWD.
- `dotagents undeploy [PATH]` — same as deploy; uses `PATH` as workspace root.
- `PATH` is an optional positional first argument on all three subcommands; omitting it preserves the current CWD-based behaviour.
- Both absolute and relative paths are accepted; relative paths are resolved against CWD at runtime.
- The interactive init wizard continues to run when `PATH` is provided (it is not a headless flag).

## Capabilities

### New Capabilities

- `workspace-path-arg`: Optional positional `[PATH]` argument on `init`, `deploy`, and `undeploy` that overrides the default CWD-based workspace resolution.

### Modified Capabilities

<!-- No existing spec-level requirements are changing. -->

## Impact

- `src/cli/options.rs` — add `pub dir: Option<PathBuf>` (positional) to `InitOptions`, `DeployOptions`, `UndeployOptions`.
- `src/cli/init.rs` — resolve `main_dir` from `opts.dir` instead of hardcoded `ROOT_DIR`.
- `src/cli/deploy.rs` — pre-populate `WORKSPACE_DIR` OnceLock before config loading.
- `src/cli/undeploy.rs` — same as deploy.
- `src/utils/path.rs` — new `override_workspace_dir(PathBuf) -> Result<()>` helper.
- No new dependencies required.
- No breaking changes; existing callers with no `PATH` argument are unaffected.
