## Why

When `dotagents deploy` writes provider config files (e.g. `CLAUDE.md`, `AGENTS.md`, `.claude/commands/*.md`), those files are immediately tracked by git — causing noise on every deploy even though they are generated build artifacts whose source of truth is `.dotagents/`. Since `dotagents deploy` is the standard repo setup step, the tool itself should handle gitignoring its own outputs rather than leaving it to the user.

## What Changes

- `dotagents deploy` gains an optional final step: update the workspace root `.gitignore` with a dotagents-managed fenced section listing all rendered target paths
- New `--gitignore` flag on `deploy` — always update `.gitignore` without prompting
- New `--no-gitignore` flag on `deploy` — always skip, never touch `.gitignore`
- No flag given — prompt the user interactively after deploy: `Add deployed paths to .gitignore? [y/N]`
- If `.gitignore` does not exist at the workspace root, it is created
- Target paths are written as specific workspace-relative paths (e.g. `.claude/commands/hello.md`), not wildcards — avoids collateral damage on shared directories like `.github/` that contain both dotagents outputs and user files (e.g. workflows)
- A fenced section (`# BEGIN dotagents` / `# END dotagents`) isolates dotagents entries from user entries; user content is never modified
- Stale entries (targets removed from config) are accumulated harmlessly — never removed

## Capabilities

### New Capabilities

- `deploy-gitignore-update`: Defines the behaviour for collecting rendered target paths, deriving workspace-relative gitignore patterns, and writing/updating the fenced section in the workspace root `.gitignore`. Covers the three operating modes (`--gitignore`, `--no-gitignore`, interactive prompt) and the accumulate-only stale-entry policy.

### Modified Capabilities

*(none)*

## Impact

- `src/cli/deploy.rs` — add gitignore update step as the final action after all files are written
- `src/cli/options.rs` — add `--gitignore` and `--no-gitignore` flags to the `Deploy` subcommand
- `src/utils/fs.rs` (or new `src/utils/gitignore.rs`) — add helpers: read existing `.gitignore`, parse fenced section, append new paths, write back
- `src/utils/` — add interactive prompt helper (or reuse `crossterm` already in dependencies)
- No new external dependencies required
