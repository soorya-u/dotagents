## 1. Deploy intro + outro

- [x] 1.1 In `src/cli/runner.rs` (or wherever the top-level deploy dispatch lives), add `if is_tty() { intro("dotagents").ok(); }` before calling `deploy(opts)`
- [x] 1.2 In `src/cli/deploy.rs`, add `if is_tty() { outro("Done.").ok(); }` at the early-return after `print_deploy_summary` when `stats.paths.is_empty() || opts.no_gitignore` (line ~229)
- [x] 1.3 Add `if is_tty() { outro("Done.").ok(); }` at the early-return when `new_count == 0` (line ~251)
- [x] 1.4 Add `if is_tty() { outro("Done.").ok(); }` at the final return after the gitignore write (line ~261)
- [x] 1.5 Add `cliclack::intro` and `cliclack::outro` to the import list in `deploy.rs`

## 2. Fix intro text for skills and commands subcommands

- [x] 2.1 In `src/cli/skills.rs`, change `intro("dotagents skills new")` to `intro("New skill")` and confirm it is gated on `use_interactive`
- [x] 2.2 In `src/cli/skills.rs`, change `intro("dotagents skills rm")` to `intro("Remove skill")` and gate the call on `is_tty()`
- [x] 2.3 In `src/cli/commands.rs`, change `intro("dotagents commands new")` (or equivalent) to `intro("New command")` and confirm it is gated on its interactive guard
- [x] 2.4 In `src/cli/commands.rs`, change `intro("dotagents commands rm")` (or equivalent) to `intro("Remove command")` and gate the call on `is_tty()`

## 3. Gitignore fence format

- [x] 3.1 In `src/utils/gitignore.rs`, change `FENCE_START` to `"#region dotagents"` (line 23)
- [x] 3.2 In `src/utils/gitignore.rs`, change `FENCE_END` to `"#endregion dotagents"` (line 24)

## 4. Update tests

- [x] 4.1 In `tests/integration/gitignore.rs`, replace all occurrences of `"# BEGIN dotagents managed - do not edit manually"` with `"#region dotagents"` and `"# END dotagents managed"` with `"#endregion dotagents"`
- [x] 4.2 In `tests/integration/undeploy.rs`, same string replacements
- [x] 4.3 In `tests/e2e/undeploy.test.ts`, same string replacements

## 5. Verification

- [x] 5.1 Run `tui-devtools` against `dotagents deploy` and verify intro appears before offline prompt and outro appears after gitignore step
- [x] 5.2 Run `tui-devtools` against `dotagents skills new` and `dotagents skills rm` and verify correct intro text
- [x] 5.3 Run `mise check && mise tests` and fix any failures
