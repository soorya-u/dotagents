## Context

The current `maybe_prompt_deploy(deploy_flag: bool)` function in both `commands.rs` and `skills.rs` prompts in TTY and skips in CI unless `--deploy` is explicitly passed. This is backwards from the expected default: CI pipelines should deploy automatically; human users should be prompted.

## Goals / Non-Goals

**Goals:**
- Auto-deploy in CI after `new`/`rm` unless `--no-deploy` is set
- Prompt in TTY after `new`/`rm` unless `--no-deploy` is set
- `--no-deploy` as an explicit escape hatch for both environments
- Identical behavior for `new` and `rm` subcommands

**Non-Goals:**
- Changing the deploy implementation itself — only the trigger logic changes
- Adding `--no-deploy` to `skills add` (that command invokes an external package manager, not the local deploy pipeline)

## Decisions

1. **Replace `--deploy` with `--no-deploy`**: Inverting the flag keeps the API surface the same size while making the default behavior correct. Alternative: make `--deploy` a tri-state (yes/no/auto) — rejected as overly complex.

2. **Single `maybe_prompt_deploy(no_deploy: bool)` signature**: The function checks `no_deploy` first (return early), then `is_tui_enabled()` to branch between auto-deploy and prompt. This keeps all deploy-trigger logic in one place.

3. **Identical function in both `commands.rs` and `skills.rs`**: The two files currently have duplicate implementations. They remain duplicated (not extracted) to keep the change minimal — a future refactor can consolidate them.

## Risks / Trade-offs

- **Breaking change for scripts using `--deploy`**: Any automation using `--deploy` will break silently (the flag is removed). Mitigation: this is a pre-1.0 CLI; document in release notes. `--no-deploy` is an explicit opt-out that is easy to discover via `--help`.
- **CI auto-deploy may be unexpected for first-time users**: Mitigation: the `--no-deploy` flag appears in `--help` for all affected subcommands.
