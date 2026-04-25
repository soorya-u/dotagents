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

// ─────────────────────────────────────────────────────────────────────────────
// Per-shell tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn gen_completions_bash_creates_non_empty_file() {
    let ws = TestWorkspace::new();
    let dir = generate_completions_dir(&ws, "bash");
    let path = dir.join("dotagents.bash");
    assert!(path.exists(), "dotagents.bash should be created");
    assert!(
        !fs::read_to_string(&path).unwrap().is_empty(),
        "dotagents.bash should not be empty"
    );
}

#[test]
fn gen_completions_zsh_creates_non_empty_file() {
    let ws = TestWorkspace::new();
    let dir = generate_completions_dir(&ws, "zsh");
    let path = dir.join("_dotagents");
    assert!(path.exists(), "_dotagents should be created");
    assert!(
        !fs::read_to_string(&path).unwrap().is_empty(),
        "_dotagents should not be empty"
    );
}

#[test]
fn gen_completions_fish_creates_non_empty_file() {
    let ws = TestWorkspace::new();
    let dir = generate_completions_dir(&ws, "fish");
    let path = dir.join("dotagents.fish");
    assert!(path.exists(), "dotagents.fish should be created");
    assert!(
        !fs::read_to_string(&path).unwrap().is_empty(),
        "dotagents.fish should not be empty"
    );
}

#[test]
fn gen_completions_powershell_creates_non_empty_file() {
    let ws = TestWorkspace::new();
    let dir = generate_completions_dir(&ws, "powershell");
    let path = dir.join("_dotagents.ps1");
    assert!(path.exists(), "_dotagents.ps1 should be created");
    assert!(
        !fs::read_to_string(&path).unwrap().is_empty(),
        "_dotagents.ps1 should not be empty"
    );
}

#[test]
fn gen_completions_elvish_creates_non_empty_file() {
    let ws = TestWorkspace::new();
    let dir = generate_completions_dir(&ws, "elvish");
    let path = dir.join("dotagents.elv");
    assert!(path.exists(), "dotagents.elv should be created");
    assert!(
        !fs::read_to_string(&path).unwrap().is_empty(),
        "dotagents.elv should not be empty"
    );
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
