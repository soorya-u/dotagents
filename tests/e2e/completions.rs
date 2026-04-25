//! Tests for the `gen-completions` command.
//!
//! Verifies that every supported shell generates a correctly named,
//! non-empty completion file.  Also confirms that the command works
//! without an init'd workspace (it has no dependency on config files).

use super::TestWorkspace;
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────────────────────────

/// Generate completions for `shell` into a `completions/` subdirectory of the
/// workspace, assert success, and return the path to that directory.
fn generate_completions_dir(ws: &TestWorkspace, shell: &str) -> std::path::PathBuf {
    let dir = ws.root().join("completions");
    fs::create_dir_all(&dir).expect("failed to create completions dir");
    ws.run(&[
        "gen-completions",
        "--shell",
        shell,
        "--to",
        dir.to_str().unwrap(),
    ])
    .assert_success();
    dir
}

/// Generate completions for `shell`, then assert that `filename` was created
/// and contains non-empty content.
fn assert_completion_file(shell: &str, filename: &str) {
    let ws = TestWorkspace::new();
    let dir = generate_completions_dir(&ws, shell);
    let path = dir.join(filename);
    assert!(
        path.exists(),
        "{filename} should be created for shell '{shell}'"
    );
    assert!(
        !fs::read_to_string(&path).unwrap().is_empty(),
        "{filename} should not be empty"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-shell tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn gen_completions_bash_creates_non_empty_file() {
    assert_completion_file("bash", "dotagents.bash");
}

#[test]
fn gen_completions_zsh_creates_non_empty_file() {
    assert_completion_file("zsh", "_dotagents");
}

#[test]
fn gen_completions_fish_creates_non_empty_file() {
    assert_completion_file("fish", "dotagents.fish");
}

#[test]
fn gen_completions_powershell_creates_non_empty_file() {
    assert_completion_file("powershell", "_dotagents.ps1");
}

#[test]
fn gen_completions_elvish_creates_non_empty_file() {
    assert_completion_file("elvish", "dotagents.elv");
}

// ─────────────────────────────────────────────────────────────────────────────
// Content and independence tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn gen_completions_bash_file_references_binary_name() {
    let ws = TestWorkspace::new();
    let dir = generate_completions_dir(&ws, "bash");
    let content = fs::read_to_string(dir.join("dotagents.bash")).unwrap();
    assert!(
        content.contains("dotagents"),
        "bash completion should reference the binary name"
    );
}

#[test]
fn gen_completions_works_without_an_init_workspace() {
    // gen-completions must not require a `.dotagents[-debug]` directory.
    let ws = TestWorkspace::new();
    let dir = ws.root().join("completions");
    fs::create_dir_all(&dir).unwrap();
    for shell in &["bash", "zsh", "fish", "powershell", "elvish"] {
        ws.run(&[
            "gen-completions",
            "--shell",
            shell,
            "--to",
            dir.to_str().unwrap(),
        ])
        .assert_success();
    }
}
