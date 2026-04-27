## 1. CLI Options & Dispatch

- [x] 1.1 Add `LsOptions { verbose: bool, commands: bool, skills: bool }` struct with clap derives to `src/cli/options.rs`
- [x] 1.2 Add `AddAction` subcommand enum with `Command(AddCommandOptions)` and `Skill(AddSkillOptions)` variants to `src/cli/options.rs`
- [x] 1.3 Add `AddCommandOptions { name, description, category, tags, force, deploy }` and `AddSkillOptions { name, description, license, compatibility, force, deploy }` Args structs
- [x] 1.4 Add `RmAction` subcommand enum with `Command(RmCommandOptions)` and `Skill(RmSkillOptions)` variants to `src/cli/options.rs`
- [x] 1.5 Add `RmCommandOptions { name, force, deploy }` and `RmSkillOptions { name, force, deploy }` Args structs
- [x] 1.6 Add `Action::Ls(LsOptions)`, `Action::Add(AddAction)`, `Action::Rm(RmAction)` variants to the `Action` enum
- [x] 1.7 Add dispatch arms for `Ls`, `Add`, `Rm` in `src/cli/runner.rs`

## 2. `ls` Implementation

- [x] 2.1 Create `src/cli/ls.rs` with `pub(crate) fn run_ls(opts: LsOptions) -> Result<bool>`
- [x] 2.2 Implement skill reader: glob `.dotagents/skills/*/SKILL.md`, parse YAML frontmatter via `gray_matter`, extract `name` and `description`
- [x] 2.3 Implement command reader: glob `.dotagents/commands/*.md`, parse YAML frontmatter via `gray_matter`, extract `name` and `description`
- [x] 2.4 Create `src/cli/ui/ls.rs` with `render_ls(skills, commands, opts)` function using cliclack `intro`, section headers, item rows, and `outro` with count summary
- [x] 2.5 Implement terminal-width detection via `crossterm::terminal::size()` with 80-col fallback
- [x] 2.6 Implement `truncate_to_width(text: &str, width: usize) -> String` (appends `…` when truncated)
- [x] 2.7 Implement `wrap_at_width(text: &str, width: usize) -> String` for `--verbose` display
- [x] 2.8 Apply `--commands` / `--skills` filter logic: neither or both → show both sections; one → show only that section
- [x] 2.9 Handle empty sections (omit header) and empty workspace (print "No skills or commands found.", exit 0)
- [x] 2.10 Wire `src/cli/ui/ls.rs` into `src/cli/ui/mod.rs`

## 3. `add` Implementation

- [x] 3.1 Create `src/cli/add.rs` with `pub(crate) fn run_add(action: AddAction) -> Result<bool>`
- [x] 3.2 Implement dual-mode field collection for commands: if any flag provided or non-TTY → use flags with empty defaults; if TTY + no flags → cliclack prompts for description, category, tags
- [x] 3.3 Implement dual-mode field collection for skills: same pattern for description, license, compatibility
- [x] 3.4 Implement command file creation: serialize frontmatter as YAML, append starter body template with `name` interpolated, write to `.dotagents/commands/<name>.md` via `utils/fs::write_file`
- [x] 3.5 Implement skill file creation: create `.dotagents/skills/<name>/` directory, serialize frontmatter with `metadata.version = "1.0"`, append skill starter body, write `SKILL.md`
- [x] 3.6 Implement `--force` check: if target path exists and `--force` not set, return error; if `--force`, overwrite
- [x] 3.7 Implement post-mutation deploy logic: `--deploy` → call `deploy(DeployOptions::default())`; TTY + no flag → cliclack confirm "Deploy now?" (default No); non-TTY → skip

## 4. `rm` Implementation

- [x] 4.1 Create `src/cli/rm.rs` with `pub(crate) fn run_rm(action: RmAction) -> Result<bool>`
- [x] 4.2 Implement command removal: resolve `.dotagents/commands/<name>.md`; error if not found; TTY without `--force` → cliclack confirm before deletion; delete file
- [x] 4.3 Implement skill removal: resolve `.dotagents/skills/<name>/`; error if not found; TTY without `--force` → cliclack confirm before deletion; `fs::remove_dir_all` the directory
- [x] 4.4 Implement post-removal deploy logic (same pattern as `add`: `--deploy` flag, TTY confirm, non-TTY skip)

## 5. Starter Templates

- [x] 5.1 Add command starter template string constant to `src/constants/mocks.rs` (or a new `src/constants/templates.rs`)
- [x] 5.2 Add skill starter template string constant alongside command template
- [x] 5.3 Implement `render_starter(template: &str, name: &str) -> String` that substitutes `{name}` placeholder

## 6. Module Wiring

- [x] 6.1 Declare `ls`, `add`, `rm` modules in `src/cli/mod.rs`
- [x] 6.2 Declare `ui::ls` submodule in `src/cli/ui/mod.rs`

## 7. Verification

- [x] 7.1 Run `mise check` (cargo fmt + clippy) and fix all warnings
- [x] 7.2 Run `mise test-all` and fix any failing tests
- [x] 7.3 Manual smoke: `cargo run -- ls` shows both sections
- [x] 7.4 Manual smoke: `cargo run -- ls --commands` shows only commands
- [x] 7.5 Manual smoke: `cargo run -- add command test-cmd` in TTY → prompts → creates file
- [x] 7.6 Manual smoke: `cargo run -- add skill test-skill --description "Test" --license MIT` → creates directory + SKILL.md without prompting
- [x] 7.7 Manual smoke: `cargo run -- rm command test-cmd` in TTY → confirm prompt → deletion
- [x] 7.8 Manual smoke: `cargo run -- add command test-cmd --deploy` → creates file then deploys
