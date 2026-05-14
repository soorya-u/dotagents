## 1. CLI struct changes

- [x] 1.1 Create `WorkspaceDirArgs` struct in `src/cli/options.rs` with `#[clap(long = "cwd", value_name = "PATH")] pub cwd: Option<PathBuf>`, derive `Args` + `Default`
- [x] 1.2 Flatten `WorkspaceDirArgs` into `AddCommandOptions` (`commands new`)
- [x] 1.3 Flatten `WorkspaceDirArgs` into `RmCommandOptions` (`commands rm`)
- [x] 1.4 Flatten `WorkspaceDirArgs` into `SubLsOptions` (`commands ls` + `skills ls`)
- [x] 1.5 Flatten `WorkspaceDirArgs` into `AddSkillOptions` (`skills new`)
- [x] 1.6 Flatten `WorkspaceDirArgs` into `RmSkillOptions` (`skills rm`)
- [x] 1.7 Flatten `WorkspaceDirArgs` into `SkillsAddOptions` (`skills add`)
- [x] 1.8 Flatten `WorkspaceDirArgs` into `ConfigOptions`

## 2. Workspace resolution helper

- [x] 2.1 Add `resolve_and_override_workspace(cwd: Option<PathBuf>) -> Result<()>` to `src/utils/path.rs` that resolves relative paths against CWD and calls `override_workspace_dir`
- [x] 2.2 Export the helper from `src/utils/path.rs` module

## 3. Wire up command handlers

- [x] 3.1 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `new_command()` in `src/cli/commands.rs`
- [x] 3.2 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `rm_command()` in `src/cli/commands.rs`
- [x] 3.3 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `ls_commands()` in `src/cli/commands.rs`
- [x] 3.4 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `new_skill()` in `src/cli/skills.rs`
- [x] 3.5 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `rm_skill()` in `src/cli/skills.rs`
- [x] 3.6 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `ls_skills()` in `src/cli/skills.rs`
- [x] 3.7 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `add()` in `src/cli/skills.rs`

## 4. Wire up config handler

- [x] 4.1 Change `config::handle()` signature to accept `ConfigOptions` instead of individual fields (`ConfigTarget`, `bool`, `bool`)
- [x] 4.2 Call `resolve_and_override_workspace(opts.workspace.cwd)` at the top of `config::handle()` in `src/cli/config.rs`
- [x] 4.3 Update `runner.rs` to pass `opts` (the full `ConfigOptions`) instead of destructuring fields

## 5. Unit tests

- [x] 5.1 Add unit test for `resolve_and_override_workspace` with relative path (verify it resolves against CWD)
- [x] 5.2 Add unit test for `resolve_and_override_workspace` with absolute path
- [x] 5.3 Add unit test for `resolve_and_override_workspace` with `None` (returns Ok without touching OnceLock)
- [x] 5.4 Add unit test for `resolve_and_override_workspace` with path missing `.dotagents/` (returns Err)

## 6. E2E tests

- [x] 6.1 Add e2e test: `commands ls --cwd <existing-workspace>` reads from that workspace
- [x] 6.2 Add e2e test: `commands ls --cwd <nonexistent>` exits non-zero with clear error
- [x] 6.3 Add e2e test: `commands new --cwd ...` creates file in that workspace
- [x] 6.4 Add e2e test: `commands rm --cwd ...` removes from that workspace and runs cleanup
- [x] 6.5 Add e2e test: `skills ls --cwd <existing>` reads from that workspace
- [x] 6.6 Add e2e test: `skills new --cwd ...` creates skill in that workspace
- [x] 6.7 Add e2e test: `skills rm --cwd ...` removes from that workspace and runs cleanup
- [x] 6.8 Add e2e test: `config --cwd <existing>` reads config from that workspace
- [x] 6.9 Add e2e test: `--cwd` omitted behaves identically to before (no regression)
- [x] 6.10 Add e2e test: relative `--cwd` path resolved correctly

## 7. Verification

- [x] 7.1 Run `mise check` (format + lint) and fix any issues
- [x] 7.2 Run `mise tests:unit` and ensure all unit tests pass
- [x] 7.3 Run `mise tests:e2e` and ensure all e2e tests pass
- [x] 7.4 Run `mise tests` (full suite) to confirm no regressions
