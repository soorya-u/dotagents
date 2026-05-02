//! Integration tests for `dotagents undeploy`.

use super::{TestWorkspace, init_with_mycode_provider};
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// Basic undeploy lifecycle
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn undeploy_removes_deployed_files_and_clears_cache() {
    // After deploy + undeploy, deployed files are gone and cache is empty.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Deployed files should exist.
    assert!(
        ws.file_exists(".mycode/instructions.md"),
        "instructions.md should exist after deploy"
    );

    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();

    assert!(
        !ws.file_exists(".mycode/instructions.md"),
        "instructions.md should be removed after undeploy"
    );

    // Cache should be emptied (file still exists but has no entries).
    let cache_path = format!("{}/cache.toml", ws.root_dir_name());
    if ws.file_exists(&cache_path) {
        let content = ws.read_file(&cache_path);
        assert!(
            !content.contains("[providers."),
            "cache.toml should have no provider entries after undeploy; got:\n{content}"
        );
    }
}

#[test]
fn undeploy_with_no_cache_exits_cleanly() {
    // When cache is missing or empty, undeploy exits 0 with no errors.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Do NOT deploy — no cache.toml exists.
    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();
}

// ─────────────────────────────────────────────────────────────────────────────
// --no-gitignore flag
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn undeploy_no_gitignore_leaves_fence_intact() {
    // --no-gitignore preserves the .gitignore managed fence.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();

    // Fence should now exist.
    let gi_before = ws.read_file(".gitignore");
    assert!(
        gi_before.contains("BEGIN dotagents managed"),
        ".gitignore should contain managed fence after deploy"
    );

    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();

    let gi_after = ws.read_file(".gitignore");
    assert!(
        gi_after.contains("BEGIN dotagents managed"),
        ".gitignore fence should be preserved with --no-gitignore"
    );
}

#[test]
fn undeploy_removes_gitignore_fence_by_default() {
    // Without --no-gitignore, undeploy removes the managed fence.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();

    let gi_before = ws.read_file(".gitignore");
    assert!(
        gi_before.contains("BEGIN dotagents managed"),
        ".gitignore should have fence after deploy"
    );

    ws.run_command(&["undeploy"]).assert_success();

    let gi_after = ws.read_file(".gitignore");
    assert!(
        !gi_after.contains("BEGIN dotagents managed"),
        ".gitignore fence should be removed after undeploy; got:\n{gi_after}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// --no-cache deploy then undeploy
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn no_cache_deploy_then_undeploy_works() {
    // deploy --no-cache still writes cache.toml, so undeploy can use it.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--no-cache"])
        .assert_success();

    assert!(
        ws.file_exists(".mycode/instructions.md"),
        "instructions.md should exist after --no-cache deploy"
    );

    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();

    assert!(
        !ws.file_exists(".mycode/instructions.md"),
        "instructions.md should be removed after undeploy following --no-cache deploy"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Idempotency: undeploy twice is safe
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn undeploy_twice_is_safe() {
    // Running undeploy a second time is a no-op (files already gone).
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();

    // Second undeploy — cache is already empty.
    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Empty directory pruning
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn undeploy_prunes_empty_parent_directory() {
    // After removing all commands from .mycode/commands/, the directory is pruned.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.dir_exists(".mycode/commands"),
        ".mycode/commands should exist after deploy"
    );

    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();

    assert!(
        !ws.dir_exists(".mycode/commands"),
        ".mycode/commands should be pruned after all files removed"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// User-edited file handling (non-TTY: warn and skip)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn undeploy_skips_user_edited_file_in_non_tty() {
    // A manually edited deployed file is skipped (not deleted) in non-TTY mode.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Simulate user editing the deployed output.
    let out_path = ws.root().join(".mycode/instructions.md");
    fs::write(&out_path, "User has edited this file.").unwrap();

    // stdin is null (non-TTY) in run_command; edited file should be skipped.
    ws.run_command(&["undeploy", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.file_exists(".mycode/instructions.md"),
        "user-edited file should be preserved in non-TTY undeploy"
    );
}

#[test]
fn undeploy_force_deletes_user_edited_file() {
    // --force deletes user-edited files without prompting.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let out_path = ws.root().join(".mycode/instructions.md");
    fs::write(&out_path, "User has edited this file.").unwrap();

    ws.run_command(&["undeploy", "--no-gitignore", "--force"])
        .assert_success();

    assert!(
        !ws.file_exists(".mycode/instructions.md"),
        "--force should delete user-edited files"
    );
}
