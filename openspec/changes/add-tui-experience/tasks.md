## 1. Dependency

- [x] 1.1 Add `cliclack` via `cargo add cliclack` and verify it compiles with the existing `crossterm 0.29` — resolve any version conflict

## 2. Mock Files

- [x] 2.1 Create `src/mocks/local.config.starter.toml` with `features` list and `targets = []`, no `[providers]` block
- [x] 2.2 Update `src/constants/mocks.rs`: add `LOCAL_CONFIG_STARTER` pointing to the new file and rename `LOCAL_CONFIG` to `LOCAL_CONFIG_WITH_PROVIDER` pointing to existing `local.config.toml`

## 3. Options & Types

- [x] 3.1 Add `InitTemplate` enum (`Starter`, `WithCustomProvider`) to `src/cli/options.rs` with clap `ValueEnum` derive
- [x] 3.2 Add `pub template: Option<InitTemplate>` field to `InitOptions` with `--template` clap long arg
- [x] 3.3 Update existing `InitOptions` tests and add tests for the new `template` field defaults

## 4. New UI Module

- [x] 4.1 Create `src/cli/ui/mod.rs` with `pub(crate) mod init` and `pub(crate) mod deploy` declarations
- [x] 4.2 Create `src/cli/ui/init.rs` — implement `run_init_wizard(opts: &mut InitOptions) -> anyhow::Result<()>` using cliclack: intro, feature multiselect, template select, overwrite confirm (when `!opts.force`), outro
- [x] 4.3 Create `src/cli/ui/init.rs` target selection — after the file-write step, add `prompt_targets() -> anyhow::Result<Vec<String>>`: show a cliclack spinner while fetching `Registry::fetch(REGISTRY_URL)`, then a multiselect of all `registry.providers` keys sorted alphabetically; on fetch failure emit a cliclack `log::warn` and return empty vec
- [x] 4.4 Create `src/cli/ui/deploy.rs` — implement two functions: `prompt_offline() -> bool` (cliclack select: Run online / Run offline, default online, returns true if offline chosen) and `prompt_gitignore_update(new_path_count: usize) -> bool` (cliclack select: Yes / No, default No); both return the safe default immediately in non-TTY environments
- [x] 4.5 Register `ui` as a submodule in `src/cli/mod.rs`

## 5. Init Command

- [x] 5.1 Update `initialize_agents_dir` in `src/cli/init.rs` to implement dual-mode logic: call `ui::init::run_init_wizard` when no flags and TTY; otherwise skip prompts
- [x] 5.2 Add template-gated `InitFile` entries for `templates/mycode/*` files, skipped when template is `Starter`
- [x] 5.3 Switch the `local.config.toml` `InitFile` to use `mocks::LOCAL_CONFIG_STARTER` vs `mocks::LOCAL_CONFIG_WITH_PROVIDER` based on resolved template
- [x] 5.4 After all files are written in TUI mode, call `ui::init::prompt_targets`; if non-empty result, read the written `config.toml`, update its `targets` array with the selected provider names, and write it back

## 6. Deploy Command

- [x] 6.1 Remove `prompt_gitignore_update` from `src/utils/gitignore.rs`
- [x] 6.2 In `src/cli/deploy.rs`, before the registry-fetch block (added by PR #38), call `ui::deploy::prompt_offline()` in TUI mode (TTY + no `--offline` flag) and set `opts.offline = true` if the user chooses offline — note: this file already has registry resolution code from PR #38; preserve that code and only insert the prompt before it
- [x] 6.3 Update the gitignore prompt call in `src/cli/deploy.rs` to use `ui::deploy::prompt_gitignore_update` instead of `gitignore::prompt_gitignore_update`

## 7. Verification

- [x] 7.1 Run `mise check` (cargo fmt + clippy) and fix all warnings
- [x] 7.2 Run `mise test-all` (unit + integration + e2e) and fix any failures
- [ ] 7.3 Manual smoke test: `cargo run -- init` in a TTY — verify full wizard (features, template, target selection with spinner), both templates produce correct file sets and `config.toml` targets
- [ ] 7.4 Manual smoke test: `cargo run -- init --no-mcp --template starter` — flag-only path skips all prompts and skips registry fetch
- [ ] 7.5 Manual smoke test: `cargo run -- deploy` in a TTY — verify offline prompt appears before deploy, gitignore select appears after
- [ ] 7.6 Manual smoke test: `cargo run -- deploy --offline` — verify no offline prompt shown, deploy runs in offline mode
