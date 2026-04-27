//! Smoke tests for `dotagents ls`.

use super::TestWorkspace;

// ─────────────────────────────────────────────────────────────────────────────
// Workspace discovery
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ls_without_workspace_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["ls"]).assert_failure();
}

#[test]
fn ls_after_init_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["ls"]).assert_success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Filter flags
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ls_commands_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["ls", "--commands"]).assert_success();
}

#[test]
fn ls_skills_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["ls", "--skills"]).assert_success();
}

#[test]
fn ls_both_flags_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["ls", "--commands", "--skills"])
        .assert_success();
}

#[test]
fn ls_full_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["ls", "--full"]).assert_success();
}
