//! End-to-end tests for `dotagents ls`.
//!
//! Groups covered
//! ──────────────
//!  1.  Workspace discovery  – fails without .dotagents/, works after init
//!  2.  Output content       – skills section, commands section, count summary
//!  3.  Filter flags         – --commands, --skills, both, neither
//!  4.  Verbose mode         – --full flag shows untruncated descriptions
//!  5.  Empty workspace      – graceful handling when no items exist

use super::TestWorkspace;

// ═════════════════════════════════════════════════════════════════════════════
// Group 1 – workspace discovery
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ls_fails_without_workspace() {
    let ws = TestWorkspace::new();
    let result = ws.run(&["ls"]);
    result.assert_failure();
    assert!(
        !result.stderr.is_empty(),
        "a helpful error should be written to stderr"
    );
}

#[test]
fn ls_error_mentions_init() {
    let ws = TestWorkspace::new();
    let result = ws.run(&["ls"]);
    result.assert_failure();
    assert!(
        result.stderr.to_lowercase().contains("init"),
        "error should suggest running `dotagents init`; stderr: {}",
        result.stderr
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 2 – output content
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ls_after_init_shows_skills_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        result.stderr.contains("Skills"),
        "output should include a Skills section; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_after_init_shows_commands_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        result.stderr.contains("Commands"),
        "output should include a Commands section; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_shows_sample_hello_skill() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        result.stderr.contains("hello-skill"),
        "sample skill 'hello-skill' should appear in ls output; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_shows_sample_hello_command() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        result.stderr.contains("hello"),
        "sample command 'hello' should appear in ls output; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_shows_count_summary_in_outro() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    // The outro line contains "N skill(s) · M command(s)".
    assert!(
        result.stderr.contains("skill(s)") && result.stderr.contains("command(s)"),
        "outro should show count summary; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_newly_added_command_appears_in_output() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "command",
        "brand-new",
        "--description",
        "A brand new command",
    ])
    .assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        result.stderr.contains("brand-new"),
        "newly added command should appear in ls; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_newly_added_skill_appears_in_output() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "skill",
        "brand-new-skill",
        "--description",
        "A brand new skill",
    ])
    .assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        result.stderr.contains("brand-new-skill"),
        "newly added skill should appear in ls; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_removed_command_no_longer_appears() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "ephemeral", "--description", "gone"])
        .assert_success();
    ws.run(&["rm", "command", "ephemeral"]).assert_success();
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        !result.stderr.contains("ephemeral"),
        "removed command must not appear in ls; stderr: {}",
        result.stderr
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 3 – filter flags
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ls_commands_flag_shows_commands_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls", "--commands"]);
    result.assert_success();
    assert!(
        result.stderr.contains("Commands"),
        "--commands flag should show Commands section; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_commands_flag_hides_skills_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls", "--commands"]);
    result.assert_success();
    assert!(
        !result.stderr.contains("Skills ("),
        "--commands flag should suppress Skills section; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_skills_flag_shows_skills_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls", "--skills"]);
    result.assert_success();
    assert!(
        result.stderr.contains("Skills"),
        "--skills flag should show Skills section; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_skills_flag_hides_commands_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls", "--skills"]);
    result.assert_success();
    assert!(
        !result.stderr.contains("Commands ("),
        "--skills flag should suppress Commands section; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_both_flags_shows_both_sections() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls", "--commands", "--skills"]);
    result.assert_success();
    assert!(
        result.stderr.contains("Skills") && result.stderr.contains("Commands"),
        "--commands --skills together should show both sections; stderr: {}",
        result.stderr
    );
}

#[test]
fn ls_commands_summary_shows_only_command_count() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["ls", "--commands"]);
    result.assert_success();
    assert!(
        result.stderr.contains("command(s)"),
        "summary should mention commands; stderr: {}",
        result.stderr
    );
    assert!(
        !result.stderr.contains("skill(s)"),
        "summary should not mention skills when --commands is used; stderr: {}",
        result.stderr
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 4 – verbose / full mode
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ls_full_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["ls", "--full"]).assert_success();
}

#[test]
fn ls_verbose_flag_succeeds() {
    // -v is the global verbosity flag; ls treats verbosity >= 1 as full mode.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["-v", "ls"]).assert_success();
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 5 – empty workspace
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ls_empty_workspace_exits_zero() {
    // Init with no skills and no commands → ls should still exit 0.
    let ws = TestWorkspace::new();
    ws.run(&["init", "--no-skill", "--no-command"])
        .assert_success();
    // Remove any init-scaffolded files.
    let d = ws.root_dir_name();
    let skills_dir = ws.path(format!("{d}/skills"));
    let cmds_dir = ws.path(format!("{d}/commands"));
    if skills_dir.exists() {
        std::fs::remove_dir_all(&skills_dir).ok();
        std::fs::create_dir(&skills_dir).ok();
    }
    if cmds_dir.exists() {
        std::fs::remove_dir_all(&cmds_dir).ok();
        std::fs::create_dir(&cmds_dir).ok();
    }
    ws.run(&["ls"]).assert_success();
}

#[test]
fn ls_empty_workspace_mentions_no_items_found() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--no-skill", "--no-command"])
        .assert_success();
    let d = ws.root_dir_name();
    let skills_dir = ws.path(format!("{d}/skills"));
    let cmds_dir = ws.path(format!("{d}/commands"));
    if skills_dir.exists() {
        std::fs::remove_dir_all(&skills_dir).ok();
        std::fs::create_dir(&skills_dir).ok();
    }
    if cmds_dir.exists() {
        std::fs::remove_dir_all(&cmds_dir).ok();
        std::fs::create_dir(&cmds_dir).ok();
    }
    let result = ws.run(&["ls"]);
    result.assert_success();
    assert!(
        result.stderr.to_lowercase().contains("no") || result.stderr.contains("0 skill"),
        "output should indicate nothing was found; stderr: {}",
        result.stderr
    );
}
