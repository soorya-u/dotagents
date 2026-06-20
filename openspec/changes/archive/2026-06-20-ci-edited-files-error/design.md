## Context

Currently, when `dotagents deploy` detects user-edited target files (on-disk hash differs from cached hash, rendered hash matches cached hash), it:

1. Emits `warn!("Target file {} was manually edited; skipping", path)` per file (`src/templates/renderer.rs:199-202`)
2. Returns `CacheUpdate::UserEditedSkipped { path }` which is counted as `skipped` in `DeployStats` (`src/cli/deploy.rs:153-156`)
3. Deploy completes successfully with exit code 0

In CI (non-TTY), this means the pipeline passes silently even though files were not updated. The individual file paths in warnings clutter the log without providing actionable information in a non-interactive context.

## Goals / Non-Goals

**Goals:**

- In CI/non-TTY mode, fail deploy (exit 1) when user-edited files are detected without `--force`
- Show a concise summary error with count of edited files, not individual paths
- Suggest `--force` as the override mechanism in the error message
- Preserve existing TTY behavior (per-file warnings, exit 0)

**Non-Goals:**

- Changing the behavior of `--force` (it already overrides user-edit detection)
- Changing merge-skip behavior (different concern)
- Adding a new CLI flag for this behavior (CI mode detection is sufficient)

## Decisions

### Decision 1: Track user-edited count in DeployStats

Add a `user_edited: usize` field to `DeployStats`. In `process_cache_update`, when `CacheUpdate::UserEditedSkipped` is encountered, increment `user_edited` in addition to `skipped`.

**Rationale**: The count is already implicitly tracked via `skipped`, but conflating "unchanged" skips with "user-edited" skips makes it impossible to distinguish at finalization time. A dedicated counter is the minimal change.

**Alternative considered**: Collecting edited paths in a `Vec<PathBuf>` on `DeployStats`. Rejected because the proposal explicitly says not to list files — we only need the count, and storing paths would tempt future display.

### Decision 2: Check at finalization, not at detection time

The error check happens in `finalize_deploy` (or just before it), not in the renderer or `process_cache_update`.

**Rationale**: The deploy pipeline uses `rayon::par_iter` — returning an error mid-iteration would short-circuit other work items. We want to deploy everything we can (including non-edited files) and then fail at the end with the total count. This is consistent with the existing "collect stats, then summarize" pattern.

### Decision 3: Gate on non-TTY, not on `--ci` flag

Use `!is_tui_enabled()` (which checks both `--ci` flag and TTY state) rather than checking only the `--ci` flag.

**Rationale**: Non-TTY environments (piped output, CI runners) should all get the concise error behavior. The `is_tui_enabled()` function already encapsulates this logic and is used throughout the codebase for TTY-vs-non-TTY branching.

### Decision 4: Downgrade per-file warn to debug in all modes

Change `warn!("Target file {} was manually edited; skipping")` to `debug!(...)` regardless of TTY mode. The summary message replaces it in both modes.

**Rationale**: In TTY mode, the deploy summary already shows skip counts. Adding a "N files edited" line to the summary is cleaner than per-file warnings. In non-TTY mode, the error summary replaces the warnings entirely. This keeps the change simple — one code path for the per-file log level.

## Risks / Trade-offs

- **TTY users lose per-file visibility** → Mitigation: The summary message includes the count, and `debug!` output is available with `-v`. Users who need per-file detail can add the flag.
- **CI pipelines that previously passed will start failing** → This is the intended behavior — those pipelines were silently incomplete. The error message clearly indicates `--force` as the fix.
