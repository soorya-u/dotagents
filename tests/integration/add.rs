//! Smoke tests for `dotagents add command` and `dotagents add skill`.

use super::TestWorkspace;

// ─────────────────────────────────────────────────────────────────────────────
// Workspace discovery
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn add_command_without_workspace_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["add", "command", "my-cmd"])
        .assert_failure();
}

#[test]
fn add_skill_without_workspace_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["add", "skill", "my-skill"])
        .assert_failure();
}

// ─────────────────────────────────────────────────────────────────────────────
// Command creation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn add_command_after_init_creates_file() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "command", "test-cmd", "--description", "A test"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.file_exists(format!("{d}/commands/test-cmd.md")),
        "test-cmd.md should be created in .dotagents/commands/"
    );
}

#[test]
fn add_command_twice_without_force_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "command", "dup", "--description", "First"])
        .assert_success();
    ws.run_command(&["add", "command", "dup", "--description", "Second"])
        .assert_failure();
}

#[test]
fn add_command_with_force_overwrites() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "command", "dup", "--description", "First"])
        .assert_success();
    ws.run_command(&[
        "add",
        "command",
        "dup",
        "--description",
        "Second",
        "--force",
    ])
    .assert_success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Skill creation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn add_skill_after_init_creates_skill_md() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "skill", "my-skill", "--description", "A skill"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.file_exists(format!("{d}/skills/my-skill/SKILL.md")),
        "SKILL.md should be created inside .dotagents/skills/my-skill/"
    );
}

#[test]
fn add_skill_creates_parent_directory() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "skill", "new-skill"])
        .assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.dir_exists(format!("{d}/skills/new-skill")),
        "skill directory should be created"
    );
}

#[test]
fn add_skill_twice_without_force_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "skill", "dup-skill", "--description", "First"])
        .assert_success();
    ws.run_command(&["add", "skill", "dup-skill", "--description", "Second"])
        .assert_failure();
}

#[test]
fn add_skill_with_force_overwrites() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["add", "skill", "dup-skill", "--description", "First"])
        .assert_success();
    ws.run_command(&[
        "add",
        "skill",
        "dup-skill",
        "--description",
        "Second",
        "--force",
    ])
    .assert_success();
}
