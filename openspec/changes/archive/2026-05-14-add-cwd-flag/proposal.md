## Why

`commands`, `skills`, and `config` subcommands always resolve the workspace by walking up from CWD — there's no way to target a different workspace without `cd`-ing first. Issue #57 added positional `[PATH]` to `init`, `deploy`, and `undeploy`, but deliberately deferred `commands` and `skills` (and `config` was overlooked). Users need a single consistent way to point these subcommands at any workspace.

## What Changes

- Add a shared `--cwd <PATH>` flag (via `#[clap(flatten)]` on a `WorkspaceDirArgs` struct) to all `commands` subcommands (`new`, `rm`, `ls`), all `skills` subcommands (`new`, `rm`, `ls`, `add`), and `config`.
- Relative paths are resolved against the current working directory before being validated.
- Validates that the resolved path contains `.dotagents/` (same as `deploy`/`undeploy`), failing with a clear error if not.
- When `--cwd` is omitted, behaviour is unchanged — workspace is resolved by walking up from CWD as today.
- No change to `init`, `deploy`, or `undeploy` — they keep their positional `[PATH]` argument.

## Capabilities

### New Capabilities

_None — this extends an existing capability._

### Modified Capabilities

- `workspace-path-arg`: extends workspace path specification to `commands`, `skills`, and `config` subcommands via a `--cwd` flag (named, not positional, because these subcommands already take their own positional arguments).

## Impact

- **CLI structs** (`src/cli/options.rs`): Six sub-options structs plus one config struct gain a `#[clap(flatten)] workspace: WorkspaceDirArgs` field; a new shared `WorkspaceDirArgs` struct is introduced.
- **Handler functions** (`src/cli/commands.rs`, `src/cli/skills.rs`, `src/cli/config.rs`): Each handler calls `override_workspace_dir()` before any path resolution, following the existing `deploy`/`undeploy` pattern.
- **`rm` cleanup** (`rm-cleanup` spec): The existing `get_workspace_dir()` call in `rm` handlers automatically picks up the overridden value — no additional changes needed.
- **E2E tests**: New test scenarios for `--cwd` on each affected subcommand.
