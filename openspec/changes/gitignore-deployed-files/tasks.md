## 1. Deploy Pipeline — Collect Written Paths

- [ ] 1.1 Change `render_feature_with_settings` in `src/templates/renderer.rs` to return `Result<PathBuf>` (the path it wrote) instead of `Result<()>`
- [ ] 1.2 Update `deploy_feature` in `src/cli/deploy.rs` to collect all returned paths into a `Vec<PathBuf>` across providers and features
- [ ] 1.3 Return the aggregated `Vec<PathBuf>` from `deploy()` so the caller can pass it to the gitignore update step

## 2. CLI Flags

- [ ] 2.1 Add `--gitignore` boolean flag to the `Deploy` subcommand in `src/cli/options.rs`
- [ ] 2.2 Add `--no-gitignore` boolean flag to the `Deploy` subcommand in `src/cli/options.rs`
- [ ] 2.3 Pass both flags through to the gitignore update logic in `src/cli/deploy.rs`

## 3. Gitignore Utility

- [ ] 3.1 Create `src/utils/gitignore.rs` with a `read_gitignore(path: &PathBuf) -> Result<String>` helper that returns empty string if file missing
- [ ] 3.2 Implement `parse_fenced_section(content: &str) -> HashSet<String>` — extracts paths currently inside the dotagents fenced section
- [ ] 3.3 Implement `update_gitignore(content: &str, new_paths: &[String]) -> String` — adds any paths not already in the fence; preserves all content outside the fence; creates the fence if absent
- [ ] 3.4 Implement `write_gitignore(workspace_root: &PathBuf, new_paths: &[PathBuf]) -> Result<()>` — orchestrates read → update → write; returns `Ok(())` if nothing changed (no write needed)
- [ ] 3.5 Add `make_workspace_relative(path: &PathBuf, workspace: &PathBuf) -> Option<String>` helper to strip the workspace prefix from absolute target paths
- [ ] 3.6 Write unit tests for `parse_fenced_section`, `update_gitignore` (new file, existing no fence, existing with fence, all paths present, user content preserved)

## 4. Interactive Prompt

- [ ] 4.1 Add `is_tty() -> bool` helper in `src/utils/` using `std::io::IsTerminal` (stable since Rust 1.70) to detect non-interactive environments
- [ ] 4.2 Implement `prompt_gitignore_update(new_path_count: usize) -> bool` using `crossterm` (already a dependency) — prints `"Add {n} deployed path(s) to .gitignore? [y/N]: "` and reads a single keypress; returns `false` if non-TTY
- [ ] 4.3 Write a unit test for the TTY detection helper

## 5. Wire Into Deploy

- [ ] 5.1 In `src/cli/deploy.rs`, after all features are deployed, determine the gitignore mode:
  - `--gitignore` → call `write_gitignore` directly
  - `--no-gitignore` → skip
  - neither → call `prompt_gitignore_update`; if `true`, call `write_gitignore`
- [ ] 5.2 Wrap `write_gitignore` call in error handling that logs a warning and returns `Ok(())` on failure (non-fatal)
- [ ] 5.3 Skip the prompt and update entirely when the collected path list is empty

## 6. Verification

- [ ] 6.1 Run `cargo build` — no compilation errors
- [ ] 6.2 Run `cargo test` — all tests pass
- [ ] 6.3 Run `dotagents deploy` in a test workspace; confirm prompt appears, answer `y`, verify `.gitignore` contains the fenced section with correct paths
- [ ] 6.4 Run `dotagents deploy --gitignore`; confirm `.gitignore` is updated without a prompt
- [ ] 6.5 Run `dotagents deploy --no-gitignore`; confirm `.gitignore` is not modified
- [ ] 6.6 Run deploy a second time with `--gitignore`; confirm no duplicate entries are added
- [ ] 6.7 Verify `.github/copilot-instructions.md` entry is the specific path, not `.github/` or `.github/*`
- [ ] 6.8 Run `cargo fmt && cargo clippy` — no warnings
