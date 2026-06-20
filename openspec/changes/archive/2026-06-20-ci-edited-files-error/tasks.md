## 1. Track user-edited count in DeployStats

- [x] 1.1 Add `user_edited: usize` field to `DeployStats` in `src/cli/deploy.rs`
- [x] 1.2 In `process_cache_update`, increment `stats.user_edited` when handling `CacheUpdate::UserEditedSkipped`

## 2. Downgrade per-file warning to debug

- [x] 2.1 Change `warn!("Target file {} was manually edited; skipping")` to `debug!(...)` in `src/templates/renderer.rs:199-202`

## 3. Fail deploy in CI when user-edited files exist

- [x] 3.1 In `finalize_deploy` (`src/cli/ui/deploy.rs`), after printing summary, check `stats.user_edited > 0 && !is_tui_enabled()` and return `Err` with message like `"N file(s) were manually edited. Use --force to override."`
- [x] 3.2 Update `finalize_deploy` signature to accept `force: bool` (already available via `opts.force`) so the check can be gated on `!opts.force`

## 4. Update deploy summary output

- [x] 4.1 Add edited count to `write_summary` in `src/cli/ui/deploy.rs` — show `"N edited"` alongside written/skipped when `user_edited > 0`
- [x] 4.2 Update `deploy_outro` to include edited count for TTY mode

## 5. Unit tests

- [x] 5.1 Add unit test for `write_summary` with `user_edited > 0` in non-TTY mode
- [x] 5.2 Add unit test for `write_summary` with `user_edited > 0` in TTY mode
- [x] 5.3 Add unit test verifying `process_cache_update` increments `user_edited` on `UserEditedSkipped`

## 6. E2E tests

- [x] 6.1 Run tui-devtools discovery pass for deploy with user-edited files in CI mode
- [x] 6.2 Add E2E test: non-TTY deploy with edited files exits 1 and shows count (not paths) in error message
- [x] 6.3 Add E2E test: non-TTY deploy with `--force` exits 0 despite edited files

## 7. Verification

- [x] 7.1 Run `mise check` (fmt + clippy) and fix any issues
- [x] 7.2 Run `mise tests` and fix any failures
