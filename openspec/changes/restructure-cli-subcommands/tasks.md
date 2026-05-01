## 1. Init flag restructuring

- [x] 1.1 Add `Feature` enum to `src/cli/options.rs` with variants `Commands`, `Instructions`, `Mcp`, `Skills`, `None` (Clap-parseable, kebab-case)
- [x] 1.2 Replace `no_mcp`, `no_command`, `no_instruction`, `no_skill` bool fields in `InitOptions` with `features: Option<Vec<Feature>>` using `value_delimiter = ','` and `num_args = 0..`
- [x] 1.3 Add post-parse validation in `src/cli/init.rs`: error if `features` is `Some([])` (empty), error if `features` contains `None` alongside other variants
- [x] 1.4 Update `is_tui_mode()` in `src/cli/init.rs`: replace four-bool check with `opts.features.is_none() && opts.template.is_none() && stdin.is_terminal()`
- [x] 1.5 Update `initialize_agents_dir()` to derive enabled-features set from `opts.features` (`None` → all four, `Some([None])` → empty, `Some(list)` → explicit set) and pass to `InitFile::with_skip_if` guards

## 2. New `commands` subcommand group

- [x] 2.1 Add `SubLsOptions { full: bool }` struct to `src/cli/options.rs`
- [x] 2.2 Add `CommandsAction` enum to `src/cli/options.rs` with variants `New(AddCommandOptions)`, `Rm(RmCommandOptions)`, `Ls(SubLsOptions)`
- [x] 2.3 Add `Commands(CommandsAction)` variant to the top-level `Action` enum in `src/cli/options.rs`
- [x] 2.4 Create `src/cli/commands.rs`: implement `run_commands(action: CommandsAction)` that delegates `New` → existing add-command logic, `Rm` → existing rm-command logic, `Ls` → existing ls-commands logic
- [x] 2.5 Wire `Action::Commands` in `src/cli/runner.rs` (or wherever dispatch lives) to call `commands::run_commands`

## 3. Expand `skills` subcommand group

- [x] 3.1 Add `New(AddSkillOptions)`, `Rm(RmSkillOptions)`, `Ls(SubLsOptions)` variants to `SkillsAction` enum in `src/cli/options.rs`
- [x] 3.2 Implement handlers for `SkillsAction::New`, `SkillsAction::Rm`, `SkillsAction::Ls` in `src/cli/skills.rs`, delegating to existing add-skill, rm-skill, ls-skills logic

## 4. Remove implicit `-v` → `--full` tie-in

- [x] 4.1 Remove any code in the old `ls.rs` (or wherever `--full` is set) that enables full descriptions based on the global verbose level — `--full` is now only set by the explicit flag

## 5. Delete obsolete files

- [x] 5.1 Delete `src/cli/add.rs`
- [x] 5.2 Delete `src/cli/rm.rs`
- [x] 5.3 Delete `src/cli/ls.rs`
- [x] 5.4 Remove all `mod add`, `mod rm`, `mod ls` declarations and any `use` imports referencing them

## 6. Verification

- [x] 6.1 Run `mise check` (cargo fmt + cargo clippy) — fix all warnings and errors
- [x] 6.2 Run `mise test-all` (unit + integration + e2e) — fix any failures caused by the removed commands or changed flags
- [x] 6.3 Manually smoke-test: `dotagents commands new test-cmd`, `dotagents commands ls`, `dotagents commands rm test-cmd`, `dotagents skills new test-skill`, `dotagents skills ls`, `dotagents skills rm test-skill`
- [x] 6.4 Manually smoke-test: `dotagents init --features commands,mcp` (headless), `dotagents init --features none` (headless), `dotagents init --features none,commands` (should error)
