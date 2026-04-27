//! Smoke tests for `dotagents rm command` and `dotagents rm skill`.

use super::TestWorkspace;

// ─────────────────────────────────────────────────────────────────────────────
// Workspace discovery
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn rm_command_without_workspace_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["rm", "command", "missing"])
        .assert_failure();
}

#[test]
fn rm_skill_without_workspace_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["rm", "skill", "missing"]).assert_failure();
}

// ─────────────────────────────────────────────────────────────────────────────
// Non-existent targets
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn rm_command_not_found_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["rm", "command", "no-such-cmd"])
        .assert_failure();
}

#[test]
fn rm_skill_not_found_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["rm", "skill", "no-such-skill"])
        .assert_failure();
}

// ─────────────────────────────────────────────────────────────────────────────
// Successful removal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn rm_command_removes_file() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "command", "to-remove", "--description", "bye"])
        .assert_success();
    ws.run_command(&["rm", "command", "to-remove"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(
        !ws.file_exists(format!("{d}/commands/to-remove.md")),
        "file should no longer exist after rm"
    );
}

#[test]
fn rm_skill_removes_directory() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "skill", "to-remove", "--description", "bye"])
        .assert_success();
    ws.run_command(&["rm", "skill", "to-remove"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(
        !ws.dir_exists(format!("{d}/skills/to-remove")),
        "skill directory should no longer exist after rm"
    );
}

#[test]
fn rm_command_with_force_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "command", "forced", "--description", "x"])
        .assert_success();
    ws.run_command(&["rm", "command", "forced", "--force"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(!ws.file_exists(format!("{d}/commands/forced.md")));
}
