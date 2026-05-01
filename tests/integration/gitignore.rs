//! Integration tests for .gitignore fence management.

use super::{TestWorkspace, init_with_mycode_provider};

const FENCE_START: &str = "# BEGIN dotagents managed - do not edit manually";
const FENCE_END: &str = "# END dotagents managed";

// ─────────────────────────────────────────────────────────────────────────────
// --gitignore flag
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_gitignore_flag_creates_managed_fence_in_workspace_gitignore() {
    // --gitignore forces the fence section to be written without prompting.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();

    assert!(
        ws.file_exists(".gitignore"),
        "workspace .gitignore should be created after deploy --gitignore"
    );
    let content = ws.read_file(".gitignore");
    assert!(
        content.contains(FENCE_START),
        "workspace .gitignore should contain the dotagents managed fence; got:\n{content}"
    );
    assert!(
        content.contains(FENCE_END),
        "workspace .gitignore fence should be closed; got:\n{content}"
    );
}

#[test]
fn deploy_gitignore_flag_adds_output_paths_to_fence() {
    // Deployed output paths should appear inside the fence section.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();

    let content = ws.read_file(".gitignore");
    // The mycode provider writes to .mycode/; at least one .mycode path should be listed.
    assert!(
        content.contains(".mycode"),
        ".gitignore fence should reference .mycode output paths; got:\n{content}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// --no-gitignore flag
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_no_gitignore_flag_skips_fence_creation() {
    // --no-gitignore must not touch the workspace .gitignore.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Either the workspace .gitignore does not exist, or it lacks the managed fence.
    if ws.file_exists(".gitignore") {
        let content = ws.read_file(".gitignore");
        assert!(
            !content.contains(FENCE_START),
            "deploy --no-gitignore should not write the managed fence; got:\n{content}"
        );
    }
    // If .gitignore doesn't exist at all, the test passes implicitly.
}

// ─────────────────────────────────────────────────────────────────────────────
// Idempotency
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn second_deploy_does_not_duplicate_fence_entries() {
    // Running deploy --gitignore twice should not duplicate the fence section.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();
    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();

    let content = ws.read_file(".gitignore");
    let fence_count = content.matches(FENCE_START).count();
    assert_eq!(
        fence_count, 1,
        "fence start marker should appear exactly once; got:\n{content}"
    );
}

#[test]
fn second_deploy_does_not_duplicate_path_entries_in_fence() {
    // Each path should appear exactly as many times after two deploys as after one.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();
    let count_after_first = ws
        .read_file(".gitignore")
        .lines()
        .filter(|l| l.contains(".mycode") && !l.starts_with('#'))
        .count();

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();
    let count_after_second = ws
        .read_file(".gitignore")
        .lines()
        .filter(|l| l.contains(".mycode") && !l.starts_with('#'))
        .count();

    assert_eq!(
        count_after_first, count_after_second,
        "second deploy should not add duplicate .mycode entries to .gitignore"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// User content preservation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn user_content_in_workspace_gitignore_preserved_after_deploy() {
    // Existing .gitignore content must survive a deploy --gitignore run.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Write user-owned gitignore entries before deploy.
    ws.write_file(".gitignore", "*.log\n.DS_Store\ntarget/\n");

    ws.run_command(&["deploy", "--offline", "--gitignore"])
        .assert_success();

    let content = ws.read_file(".gitignore");
    assert!(
        content.contains("*.log"),
        "user entry *.log should be preserved; got:\n{content}"
    );
    assert!(
        content.contains(".DS_Store"),
        "user entry .DS_Store should be preserved; got:\n{content}"
    );
    assert!(
        content.contains("target/"),
        "user entry target/ should be preserved; got:\n{content}"
    );
    assert!(
        content.contains(FENCE_START),
        "managed fence should also be present; got:\n{content}"
    );
}
