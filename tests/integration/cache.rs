//! Integration tests for deploy idempotency and cache behaviour.

use super::{TestWorkspace, init_with_mycode_provider};
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// cache.toml lifecycle
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cache_toml_written_after_first_deploy() {
    // After a successful deploy, cache.toml should exist in the application dir.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.file_exists(format!("{}/cache.toml", ws.root_dir_name())),
        "cache.toml should be created in the application directory after deploy"
    );
}

#[test]
fn no_cache_flag_does_not_write_cache_toml() {
    // --no-cache should skip both reading and writing cache.toml.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--no-cache"])
        .assert_success();

    assert!(
        !ws.file_exists(format!("{}/cache.toml", ws.root_dir_name())),
        "cache.toml should not be written when --no-cache is supplied"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Idempotency
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn two_consecutive_deploys_produce_identical_output() {
    // Running deploy twice without any source changes should yield byte-identical output.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();
    let first = ws.read_file(".mycode/commands/hello.md");

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();
    let second = ws.read_file(".mycode/commands/hello.md");

    assert_eq!(
        first, second,
        "output should be identical across two deploys with no source changes"
    );
}

#[test]
fn no_cache_deploy_twice_produces_identical_output() {
    // Even with --no-cache (fresh render each time), output should be deterministic.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--no-cache"])
        .assert_success();
    let first = ws.read_file(".mycode/instructions.md");

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--no-cache"])
        .assert_success();
    let second = ws.read_file(".mycode/instructions.md");

    assert_eq!(
        first, second,
        "output should be identical across --no-cache deploys with no source changes"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Source modification reflected on redeploy
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn modifying_source_is_reflected_in_subsequent_deploy() {
    // Changing the instructions source should update the deployed output on next deploy.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Append custom text to the source instruction file.
    let d = ws.root_dir_name();
    let src_path = ws.root().join(format!("{d}/INSTRUCTIONS.md"));
    let original = fs::read_to_string(&src_path).unwrap();
    fs::write(&src_path, format!("{original}\n\nCustom integration note.")).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let output = ws.read_file(".mycode/instructions.md");
    assert!(
        output.contains("Custom integration note."),
        "redeployed output should reflect source modification; got:\n{output}"
    );
}

#[test]
fn force_flag_overwrites_user_modified_output() {
    // If the user edits the deployed output, --force should overwrite it with the fresh render.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Simulate a user editing the deployed file.
    let out_path = ws.root().join(".mycode/instructions.md");
    fs::write(
        &out_path,
        "User-modified content that should be overwritten.",
    )
    .unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--force"])
        .assert_success();

    let output = ws.read_file(".mycode/instructions.md");
    assert!(
        !output.contains("User-modified content"),
        "--force should overwrite user-modified output; got:\n{output}"
    );
    assert!(
        output.contains("Mycode") || output.contains("dotagents"),
        "overwritten output should contain rendered content; got:\n{output}"
    );
}
