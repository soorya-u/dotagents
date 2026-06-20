## Why

When `dotagents deploy` runs in CI and encounters user-edited target files (files whose on-disk content diverges from the cached hash), it currently emits a per-file `warn!` listing each file path and exits 0 (success). In CI this is unhelpful: the individual file paths clutter the log, and the zero exit code means the CI pipeline silently passes even though deploy was incomplete. CI should fail fast with a concise summary instead.

## What Changes

- In CI mode (non-TTY), when user-edited files are detected and `--force` is not passed, deploy SHALL exit with status 1 instead of 0.
- The per-file `warn!("Target file {} was manually edited; skipping")` SHALL be replaced with a debug-level log per file and a single summary error at the end: e.g. `"3 file(s) were manually edited. Use --force to override."`.
- In TTY mode, behavior remains unchanged: per-file warnings are shown and deploy exits 0 (the user can see and act on the warnings interactively).

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `deploy-output-cache`: The "Detect and preserve user-edited target files" requirement changes — in CI mode, user-edited files cause a non-zero exit and a summary error instead of per-file warnings with a zero exit.
- `ci-mode`: CI mode gains an additional behavior — deploy exits 1 when user-edited files are detected without `--force`.

## Impact

- `src/templates/renderer.rs`: Per-file `warn!` for user-edited files downgraded to `debug!`.
- `src/cli/deploy.rs`: `process_cache_update` needs to track user-edited skip count; `finalize_deploy` (or the deploy entry point) needs to check the count in CI mode and return an error.
- `src/cli/ui/deploy.rs`: Summary output needs a new "edited files" error line.
- E2E tests: new CI-mode deploy test asserting exit code 1 and summary message when edited files exist.
