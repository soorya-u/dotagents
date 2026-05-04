## 1. CLI Options

- [x] 1.1 Add `pub dir: Option<PathBuf>` positional field to `InitOptions` in `src/cli/options.rs`
- [x] 1.2 Add `pub dir: Option<PathBuf>` positional field to `DeployOptions` in `src/cli/options.rs`
- [x] 1.3 Add `pub dir: Option<PathBuf>` positional field to `UndeployOptions` in `src/cli/options.rs`

## 2. Workspace Override Helper

- [x] 2.1 Add `pub fn override_workspace_dir(path: PathBuf) -> Result<()>` to `src/utils/path.rs` — validates `path/.dotagents/` exists, then pre-populates `WORKSPACE_DIR` OnceLock
- [x] 2.2 Add unit tests for `override_workspace_dir` in `src/utils/path.rs` (valid path, missing `.dotagents`, already-set lock)

## 3. init Command

- [x] 3.1 In `src/cli/init.rs`, resolve the workspace root: `std::env::current_dir().join(opts.dir.unwrap_or("."))`
- [x] 3.2 Replace `let main_dir = Path::new(ROOT_DIR)` with `let main_dir = workspace.join(ROOT_DIR)`
- [x] 3.3 Add `fs::create_dir_all(&workspace)` before the existing `fs::create_dir(&main_dir)` call
- [x] 3.4 Add unit tests for `initialize_agents_dir` with an explicit `dir` path (new dir created, existing dir, missing parents)

## 4. deploy Command

- [x] 4.1 In `src/cli/deploy.rs`, at the top of `deploy()`, resolve and call `override_workspace_dir` when `opts.dir` is `Some`
- [x] 4.2 Resolve relative path against CWD before passing to `override_workspace_dir`

## 5. undeploy Command

- [x] 5.1 In `src/cli/undeploy.rs`, at the top of `undeploy()`, resolve and call `override_workspace_dir` when `opts.dir` is `Some`
- [x] 5.2 Resolve relative path against CWD before passing to `override_workspace_dir`

## 6. tui-devtools Discovery

- [x] 6.1 Run `tui-devtools` and drive `dotagents init <PATH>` through the interactive wizard — record exact terminal output (symbols, prompts, outro)
- [x] 6.2 Run `dotagents deploy <PATH>` and `dotagents undeploy <PATH>` with a temp workspace — verify output unchanged from no-arg invocations

## 7. E2E Tests

- [x] 7.1 Add e2e test: `init` with explicit absolute PATH creates `.dotagents/` there
- [x] 7.2 Add e2e test: `init` with relative PATH resolves correctly against CWD
- [x] 7.3 Add e2e test: `init` with non-existent PATH creates the directory and scaffolds inside it
- [x] 7.4 Add e2e test: `init <PATH>` in a TTY still launches the interactive wizard
- [x] 7.5 Add e2e test: `deploy` with explicit PATH deploys to the correct workspace
- [x] 7.6 Add e2e test: `deploy` with a PATH missing `.dotagents/` exits non-zero with an error message
- [x] 7.7 Add e2e test: `undeploy` with explicit PATH removes files from the correct workspace
- [x] 7.8 Add e2e test: `undeploy` with a PATH missing `.dotagents/` exits non-zero with an error message

## 8. Verification

- [x] 8.1 Run `mise check` and fix any fmt/clippy failures
- [x] 8.2 Run `mise tests` and confirm all unit, integration, and e2e tests pass
