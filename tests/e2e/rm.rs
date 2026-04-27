//! End-to-end tests for `dotagents rm command` and `dotagents rm skill`.
//!
//! Groups covered
//! ──────────────
//!  1.  Workspace discovery  – fails without .dotagents/
//!  2.  Command removal      – file deleted, error when missing
//!  3.  Skill removal        – directory deleted, error when missing
//!  4.  --force              – skips TTY confirm (non-TTY tests always skip it anyway)
//!  5.  Integration with ls  – item gone from ls output after removal

use super::TestWorkspace;

// ═════════════════════════════════════════════════════════════════════════════
// Group 1 – workspace discovery
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn rm_command_without_workspace_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["rm", "command", "x"]).assert_failure();
}

#[test]
fn rm_skill_without_workspace_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["rm", "skill", "x"]).assert_failure();
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 2 – command removal
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn rm_command_deletes_md_file() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "delete-me", "--description", "gone"])
        .assert_success();
    ws.run(&["rm", "command", "delete-me"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        !ws.file_exists(format!("{d}/commands/delete-me.md")),
        "command file should be deleted"
    );
}

#[test]
fn rm_command_not_found_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["rm", "command", "ghost"]).assert_failure();
}

#[test]
fn rm_command_not_found_error_on_stderr() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["rm", "command", "ghost"]);
    result.assert_failure();
    assert!(
        !result.stderr.is_empty(),
        "error output should go to stderr"
    );
}

#[test]
fn rm_command_only_deletes_named_command() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "keep-me", "--description", "stays"])
        .assert_success();
    ws.run(&["add", "command", "remove-me", "--description", "goes"])
        .assert_success();
    ws.run(&["rm", "command", "remove-me"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.file_exists(format!("{d}/commands/keep-me.md")),
        "untouched command should still exist"
    );
    assert!(
        !ws.file_exists(format!("{d}/commands/remove-me.md")),
        "removed command should be gone"
    );
}

#[test]
fn rm_command_success_exits_zero() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "exit-test", "--description", "x"])
        .assert_success();
    ws.run(&["rm", "command", "exit-test"]).assert_success();
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 3 – skill removal
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn rm_skill_deletes_directory() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "delete-skill", "--description", "gone"])
        .assert_success();
    ws.run(&["rm", "skill", "delete-skill"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        !ws.dir_exists(format!("{d}/skills/delete-skill")),
        "skill directory should be deleted"
    );
}

#[test]
fn rm_skill_deletes_skill_md_inside_directory() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "inner-skill", "--description", "gone"])
        .assert_success();
    ws.run(&["rm", "skill", "inner-skill"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        !ws.file_exists(format!("{d}/skills/inner-skill/SKILL.md")),
        "SKILL.md inside the directory should be deleted"
    );
}

#[test]
fn rm_skill_not_found_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["rm", "skill", "ghost-skill"]).assert_failure();
}

#[test]
fn rm_skill_not_found_error_on_stderr() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["rm", "skill", "ghost-skill"]);
    result.assert_failure();
    assert!(
        !result.stderr.is_empty(),
        "error output should go to stderr"
    );
}

#[test]
fn rm_skill_only_deletes_named_skill() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "keep-skill", "--description", "stays"])
        .assert_success();
    ws.run(&["add", "skill", "remove-skill", "--description", "goes"])
        .assert_success();
    ws.run(&["rm", "skill", "remove-skill"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.dir_exists(format!("{d}/skills/keep-skill")),
        "untouched skill directory should still exist"
    );
    assert!(
        !ws.dir_exists(format!("{d}/skills/remove-skill")),
        "removed skill directory should be gone"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 4 – --force flag
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn rm_command_force_flag_succeeds() {
    // In non-TTY the confirm is skipped regardless; --force just makes it explicit.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "force-rm", "--description", "x"])
        .assert_success();
    ws.run(&["rm", "command", "force-rm", "--force"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(!ws.file_exists(format!("{d}/commands/force-rm.md")));
}

#[test]
fn rm_skill_force_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "force-skill", "--description", "x"])
        .assert_success();
    ws.run(&["rm", "skill", "force-skill", "--force"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(!ws.dir_exists(format!("{d}/skills/force-skill")));
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 5 – integration with ls
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn rm_command_item_gone_from_ls_output() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "vanish-cmd", "--description", "poof"])
        .assert_success();
    ws.run(&["rm", "command", "vanish-cmd"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        !result.stdout.contains("vanish-cmd"),
        "removed command should not appear in ls; stdout: {}",
        result.stdout
    );
}

#[test]
fn rm_skill_item_gone_from_ls_output() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "vanish-skill", "--description", "poof"])
        .assert_success();
    ws.run(&["rm", "skill", "vanish-skill"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        !result.stdout.contains("vanish-skill"),
        "removed skill should not appear in ls; stdout: {}",
        result.stdout
    );
}

#[test]
fn add_then_rm_then_add_again_succeeds() {
    // Full CRUD roundtrip: create → remove → recreate.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "roundtrip", "--description", "v1"])
        .assert_success();
    ws.run(&["rm", "command", "roundtrip"]).assert_success();
    ws.run(&["add", "command", "roundtrip", "--description", "v2"])
        .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/roundtrip.md"));
    assert!(
        content.contains("v2"),
        "second add should have written v2 description; content:\n{content}"
    );
}
