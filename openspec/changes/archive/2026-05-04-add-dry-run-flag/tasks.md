## 1. CLI Options

- [x] 1.1 Add `dry_run: bool` field to `DeployOptions` in `src/cli/options.rs` with `--dry-run` long flag and doc comment
- [x] 1.2 Add `dry_run: bool` field to `UndeployOptions` in `src/cli/options.rs` with `--dry-run` long flag and doc comment

## 2. Dry-Run UI Module

- [x] 2.1 Create `src/cli/ui/dry_run.rs` with `DryRunDeployEntry { path: PathBuf, status: DeployDryRunStatus }` and `DeployDryRunStatus` enum (`New`, `Modified`)
- [x] 2.2 Add `DryRunUndeployEntry { path: PathBuf, status: UndeployDryRunStatus }` and `UndeployDryRunStatus` enum (`WouldDelete`, `Edited`) to `src/cli/ui/dry_run.rs`
- [x] 2.3 Implement `print_dry_run_deploy_summary(entries: &[DryRunDeployEntry])` in `src/cli/ui/dry_run.rs` — prints header, `[+]`/`[~]` lines, and `N files would be affected` footer
- [x] 2.4 Implement `print_dry_run_undeploy_summary(entries: &[DryRunUndeployEntry])` in `src/cli/ui/dry_run.rs` — prints header, `[-]`/`[x]` lines (with `(edited)` suffix), and `N files would be affected` footer
- [x] 2.5 Register `dry_run` module in `src/cli/ui/mod.rs`

## 3. Deploy Dry-Run Logic

- [x] 3.1 In `src/cli/deploy.rs`, after template rendering and before `write_file`, check `opts.dry_run`; if set, compare rendered content to on-disk file to determine `New` vs `Modified` vs unchanged, and collect into `Vec<DryRunDeployEntry>`
- [x] 3.2 Skip `write_file` call when `opts.dry_run` is set
- [x] 3.3 Skip cache save (`cache.toml` write) when `opts.dry_run` is set
- [x] 3.4 Skip gitignore update (and its prompt) when `opts.dry_run` is set
- [x] 3.5 After collecting all entries, call `print_dry_run_deploy_summary` and return — do not print the normal deploy outro

## 4. Undeploy Dry-Run Logic

- [x] 4.1 In `src/cli/undeploy.rs`, skip the bulk-confirmation prompt when `opts.dry_run` is set
- [x] 4.2 For each cache entry, when `opts.dry_run` is set, compute on-disk hash and compare to cached hash to determine `WouldDelete` vs `Edited`; collect into `Vec<DryRunUndeployEntry>` — do not call `delete_file`
- [x] 4.3 Skip cache clear when `opts.dry_run` is set
- [x] 4.4 Skip gitignore fence removal when `opts.dry_run` is set
- [x] 4.5 After collecting all entries, call `print_dry_run_undeploy_summary` and return

## 5. Unit Tests

- [x] 5.1 Add unit tests in `src/cli/ui/dry_run.rs` for `print_dry_run_deploy_summary` — empty list, all-new, all-modified, mixed
- [x] 5.2 Add unit tests in `src/cli/ui/dry_run.rs` for `print_dry_run_undeploy_summary` — empty list, all-delete, all-edited, mixed

## 6. E2E Tests

- [x] 6.1 Run tui-devtools discovery pass over `deploy --dry-run` to record exact terminal output (symbols, spacing, header/footer text) — N/A: dry-run adds no new interactive prompts; all tests use --offline to bypass the offline prompt
- [x] 6.2 Run tui-devtools discovery pass over `undeploy --dry-run` to record exact terminal output — N/A: dry-run suppresses the confirmation prompt; no new TUI interactions
- [x] 6.3 Add `deploy.dry-run.test.ts` in `tests/e2e/` — assert exit code 0, `[+]` lines for new files, no files written to disk, no cache change
- [x] 6.4 Add scenario in `deploy.dry-run.test.ts` — template error causes exit code 1 and error on stderr
- [x] 6.5 Add `undeploy.dry-run.test.ts` in `tests/e2e/` — assert exit code 0, `[-]` lines for unmodified files, `[x]` for edited, no files deleted, cache unchanged
- [x] 6.6 Add scenario in `undeploy.dry-run.test.ts` — empty cache exits 0 with `0 files would be affected`

## 7. Verification

- [x] 7.1 Run `mise check` (cargo fmt + clippy) — fix all warnings and format issues
- [x] 7.2 Run `mise tests` (unit + integration + e2e) — all suites must pass
