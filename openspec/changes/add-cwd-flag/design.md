## Context

`DeployOptions`, `InitOptions`, and `UndeployOptions` all carry `pub dir: Option<PathBuf>` — a positional `[PATH]` argument. `deploy()` and `undeploy()` call `override_workspace_dir()` early in their handler, resolving relative paths against CWD and validating the path contains `.dotagents/`. The `OnceLock`-based `WORKSPACE_DIR` picks up the overridden value for all subsequent `get_workspace_dir()` calls.

`CommandsAction`, `SkillsAction`, and `ConfigOptions` have **no** path argument. Handlers call `get_workspace_dir()` → `get_application_dir()` directly, which walks up from CWD via the `OnceLock`. There is no mechanism to override the workspace before these handlers fire.

The existing `override_workspace_dir` function in `src/utils/path.rs` validates the path and sets the `OnceLock`, but must be called before any `get_workspace_dir()` call.

## Goals / Non-Goals

**Goals:**
- `commands new/rm/ls`, `skills new/rm/ls/add`, and `config` all accept `--cwd <PATH>`
- Relative paths resolved against CWD
- Validates the path contains `.dotagents/` (same guard as deploy/undeploy)
- Omitted `--cwd` behaves identically to today (walk up from CWD)
- Avoid repetition: single shared struct used across all 7 option structs

**Non-Goals:**
- Adding `--cwd` to `init`, `deploy`, or `undeploy` (they keep positional `[PATH]`)
- Changing `init` to use the OnceLock override pattern
- Adding `--cwd` as an alias for `[PATH]` on deploy/init/undeploy
- Changing any other subcommand (`GenCompletions`)

## Decisions

### 1. Shared `WorkspaceDirArgs` struct via `#[clap(flatten)]`

Introduce a single struct in `src/cli/options.rs`:

```rust
#[derive(Args, Default)]
pub(crate) struct WorkspaceDirArgs {
    #[clap(long = "cwd", value_name = "PATH")]
    pub cwd: Option<PathBuf>,
}
```

Seven option structs flatten it in: `AddCommandOptions`, `RmCommandOptions`, `SubLsOptions`, `SkillsAddOptions`, `AddSkillOptions`, `RmSkillOptions`, `ConfigOptions`.

**Alternatives considered:**
- **Individual `dir` field on each struct**: Repetitive, harder to change in the future (e.g., adding validation or short alias).
- **Hoisting to the parent enum** (`CommandsAction` / `SkillsAction`): Clap doesn't natively support adding args to enum variants without complex workarounds. A flattened struct is the idiomatic clap pattern.
- **Calling it `--dir` instead of `--cwd`**: `--cwd` is more precise — it's a workspace root, not a config directory. The user made this choice explicitly.

### 2. Orchestration in each handler (not in runner)

Each handler function calls a shared helper at the top, before any path resolution:

```rust
// src/utils/path.rs
pub fn resolve_and_override_workspace(cwd: Option<PathBuf>) -> Result<()> {
    let Some(cwd) = cwd else { return Ok(()); };
    let absolute = std::env::current_dir()?.join(cwd);
    override_workspace_dir(absolute)
}
```

Then in each handler:

```rust
fn new_command(opts: AddCommandOptions) -> Result<bool> {
    resolve_and_override_workspace(opts.workspace.cwd)?;
    // ... existing code that calls get_application_dir() ...
}
```

**Alternatives considered:**
- **Orchestration in runner** (`runner.rs`): The runner would need to destructure and extract `cwd` from 7 different option structs, then call `override_workspace_dir` before dispatching. This adds indirection and makes it less obvious where the override happens. Each handler doing it is transparent and follows the `deploy`/`undeploy` pattern.
- **Inline logic in each handler** (no helper): More repetition, harder to update if the resolution or validation changes.

### 3. `SubLsOptions` shared by both `commands ls` and `skills ls`

`SubLsOptions` is already shared between the two `Ls` variants. Flattening `WorkspaceDirArgs` into it means both `commands ls --cwd ...` and `skills ls --cwd ...` get the flag automatically. No additional wiring needed.

### 4. Config handler refactor

Currently `config::handle(target, json, edit)` only takes `ConfigTarget`, `bool`, `bool`. It needs the `cwd` field too. Options:

- Pass `Option<PathBuf>` as a 4th parameter — minimal change, straightforward.
- Pass the full `ConfigOptions` struct — more future-proof if config gets more options.

**Decision**: Pass the full `ConfigOptions` to `config::handle()`. The runner already destructures `Config(opts)`, so just pass `opts` directly and let `handle()` destructure or call `resolve_and_override_workspace` internally.

### 5. Timeline and interaction with `add-providers-command`

The `add-providers-command` change is independent — it doesn't touch the `CommandsAction`, `SkillsAction`, or config CLI structs. No coordination needed.

## Risks / Trade-offs

- **Inconsistency with deploy/init/undeploy**: `--cwd` (flag) vs `[PATH]` (positional). Mitigation: this is intentional — deploy/init/undeploy are primary actions with no positional args of their own, while commands/skills already have positional `name` args. Users are unlikely to confuse the two.
- **`resolve_and_override_workspace` must be called before any `get_workspace_dir()`**: If a handler calls `get_workspace_dir()` before resolving `--cwd`, the `OnceLock` will cache the walk-from-CWD value and the override will be silently ignored. Mitigation: the helper is called at the very top of each handler function, before any other logic.
- **Tests sharing `OnceLock`**: The existing workspace cache means tests that set `override_workspace_dir` in one test can affect others if they run in the same process. This is a pre-existing concern — see test caveats in AGENTS.md. No new risk introduced.
