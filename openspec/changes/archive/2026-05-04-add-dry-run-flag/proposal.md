## Why

Users want to preview what `deploy` or `undeploy` would do before committing to it — seeing which files would be written, modified, or deleted — without any side effects. This is a standard safety affordance that `dotter` (the inspiration for dotagents) already provides.

## What Changes

- Add `--dry-run` flag to `deploy`: renders templates, resolves providers, and prints the list of files that would be written (new or modified) — without writing files, saving cache, or updating `.gitignore`.
- Add `--dry-run` flag to `undeploy`: reads `cache.toml` and prints the list of files that would be deleted (distinguishing edited files) — without deleting files, clearing cache, or removing the `.gitignore` fence.
- No `--dry-run` for `init` (scaffolding a config dir has no meaningful preview value).
- `--dry-run` is a flag-only feature — no TUI option is presented; interactive prompts are suppressed when the flag is set.
- Template validation errors and config errors still surface and cause exit code 1, same as a real run.
- The flag respects all existing flags (`--offline`, `--force`, `--no-cache`, etc.).

## Capabilities

### New Capabilities

- `deploy-dry-run`: `--dry-run` flag on `deploy` — renders templates, compares against on-disk state, prints `[+]` (new) / `[~]` (modified) per target path, exits 0/1 without side effects.
- `undeploy-dry-run`: `--dry-run` flag on `undeploy` — reads cache, checks on-disk hashes, prints `[-]` (would delete) / `[x]` (edited — would prompt) per path, exits 0/1 without side effects.

### Modified Capabilities

## Impact

- `src/cli/options.rs` — add `dry_run: bool` to `DeployOptions` and `UndeployOptions`
- `src/cli/deploy.rs` — branch on `dry_run`: skip `write_file`, cache save, and gitignore update; collect and print dry-run results
- `src/cli/undeploy.rs` — branch on `dry_run`: skip `delete_file`, cache clear, and gitignore removal; collect and print dry-run results
- `src/cli/ui/dry_run.rs` (new) — `print_dry_run_deploy_summary()` and `print_dry_run_undeploy_summary()`
- `tests/e2e/` — new e2e tests for both `deploy --dry-run` and `undeploy --dry-run` paths
